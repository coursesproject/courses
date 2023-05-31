use crate::ast::{Ast, Block, CodeAttributes};
use crate::document::DocumentMetadata;
use crate::parsers::split::parse_code_string;
use crate::parsers::split_types::Output;
use anyhow::Context;
use base64;
use base64::Engine;
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::Tag::CodeBlock;
use pulldown_cmark::{CowStr, Event, OffsetIter, Options, Parser};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use serde_with::{formats::PreferOne, serde_as, EnumMap, OneOrMany};
use std::collections::HashMap;
use std::iter::FlatMap;
use std::ops::Range;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Notebook {
    pub(crate) metadata: NotebookMeta,
    #[serde(default = "nbformat")]
    pub(crate) nbformat: i64,
    #[serde(default = "nbformat_minor")]
    pub(crate) nbformat_minor: i64,
    pub(crate) cells: Vec<Cell>,
}

const fn nbformat() -> i64 {
    4
}

const fn nbformat_minor() -> i64 {
    5
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellCommon {
    pub metadata: CellMeta,
    #[serde(
        deserialize_with = "concatenate_deserialize",
        serialize_with = "concatenate_serialize"
    )]
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    StdOut,
    StdErr,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "output_type")]
pub enum CellOutput {
    #[serde(rename = "stream")]
    Stream {
        name: StreamType,
        #[serde(
            deserialize_with = "concatenate_deserialize",
            serialize_with = "concatenate_serialize"
        )]
        text: String,
    },
    #[serde(rename = "display_data", alias = "execute_result")]
    Data {
        execution_count: Option<i64>,
        #[serde_as(as = "EnumMap")]
        data: Vec<OutputValue>,
        metadata: HashMap<String, Value>,
    },
    #[serde(rename = "error")]
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutputValue {
    #[serde(rename = "text/plain")]
    Plain(
        #[serde_as(
            deserialize_as = "OneOrMany<_, PreferOne>",
            serialize_as = "OneOrMany<_, PreferOne>"
        )]
        Vec<String>,
    ),
    #[serde(rename = "image/png")]
    Image(String),
    #[serde(rename = "image/svg+xml")]
    Svg(String),
    #[serde(rename = "application/json")]
    Json(HashMap<String, Value>),
    #[serde(rename = "text/html")]
    Html(
        #[serde(
            deserialize_with = "concatenate_deserialize",
            serialize_with = "concatenate_serialize"
        )]
        String,
    ),
    #[serde(rename = "application/javascript")]
    Javascript(String),
}

