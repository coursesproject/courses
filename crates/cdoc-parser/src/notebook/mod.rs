use base64;
use base64::Engine;
use std::collections::hash_map::DefaultHasher;

use anyhow::Result;

use crate::ast::Ast;
use crate::document::{CodeOutput, Document, Image, Metadata};

use crate::document;
use linked_hash_map::LinkedHashMap;
use nanoid::nanoid;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use serde_with::{formats::PreferOne, serde_as, EnumMap, OneOrMany};
use std::collections::HashMap;
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};

/// Top-level notebook structure (the type is a mostly complete implementation of the official
/// notebook specification (http://ipython.org/ipython-doc/3/notebook/nbformat.html).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Notebook {
    /// Information about the kernel and cell language.
    pub metadata: NotebookMeta,
    #[serde(default = "nbformat")]
    /// Notebook format (4 is the modern version)
    pub nbformat: i64,
    /// Minor version
    #[serde(default = "nbformat_minor")]
    pub nbformat_minor: i64,
    /// The actual content (cells) of the notebook
    pub cells: Vec<Cell>,
}

const fn nbformat() -> i64 {
    4
}

const fn nbformat_minor() -> i64 {
    5
}

/// Cell structure
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cell_type")]
pub enum Cell {
    /// Markdown cells are parsed as regular markdown
    #[serde(rename = "markdown")]
    Markdown {
        #[serde(flatten)]
        common: CellCommon,
    },
    /// Code cells can be executed and can contain saved outputs
    #[serde(rename = "code")]
    Code {
        #[serde(flatten)]
        common: CellCommon,

        /// Notebooks save how many time code cells have been run
        execution_count: Option<i64>,
        /// Execution results are saved as well
        outputs: Vec<CellOutput>,
    },
    /// Raw cells don't perform a function in the notebook but are used for specifying cdoc
    /// metadata.
    #[serde(rename = "raw")]
    Raw {
        #[serde(flatten)]
        common: CellCommon,
    },
}

/// Stuff common to all cell types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellCommon {
    #[serde(default = "get_id")]
    pub id: String,
    pub metadata: CellMeta,
    /// Cell sources are stored as lists of source lines in the notebook file.
    /// It is parsed into a single string for convenience (and is deserialized to the list representation).
    #[serde(
        deserialize_with = "concatenate_deserialize",
        serialize_with = "concatenate_serialize"
    )]
    pub source: String,
}

fn get_id() -> String {
    nanoid!()
}

/// Stream type used for stream output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    StdOut,
    StdErr,
}

impl ToString for StreamType {
    fn to_string(&self) -> String {
        match self {
            StreamType::StdOut => "stdout".to_string(),
            StreamType::StdErr => "stderr".to_string(),
        }
    }
}

/// Notebooks can save execution outputs in a variety of formats. Representing these makes it easy
/// to replicate their types in the rendered output.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "output_type")]
pub enum CellOutput {
    /// Used for outputs generated by stdio/stderr. For rendering purposes this is the same as regular text.
    #[serde(rename = "stream")]
    Stream {
        name: StreamType,
        #[serde(deserialize_with = "concatenate_deserialize")]
        text: String,
    },
    /// Complex output values (correspond to mime-types)
    #[serde(rename = "display_data", alias = "execute_result")]
    Data {
        #[serde(default)]
        execution_count: i64,
        /// The content of the output (may be multiple)
        #[serde_as(as = "EnumMap")]
        data: Vec<OutputValue>,
        metadata: LinkedHashMap<String, Value>,
    },
    #[serde(rename = "error")]
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

/// Complex cell outputs
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutputValue {
    /// Regular text
    #[serde(rename = "text/plain")]
    Plain(
        /// May be saved as either a string or a list of line strings.
        #[serde_as(
            deserialize_as = "OneOrMany<_, PreferOne>",
            serialize_as = "OneOrMany<_, PreferOne>"
        )]
        Vec<String>,
    ),
    /// Png image
    #[serde(rename = "image/png")]
    Image(
        #[serde_as(
            deserialize_as = "OneOrMany<_, PreferOne>",
            serialize_as = "OneOrMany<_, PreferOne>"
        )]
        Vec<String>,
    ),
    /// Svg image
    #[serde(rename = "image/svg+xml")]
    Svg(
        #[serde_as(
            deserialize_as = "OneOrMany<_, PreferOne>",
            serialize_as = "OneOrMany<_, PreferOne>"
        )]
        Vec<String>,
    ),
    /// Json
    #[serde(rename = "application/json")]
    Json(Value),
    /// Html
    #[serde(rename = "text/html")]
    Html(
        #[serde_as(
            deserialize_as = "OneOrMany<_, PreferOne>",
            serialize_as = "OneOrMany<_, PreferOne>"
        )]
        Vec<String>,
    ),
    /// Javascript
    #[serde(rename = "application/javascript")]
    Javascript(String),
}

