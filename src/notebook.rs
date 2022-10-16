use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::Tag::CodeBlock;
use pulldown_cmark::{CowStr, Event, Options, Parser};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::iter::FlatMap;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Serialize, Deserialize, Debug)]
pub struct Notebook {
    metadata: NotebookMeta,
    nbformat: i64,
    nbformat_minor: i64,
    cells: Vec<Cell>,
}

type Dict = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug)]
pub struct NotebookMeta {
    kernelspec: HashMap<String, Value>,
    #[serde(flatten)]
    optional: Dict,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CellMeta {
    collapsed: Option<bool>,
    autoscroll: Option<Value>,
    deletable: Option<bool>,
    format: Option<String>,
    name: Option<String>,
    tags: Option<Vec<String>>,
    #[serde(flatten)]
    additional: Dict,
}

fn concatenate_deserialize<'de, D>(input: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let base: Vec<String> = Deserialize::deserialize(input)?;
    Ok(base.into_iter().collect())
}

fn concatenate_serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(value.split("\n"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CellCommon {
    pub metadata: CellMeta,
    #[serde(
        deserialize_with = "concatenate_deserialize",
        serialize_with = "concatenate_serialize"
    )]
    pub source: String,
}

type CellOutput = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cell_type")]
pub enum Cell {
    #[serde(rename = "markdown")]
    Markdown {
        #[serde(flatten)]
        common: CellCommon,
    },
    #[serde(rename = "code")]
    Code {
        #[serde(flatten)]
        common: CellCommon,

        execution_count: Option<i64>,

        outputs: Vec<CellOutput>,
    },
    #[serde(rename = "raw")]
    Raw {
        #[serde(flatten)]
        common: CellCommon,
    },
}

pub enum CellEventIterator<'a, 'b> {
    Markdown {
        cell: &'a Cell,
        parser: Parser<'a, 'b>,
    },
    Code {
        cell: &'a Cell,
        events: IntoIter<Event<'a>>,
    },
    Raw {
        cell: &'a Cell,
    },
}

impl<'a> IntoIterator for &'a Cell {
    type Item = Event<'a>;
    type IntoIter = CellEventIterator<'a, 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Cell::Markdown { common } => CellEventIterator::Markdown {
                cell: &self,
                parser: Parser::new_ext(&common.source, Options::all()),
            },
            Cell::Code { common, .. } => {
                let cblock = CodeBlock(Fenced(CowStr::Boxed("python".into())));
                CellEventIterator::Code {
                    cell: &self,
                    events: vec![
                        Event::Start(cblock.clone()),
                        Event::Text(CowStr::Borrowed(&common.source)),
                        Event::End(cblock),
                    ]
                    .into_iter(),
                }
            }
            Cell::Raw { .. } => CellEventIterator::Raw { cell: &self },
        }
    }
}

impl<'a, 'b> Iterator for CellEventIterator<'a, 'b> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CellEventIterator::Markdown { parser, .. } => parser.next(),
            CellEventIterator::Code { events, .. } => events.next(),
            CellEventIterator::Raw { .. } => None,
        }
    }
}

pub struct NotebookIterator<'a, 'b> {
    iter: FlatMap<
        Iter<'a, Cell>,
        CellEventIterator<'a, 'b>,
        fn(&'a Cell) -> CellEventIterator<'a, 'b>,
    >,
}

impl<'a> IntoIterator for &'a Notebook {
    type Item = Event<'a>;
    type IntoIter = NotebookIterator<'a, 'a>;

    fn into_iter(self) -> Self::IntoIter {
        NotebookIterator {
            iter: self.cells.iter().flat_map(|c| c.into_iter()),
        }
    }
}

impl<'a, 'b> Iterator for NotebookIterator<'a, 'b> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

//
// impl<'a> IntoIterator for &'a Cell {
//     type Item = Event<'a>;
//     type IntoIter = Parser<'a, 'a>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Cell::Markdown { common } => {
//                 Parser::new(&common.source)
//             }
//             Cell::Code { common, .. } => {
//
//             }
//             _ => Parser::new("")
//         }
//     }
// }

#[cfg(test)]
mod tests {

    use crate::notebook::Notebook;
    use pulldown_cmark::html;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    #[test]
    fn deserialize() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/test.ipynb");
        let bf = BufReader::new(File::open(d).expect("Could not open file"));
        let nb: Notebook = serde_json::from_reader(bf).expect("Deserialization failed");

        println!("Done");
    }

    #[test]
    fn html_out() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/test.ipynb");

        let bf = BufReader::new(File::open(d).expect("Could not open file"));
        let nb: Notebook = serde_json::from_reader(bf).expect("Deserialization failed");

        let mut html_output = String::new();
        html::push_html(&mut html_output, nb.into_iter());

        // println!("{}", html_output);
    }
}