type Dict = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NotebookMeta {
    pub(crate) kernelspec: Option<HashMap<String, Value>>,
    #[serde(flatten)]
    pub(crate) optional: Dict,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CellMeta {
    collapsed: Option<bool>,
    autoscroll: Option<Value>,
    deletable: Option<bool>,
    pub format: Option<String>,
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    additional: Dict,
}

impl Notebook {
    pub fn get_front_matter(&self) -> Result<DocumentMetadata, serde_yaml::Error> {
        match &self.cells[0] {
            Cell::Raw { common } => Ok(serde_yaml::from_str(&common.source)?),
            _ => Ok(DocumentMetadata::default()),
        }
    }

    pub fn map_cell(&self, f: fn(&Cell) -> anyhow::Result<Cell>) -> anyhow::Result<Notebook> {
        let cells = self.cells.iter().map(f);
        Ok(Notebook {
            metadata: self.metadata.clone(),
            nbformat: self.nbformat,
            nbformat_minor: self.nbformat_minor,
            cells: cells.collect::<anyhow::Result<Vec<Cell>>>()?,
        })
    }

    pub fn placeholder_notebook(&self) -> anyhow::Result<Notebook> {
        self.map_cell(|c| match c {
            Cell::Code {
                common,
                execution_count,
                ..
            } => {
                let def = parse_code_string(&common.source)?;
                let placeholder = def.write_string(false);
                Ok(Cell::Code {
                    common: CellCommon {
                        source: placeholder,
                        metadata: common.metadata.clone(),
                    },
                    execution_count: *execution_count,
                    outputs: Vec::new(),
                })
            }
            c => Ok(c.clone()),
        })
    }
}

impl CellOutput {
    pub fn to_events(&self) -> Vec<(Event, Range<usize>)> {
        match self {
            CellOutput::Stream { text, .. } => {
                vec![(
                    Event::Html(CowStr::Boxed(
                        format!(
                            r#"
                            <div class="notification is-info">
                                <pre>{}</pre>
                            </div>
                            "#,
                            text
                        )
                        .into_boxed_str(),
                    )),
                    (0..0),
                )]
            }
            CellOutput::Data { data, .. } => data
                .iter()
                .flat_map(|value| match value {
                    OutputValue::Plain(v) => {
                        let s: String = v.join("");
                        vec![(
                            Event::Html(CowStr::Boxed(
                                format!(
                                    r#"
                                    <div class="notification is-info">
                                        <pre>{}</pre>
                                    </div>
                                    "#,
                                    s
                                )
                                .into_boxed_str(),
                            )),
                            (0..0),
                        )]
                    }
                    OutputValue::Image(v) => {
                        vec![(
                            Event::Html(CowStr::Boxed(
                                format!("<img src=\"data:image/png;base64,{}\"></img>", v)
                                    .into_boxed_str(),
                            )),
                            (0..0),
                        )]
                    }
                    OutputValue::Svg(v) => {
                        vec![(
                            Event::Html(CowStr::Boxed(
                                format!(
                                    "<img><svg width=\"640px\" height=\"480px\">{}</svg></img>",
                                    v
                                )
                                .into_boxed_str(),
                            )),
                            (0..0),
                        )]
                    }
                    OutputValue::Json(v) => {
                        vec![(
                            Event::Text(CowStr::Boxed(format!("{:?}", v).into_boxed_str())),
                            (0..0),
                        )]
                    }
                    OutputValue::Html(v) => {
                        vec![(
                            Event::Html(CowStr::Boxed(v.to_string().into_boxed_str())),
                            (0..0),
                        )]
                    }
                    OutputValue::Javascript(v) => {
                        vec![(
                            Event::Html(CowStr::Boxed(
                                format!("<script>{}</script>", v).into_boxed_str(),
                            )),
                            (0..0),
                        )]
                    }
                })
                .collect(),
            CellOutput::Error { .. } => {
                vec![(
                    Event::Text(CowStr::Boxed("Error".to_string().into_boxed_str())),
                    (0..0),
                )]
            }
        }
    }
}

fn concatenate_deserialize<'de, D>(input: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let base: Vec<String> = Deserialize::deserialize(input)?;
    let source: String = base.into_iter().collect();
    //let source = unescape(&source);
    Ok(source)
}

fn concatenate_serialize<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let lines: Vec<&str> = value.split('\n').collect();
    let last = lines[lines.len() - 1];
    let mut new_lines: Vec<String> = lines[..lines.len() - 1]
        .iter()
        .map(|s| format!("{}\n", s))
        .collect();
    new_lines.push(last.to_string());
    serializer.collect_seq(new_lines)
}

#[allow(unused)]
fn deserialize_png<'de, D>(input: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let base: String = Deserialize::deserialize(input)?;
    let engine = base64::engine::general_purpose::STANDARD;
    let bytes = engine
        .decode(base)
        .map_err(|e| D::Error::custom(e.to_string()))?;
    // let source = load_from_memory(&bytes).map_err(|e| D::Error::custom(e.to_string()))?;
    Ok(bytes)
}

#[allow(unused)]
fn serialize_png<S>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let engine = base64::engine::general_purpose::STANDARD;
    serializer.collect_str(&engine.encode(value))
}

// #[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
// pub enum OutputType {
//     #[serde(rename = "text/plain")]
//     Plain,
//     #[serde(rename = "image/png")]
//     Image,
//     #[serde(rename = "application/json")]
//     Json,
// }

