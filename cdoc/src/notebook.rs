use std::collections::HashMap;
use std::ops::Range;

use base64;
use pulldown_cmark::{CowStr, Event};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use serde_json::Value;
use serde_with::EnumMap;

use crate::document::{DocumentMetadata, IteratorConfig};
use crate::parsers::split::parse_code_string;
use crate::parsers::split_types::Output;
use crate::processors::shortcodes::ShortCode;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Notebook {
    pub(crate) metadata: NotebookMeta,
    pub(crate) nbformat: i64,
    pub(crate) nbformat_minor: i64,
    pub(crate) cells: Vec<Cell>,
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

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "output_type")]
pub enum CellOutput {
    #[serde(rename = "stream")]
    Stream {
        name: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutputValue {
    #[serde(rename = "text/plain")]
    Plain(
        #[serde(
        deserialize_with = "concatenate_deserialize",
        serialize_with = "concatenate_serialize"
        )]
        String,
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
    format: Option<String>,
    name: Option<String>,
    tags: Option<Vec<String>>,
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
    pub fn to_events(&self, config: &IteratorConfig) -> Vec<(Event, Range<usize>)> {
        match self {
            CellOutput::Stream { text, .. } => {
                vec![(
                    Event::Html(CowStr::Boxed(
                        format!(
                            r#"
                <div class="alert alert-info">
                    <p>{}</p>
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
                    OutputValue::Plain(_) => {
                        // let block = CodeBlock(Fenced(CowStr::Boxed(
                        //     "plaintext".to_string().into_boxed_str(),
                        // )));
                        // vec![
                        //     (Event::Start(block.clone()), (0..0)),
                        //     (Event::Text(CowStr::Borrowed(v)), (0..0)),
                        //     (Event::End(block), (0..0)),
                        // ]
                        vec![]
                    }
                    OutputValue::Image(v) => {

                        let code = ShortCode::new("output_image".to_string()).with_param("src".to_string(), format!("data:image/png;base64,{}", v));
                        let output = config.shortcode_renderer.render(&code, &config.template_context).expect("Could not render output");
                        vec![(
                            Event::Html(CowStr::Boxed(
                                output.into_boxed_str(),
                            )),
                            (0..0),
                        )]
                    }
                    OutputValue::Svg(v) => {
                        vec![(
                            Event::Html(CowStr::Boxed(
                                format!(
                                    "<figure><img><svg width=\"640px\" height=\"480px\">{}</svg></img></figure>",
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
    let source = base.into_iter().collect();
    Ok(source)
}

fn concatenate_serialize<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    serializer.collect_seq(value.split('\n'))
}

#[allow(unused)]
fn escape_string_deserialize(source: String) -> String {
    let escaped = source
        .chars()
        .flat_map(|c| match c {
            '\\' => r#"\\"#.chars().collect(),
            // '\'' => vec!['\\', '\''],
            // '\"' => vec!['\\', '\"'],
            // '±' => vec!['±'],
            _ => vec![c],
        })
        .collect::<String>();
    escaped
}

#[allow(unused)]
fn deserialize_png<'de, D>(input: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
{
    let base: String = Deserialize::deserialize(input)?;
    let bytes = base64::decode(base).map_err(|e| D::Error::custom(e.to_string()))?;
    // let source = load_from_memory(&bytes).map_err(|e| D::Error::custom(e.to_string()))?;
    Ok(bytes)
}

#[allow(unused)]
fn serialize_png<S>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    // serializer.collect_str(&base64::encode(value.as_bytes()))
    serializer.collect_str(&base64::encode(value))
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    use pulldown_cmark::html;

    use crate::notebook::Notebook;

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
