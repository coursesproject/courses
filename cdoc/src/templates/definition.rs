use crate::config::OutputFormat;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use tera::Context;
use thiserror::Error;
use toml::Value;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateDefinition {
    pub name: String,
    pub description: String,
    #[serde(flatten, rename = "type")]
    pub type_: TemplateType,
    pub templates: HashMap<OutputFormat, Template>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TemplateType {
    Shortcode(ShortcodeDefinition),
    Generic,
    Builtin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcodeDefinition {
    #[serde(rename = "type")]
    pub type_: ShortcodeType,
    pub parameters: Vec<ShortcodeParameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ShortcodeType {
    Inline,
    Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcodeParameter {
    pub name: String,
    pub description: String,
    pub optional: bool,
    pub param_type: ParameterType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ParameterType {
    Regular,
    Choice { choices: Vec<String> },
    Flag,
    Positional,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Template(TemplateSource);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TemplateSource {
    String(String),
    File(PathBuf),
}

impl Display for TemplateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            TemplateType::Shortcode(_) => "shortcode",
            TemplateType::Generic => "generic",
            TemplateType::Builtin => "builtin",
        };
        write!(f, "{}", name)
    }
}

pub fn load_template_definitions(
    path: PathBuf,
) -> anyhow::Result<HashMap<String, TemplateDefinition>> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;
            let ext = e.path().extension()?.to_str()?;
            if ext == ".yml" {
                Some(e)
            } else {
                None
            }
        })
        .map(|e| {
            let s = fs::read_to_string(e.path())?;
            let def: TemplateDefinition = serde_yaml::from_str(&s)?;
            Ok((def.name.clone(), def))
        })
        .collect()
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid parameter position '{0}'")]
    InvalidPosition(usize),
    #[error("Invalid parameter key '{0}'")]
    InvalidName(String),
    #[error("Invalid parameter value: {0}")]
    InvalidValue(String),
}

impl ParameterType {
    pub fn validate_value(&self, value: &String) -> Result<(), ValidationError> {
        match self {
            ParameterType::Regular => Ok(()),
            ParameterType::Choice { choices } => {
                choices.contains(value).then(|| ()).ok_or_else(|| {
                    ValidationError::InvalidValue(format!(
                        "The provided value {} must be one of {:?}",
                        &value, &choices
                    ))
                })
            }
            ParameterType::Flag => {
                value
                    .is_empty()
                    .then(|| ())
                    .ok_or(ValidationError::InvalidValue(
                        "Flag parameters must not have any value.".to_string(),
                    ))
            }
            ParameterType::Positional => {
                value
                    .is_empty()
                    .then(|| ())
                    .ok_or(ValidationError::InvalidValue(
                        "Positional parameters must not have any value.".to_string(),
                    ))
            }
        }
    }
}

impl TemplateDefinition {
    pub fn template_for_format(&self, format: OutputFormat) -> anyhow::Result<String> {
        let tp = self
            .templates
            .get(&format)
            .ok_or(anyhow!("Format not available"))?;
        match &tp.0 {
            TemplateSource::String(s) => Ok(s.clone()),
            TemplateSource::File(p) => Ok(fs::read_to_string(p)?),
        }
    }

    pub fn template_strings(&self) -> HashMap<OutputFormat, String> {
        self.templates
            .iter()
            .map(|(f, source)| (*f, self.template_for_format(*f).unwrap()))
            .collect()
    }

    pub fn has_format(&self, format: OutputFormat) -> anyhow::Result<()> {
        self.templates
            .get(&format)
            .ok_or(anyhow!("Format not supported by template"))?;
        Ok(())
    }

    pub fn validate_args(
        &self,
        args: &BTreeMap<String, String>,
    ) -> Result<Vec<Result<(), ValidationError>>, anyhow::Error> {
        if let TemplateType::Shortcode(s) = &self.type_ {
            let res: Vec<Result<(), ValidationError>> = args
                .iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    let param = if k.is_empty() {
                        s.parameters
                            .get(i)
                            .ok_or_else(|| ValidationError::InvalidPosition(i))?
                    } else {
                        s.parameters
                            .iter()
                            .find(|p| &p.name == k)
                            .ok_or_else(|| ValidationError::InvalidName(k.to_string()))?
                    };

                    param.param_type.validate_value(v)
                })
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
