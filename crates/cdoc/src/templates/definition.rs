use anyhow::{anyhow, Context as AContext};

use cowstr::CowStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

use crate::renderers::RenderedParam;
use crate::templates::precompiled::{PrecompiledFormat, PrecompiledTemplate};
use thiserror::Error;
use walkdir::WalkDir;

/// Type that reflects the template definitions specified in yml files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateDefinition {
    /// The given name is only used if using the template in another template (e.g. for documentation purposes)
    pub name: String,
    /// A private template is only meant to be used in project configuration.
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub value_template: bool,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: TemplateType,
    pub script: Option<String>,
    #[serde(default)]
    pub required_meta: Vec<String>,
    /// Only present for shortcodes
    pub shortcode: Option<ShortcodeDefinition>,
    /// A map of the templates for each defined output format
    pub templates: HashMap<String, TemplateSource>,
    /// Optional examples (useful for generating documentation)
    pub examples: Option<Vec<Example>>,
}

/// Specification of a template example.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Example {
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// The markdown source for the example (should generally contain a call to the shortcode)
    pub body: String,
}

/// The three template types.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    /// Usable in the content files
    Shortcode,
    /// Used for rendering documents into a template defining a layout
    Layout,
    /// Any template that corresponds to a markdown element or any of the notebook-specific elements.
    Builtin,
}

/// Describes a shortcode
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcodeDefinition {
    pub kind: ShortcodeType,
    #[serde(default)]
    pub accept_arbitrary_params: bool,
    /// The ordering of the parameters determine their expected position if positional arguments
    /// are used.
    #[serde(default)]
    pub parameters: Vec<ShortcodeParameter>,
}

/// Whether a shortcode has a body or not
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ShortcodeType {
    Inline,
    Block,
}

/// Describes a parameter for a shortcode
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcodeParameter {
    pub name: CowStr,
    pub description: CowStr,
    /// Whether the argument can be omitted
    #[serde(default)]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_: ParameterType,
}

/// Parameter types. This must currently be either an arbitrary string or a predefined set of valid
/// values.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Regular,
    Choice(Vec<CowStr>),
}

impl Display for ParameterType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterType::Regular => write!(f, "regular"),
            ParameterType::Choice(cs) => write!(f, "{:?}", cs),
        }
    }
}

/// Template sources can be defined in various ways.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TemplateSource {
    /// Raw source specified in yml file.
    String(String),
    /// Path to a file that contains the source (useful for large templates).
    File(PathBuf),
    /// Really just uses the exact template of another format
    Derive(String),
    Precompiled(PrecompiledTemplate, PrecompiledFormat),
}

impl Display for TemplateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            TemplateType::Shortcode => "shortcode",
            TemplateType::Layout => "layout",
            TemplateType::Builtin => "builtin",
        };
        write!(f, "{}", name)
    }
}

/// Load script filters from file
pub fn get_filters_from_files(dir: PathBuf) -> anyhow::Result<HashMap<String, String>> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;
            let ext = e.path().extension()?.to_str()?;
            if ext == "rhai" {
                Some(e)
            } else {
                None
            }
        })
        .map(|e| {
            let f_name = e.file_name().to_str().unwrap();
            let dot_idx = f_name.find('.').unwrap();
            let f_base = f_name[..dot_idx].to_string();
            Ok((f_base, fs::read_to_string(e.path())?))
        })
        .collect()
}

/// Maps the template definition files to a map of Tera templates that is used by the [TemplateManager]
pub fn get_templates_from_definitions(
    definitions: &HashMap<String, TemplateDefinition>,
    dir: PathBuf,
) -> Vec<(String, String)> {
    definitions
        .iter()
        .filter(|(_id, def)| !def.value_template)
        .flat_map(|(id, def)| {
            def.template_strings(dir.join("sources"))
                .iter()
                .map(|(format, source)| {
                    let name = format!("{}.{}", id, format);
                    (name, source.clone())
                })
                .collect::<Vec<(String, String)>>()
        })
        .collect()
}