pub enum CellEventIterator<'a, 'b> {
    Markdown {
        parser: Box<OffsetIter<'a, 'b>>,
    },
    Code {
        events: Box<IntoIter<(Event<'a>, Range<usize>)>>,
    },
    Raw {
        source: &'a str,
    },
}

impl<'a> IntoIterator for &'a Cell {
    type Item = (Event<'a>, Range<usize>);
    type IntoIter = CellEventIterator<'a, 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Cell::Markdown { common } => CellEventIterator::Markdown {
                parser: Box::new(
                    Parser::new_ext(&common.source, Options::all()).into_offset_iter(),
                ),
            },
            Cell::Code {
                common, outputs, ..
            } => {
                let cblock = CodeBlock(Fenced(CowStr::Boxed("python".into())));
                let mut events = vec![
                    (Event::Start(cblock.clone()), (0..0)),
                    (
                        Event::Text(CowStr::Borrowed(&common.source)),
                        (0..common.source.len()),
                    ),
                    (
                        Event::End(cblock),
                        (common.source.len()..common.source.len()),
                    ),
                ];
                outputs
                    .iter()
                    .for_each(|o| events.append(&mut o.to_events()));
                CellEventIterator::Code {
                    events: Box::new(events.into_iter()),
                }
            }
            Cell::Raw { common } => CellEventIterator::Raw {
                source: &common.source,
            },
        }
    }
}

impl<'a, 'b> Iterator for CellEventIterator<'a, 'b> {
    type Item = (Event<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CellEventIterator::Markdown { parser, .. } => parser.next(),
            CellEventIterator::Code { events, .. } => events.next(),
            CellEventIterator::Raw { source } => {
                Some((Event::Text(CowStr::Borrowed(source)), 0..0))
            }
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
    type Item = (Event<'a>, Range<usize>);
    type IntoIter = NotebookIterator<'a, 'a>;

    fn into_iter(self) -> Self::IntoIter {
        NotebookIterator {
            iter: self.cells.iter().flat_map(|c| c.into_iter()),
        }
    }
}

impl<'a, 'b> Iterator for NotebookIterator<'a, 'b> {
    type Item = (Event<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl TryFrom<Cell> for Vec<Block> {
    type Error = anyhow::Error;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        Ok(match value {
            Cell::Markdown { common } => {
                let ast: Ast = Ast::make_from_iter(Parser::new_ext(&common.source, Options::all()))
                    .context("when parsing markdown notebook cell")?;
                ast.0
            }
            Cell::Code {
                common, outputs, ..
            } => {
                vec![Block::CodeBlock {
                    source: common.source,
                    reference: None,
                    attr: CodeAttributes {
                        editable: true,
                        fold: common.metadata.collapsed.unwrap_or(false),
                    },
                    tags: common.metadata.tags,
                    outputs,
                }]
            }
            Cell::Raw { .. } => {
                vec![]
            }
        })
    }
}

// impl From<Cell> for Vec<Block> {
//     fn from(value: Cell) -> Self {
//         match value {
//             Cell::Markdown { common } => {
//                 let ast: Ast = Ast::make_from_iter(Parser::new_ext(&common.source, Options::all())).unwrap();
//                 ast.0
//             }
//             Cell::Code {
//                 common, outputs, ..
//             } => {
//                 vec![Block::CodeBlock {
//                     source: common.source,
//                     reference: None,
//                     attr: CodeAttributes {
//                         editable: true,
//                         fold: common.metadata.collapsed.unwrap_or(false),
//                     },
//                     tags: common.metadata.tags,
//                     outputs,
//                 }]
//             }
//             Cell::Raw { .. } => {
//                 vec![]
//             }
//         }
//     }
// }

impl TryFrom<Notebook> for Ast {
    type Error = anyhow::Error;

    fn try_from(value: Notebook) -> Result<Self, Self::Error> {
        Ok(Ast(value
            .cells
            .into_iter()
            .map(|c| -> anyhow::Result<Vec<Block>> { c.try_into() })
            .collect::<anyhow::Result<Vec<Vec<Block>>>>()?
            .into_iter()
            .flatten()
            .collect()))
    }
}

// impl From<Notebook> for Ast {
//     fn from(value: Notebook) -> Self {
//         Ast(value
//             .cells
//             .into_iter()
//             .flat_map(|c| -> Vec<Block> { c.into() })
//             .collect())
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
        let _nb: Notebook = serde_json::from_reader(bf).expect("Deserialization failed");

        println!("Done");
    }

    #[test]
    fn html_out() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/test.ipynb");

        let bf = BufReader::new(File::open(d).expect("Could not open file"));
        let nb: Notebook = serde_json::from_reader(bf).expect("Deserialization failed");

        let mut html_output = String::new();
        html::push_html(&mut html_output, nb.into_iter().map(|(e, _)| e));

        // println!("{}", html_output);
    }
}