type Dict = HashMap<String, Value>;

/// Notebook metadata. Currently not precisely specified.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NotebookMeta {
    /// Kernel specification
    pub kernelspec: Option<LinkedHashMap<String, Value>>,
    #[serde(flatten)]
    pub optional: Dict,
}

/// Controls cell display and function in notebook applications and is also used for rendering outputs.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CellMeta {
    /// Cell is collapsed (can be shown).
    pub collapsed: Option<bool>,
    pub autoscroll: Option<Value>,
    pub deletable: Option<bool>,
    /// JupyterLab specific options
    pub jupyter: Option<JupyterLabMeta>,
    pub format: Option<String>,
    pub name: Option<String>,
    /// Tags are useful for creating custom flags for cells.
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub additional: Dict,
}

/// Extra metadata for the JupyterLab application
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JupyterLabMeta {
    /// Hide cell outputs
    pub outputs_hidden: Option<bool>,
    /// Hide cell source but show outputs
    pub source_hidden: Option<bool>,
}

impl Notebook {
    /// Get cdoc frontmatter from notebook (this must be a raw cell at the top of the document).
    pub fn get_front_matter(&self) -> Result<Metadata, serde_yaml::Error> {
        match &self.cells[0] {
            Cell::Raw { common } => Ok(serde_yaml::from_str(&common.source)?),
            _ => Ok(Metadata::default()),
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

impl From<Vec<CellOutput>> for CodeOutput {
    fn from(value: Vec<CellOutput>) -> Self {
        let mut outputs = Vec::new();
        for output in value {
            match output {
                CellOutput::Stream { text, .. } => {
                    outputs.push(document::OutputValue::Text(text));
                }
                CellOutput::Data { data, .. } => {
                    for v in data {
                        match v {
                            OutputValue::Plain(s) => {
                                outputs.push(document::OutputValue::Plain(s.join("")));
                            }
                            OutputValue::Image(i) => {
                                outputs.push(document::OutputValue::Image(Image::Png(i.join(""))));
                            }
                            OutputValue::Svg(i) => {
                                outputs.push(document::OutputValue::Image(Image::Svg(i.join(""))));
                            }
                            OutputValue::Json(s) => {
                                outputs.push(document::OutputValue::Json(s));
                            }
                            OutputValue::Html(s) => {
                                outputs.push(document::OutputValue::Html(s.join("")));
                            }
                            OutputValue::Javascript(s) => {
                                outputs.push(document::OutputValue::Javascript(s));
                            }
                        }
                    }
                }
                CellOutput::Error { evalue, .. } => {
                    outputs.push(document::OutputValue::Error(evalue));
                }
            }
        }

        CodeOutput { values: outputs }
    }
}

pub fn notebook_to_doc(nb: Notebook, accept_draft: bool) -> Result<Option<Document<Ast>>> {
    let mut writer = BufWriter::new(Vec::new());

    let mut output_map = HashMap::new();

    let mut doc_meta = None;

    for cell in nb.cells {
        match &cell {
            Cell::Markdown { common } => {
                write!(&mut writer, "\n{}\n", common.source)?;
            }
            Cell::Code {
                common, outputs, ..
            } => {
                let attr = common
                    .metadata
                    .tags
                    .as_ref()
                    .map(|tags| tags.join(", "))
                    .unwrap_or(String::new());
                let full = format!("#| tags: {}\n{}\n", attr, common.source);

                write!(&mut writer, "\n```python, cell\n{}```\n", full)?;

                let mut hasher = DefaultHasher::new();
                full.hash(&mut hasher);
                output_map.insert(hasher.finish(), CodeOutput::from(outputs.clone()));
            }
            Cell::Raw { common } => {
                if let Ok(meta) = serde_yaml::from_str::<Metadata>(&common.source) {
                    if !accept_draft && meta.draft {
                        return Ok(None);
                    } else {
                        doc_meta = Some(meta);
                    }
                }
            }
        }
    }

    let source = String::from_utf8(writer.into_inner()?)?;
    // println!("{source}");

    let mut doc = Document::try_from(source.as_str())?;
    doc.code_outputs = output_map;
    doc.meta = doc_meta.unwrap_or_default();

    Ok(Some(doc))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast;
    use crate::ast::{Block, Command, Inline};
    use crate::code_ast::types::{CodeContent, CodeElem};
    use crate::common::Span;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    #[test]
    fn deserialize() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test_deserialize.ipynb");
        let bf = BufReader::new(File::open(d).expect("Could not open file"));
        let _nb: Notebook = serde_json::from_reader(bf).expect("Deserialization failed");

        println!("Done");
    }

    #[test]
    fn notebook_to_doc() {
        let nb = Notebook {
            metadata: Default::default(),
            nbformat: 0,
            nbformat_minor: 0,
            cells: vec![
                Cell::Markdown {
                    common: CellCommon {
                        id: "id".to_string(),
                        metadata: Default::default(),
                        source: "# Heading\n#func".to_string(),
                    },
                },
                Cell::Code {
                    common: CellCommon {
                        id: "id".to_string(),
                        metadata: Default::default(),
                        source: "print('x')".to_string(),
                    },
                    execution_count: None,
                    outputs: vec![CellOutput::Data {
                        execution_count: 0,
                        data: vec![OutputValue::Plain(vec!["x".to_string()])],
                        metadata: Default::default(),
                    }],
                },
            ],
        };

        let expected = Document {
            meta: Default::default(),
            content: Ast {
                blocks: vec![
                    Block::Heading {
                        lvl: 1,
                        id: None,
                        classes: vec![],
                        inner: vec![Inline::Text("Heading".into())],
                    },
                    Block::Plain(vec![Inline::Command(Command {
                        function: "func".into(),
                        label: None,
                        parameters: vec![],
                        body: None,
                        span: Span::new(11, 16),
                        global_idx: 0,
                    })]),
                    Block::Plain(vec![Inline::CodeBlock(ast::CodeBlock {
                        label: None,
                        source: CodeContent {
                            blocks: vec![CodeElem::Src("print('x')\n\n".into())],
                            meta: LinkedHashMap::from_iter(
                                [("tags".into(), "".into())].into_iter(),
                            ),
                            hash: 14521985544978239724,
                        },
                        attributes: vec!["python".into(), "cell".into()],
                        display_cell: false,
                        global_idx: 0,
                        span: Span::new(18, 58),
                    })]),
                ],
                source: "\n# Heading\n#func\n\n```python, cell\n#| tags: \nprint('x')\n```\n"
                    .into(),
            },
            code_outputs: HashMap::from([(
                14521985544978239724,
                CodeOutput {
                    values: vec![document::OutputValue::Plain("x".into())],
                },
            )]),
        };
        let parsed = super::notebook_to_doc(nb, true)
            .expect("parsing errors")
            .unwrap();

        assert_eq!(expected, parsed);
    }
}