/// Load template definitions from file
pub fn load_template_definitions(
    path: PathBuf,
) -> anyhow::Result<HashMap<String, TemplateDefinition>> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;
            let ext = e.path().extension()?.to_str()?;
            if ext == "yml" {
                Some(e)
            } else {
                None
            }
        })
        .map(|e| {
            let s = fs::read_to_string(e.path())?;
            let def: TemplateDefinition = serde_yaml::from_str(&s)?;
            if def.type_ == TemplateType::Shortcode {
                def.shortcode
                    .as_ref()
                    .ok_or(anyhow!("Missing shortcode definition for type 'shortcode'"))?;
            } else if def.shortcode.is_some() {
                return Err(anyhow!(
                    "Shortcode definition must only be present for type 'shortcode'"
                ));
            }

            let f_name = e.file_name().to_str().unwrap();
            let dot_idx = f_name.find('.').unwrap();
            let f_base = format!("{}_{}", &def.type_, &f_name[..dot_idx]);

            Ok((f_base, def))
        })
        .collect()
}

/// Template parameter validation errors.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Parameter is in the wrong position
    #[error("Invalid parameter position '{0}'")]
    InvalidPosition(usize),
    /// A named parameter doesn't exist
    #[error("Invalid parameter key '{0}'")]
    InvalidName(String),
    /// The value is invalid (only for choice types)
    #[error("Invalid parameter value: {0}")]
    InvalidValue(String),
    /// A required parameter is missing
    #[error("Required parameter {0} missing")]
    RequiredParameter(String),
}

impl ParameterType {
    pub fn validate(&self, value: &RenderedParam) -> Result<(), ValidationError> {
        match self {
            ParameterType::Regular => Ok(()),
            ParameterType::Choice(choices) => {
                let v = &value.value;
                choices.contains(v).then_some(()).ok_or_else(|| {
                    ValidationError::InvalidValue(format!(
                        "The provided value {} must be one of {:?}",
                        v, &choices
                    ))
                })
            }
        }
    }
}

impl TemplateDefinition {
    pub fn template_for_format(&self, base_path: PathBuf, format: &str) -> anyhow::Result<String> {
        let tp = self
            .templates
            .get(format)
            .ok_or(anyhow!("Format not available"))?;
        match &tp {
            TemplateSource::String(s) => Ok(s.clone()),
            TemplateSource::File(p) => Ok(fs::read_to_string(base_path.join(p))?),
            TemplateSource::Derive(parent) => self.template_for_format(base_path, parent),
            TemplateSource::Precompiled(_, _) => Ok(String::new()),
        }
    }

    pub fn template_strings(&self, base_path: PathBuf) -> HashMap<String, String> {
        self.templates
            .keys()
            .map(|f| {
                (
                    f.to_string(),
                    self.template_for_format(base_path.clone(), f).unwrap(),
                )
            })
            .collect()
    }

    pub fn get_format(&self, format: &str) -> anyhow::Result<&TemplateSource> {
        self.templates
            .get(format)
            .ok_or(anyhow!("Format not supported by template"))
    }

    pub fn validate_args(
        &self,
        args: &[RenderedParam],
    ) -> Result<Vec<anyhow::Result<()>>, anyhow::Error> {
        if let TemplateType::Shortcode = &self.type_ {
            let s = self.shortcode.as_ref().unwrap();
            let res: Vec<Result<(), anyhow::Error>> = args
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if let Some(key) = &p.key {
                        if s.accept_arbitrary_params {
                            Ok(())
                        } else {
                            s.parameters
                                .iter()
                                .find(|sp| sp.name == key)
                                .map(|sp| sp.type_.validate(p))
                                .ok_or(ValidationError::InvalidName(key.to_string()))?
                        }
                    } else {
                        s.parameters
                            .get(i)
                            .map(|sp| sp.type_.validate(p))
                            .ok_or(ValidationError::InvalidValue(p.value.to_string()))?
                    }
                })
                .map(|r| r.context(format!("when parsing shortcode '{}'", self.name)))
                .collect();
            Ok(res)
        } else {
            Err(anyhow!(
                "Invalid template type {}, must be 'shortcode'",
                self.type_
            ))
        }
    }
}
