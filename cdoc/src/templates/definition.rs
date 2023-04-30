use crate::templates::TemplateContext;
use anyhow::anyhow;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

use thiserror::Error;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: TemplateType,
    pub shortcode: Option<ShortcodeDefinition>,
    pub templates: HashMap<String, Template>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    Shortcode,
    Layout,
    Builtin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcodeDefinition {
    pub kind: ShortcodeType,
    #[serde(default)]
    pub parameters: Vec<ShortcodeParameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ShortcodeType {
    Inline,
    Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcodeParameter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_: ParameterType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Regular,
    Choice(Vec<String>),
    Flag,
    Positional,
}

impl Display for ParameterType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterType::Regular => write!(f, "regular"),
            ParameterType::Choice(cs) => write!(f, "{:?}", cs),
            ParameterType::Flag => write!(f, "flag"),
            ParameterType::Positional => write!(f, "positional"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Template(TemplateSource);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TemplateSource {
    String(String),
    File(PathBuf),
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

pub fn get_templates_from_definitions(
    definitions: &HashMap<String, TemplateDefinition>,
    dir: PathBuf,
) -> Vec<(String, String)> {
    definitions
        .iter()
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
    pub fn validate_value(&self, value: String) -> Result<(), ValidationError> {
        match self {
            ParameterType::Regular => Ok(()),
            ParameterType::Choice(choices) => {
                choices.contains(&value).then_some(()).ok_or_else(|| {
                    ValidationError::InvalidValue(format!(
                        "The provided value {} must be one of {:?}",
                        &value, &choices
                    ))
                })
            }
            ParameterType::Flag => {
                value
                    .is_empty()
                    .then_some(())
                    .ok_or(ValidationError::InvalidValue(
                        "Flag parameters must not have any value.".to_string(),
                    ))
            }
            ParameterType::Positional => {
                value
                    .is_empty()
                    .then_some(())
                    .ok_or(ValidationError::InvalidValue(
                        "Positional parameters must not have any value.".to_string(),
                    ))
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
        match &tp.0 {
            TemplateSource::String(s) => Ok(s.clone()),
            TemplateSource::File(p) => Ok(fs::read_to_string(base_path.join(p))?),
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

    pub fn has_format(&self, format: &str) -> anyhow::Result<()> {
        self.templates
            .get(format)
            .ok_or(anyhow!("Format not supported by template"))?;
        Ok(())
    }

    pub fn validate_args(
        &self,
        args: &TemplateContext,
    ) -> Result<Vec<Result<(), ValidationError>>, anyhow::Error> {
        if let TemplateType::Shortcode = &self.type_ {
            let s = self.shortcode.as_ref().unwrap();
            let res: Vec<Result<(), ValidationError>> = args
                .map
                .iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    let param = if k.is_empty() {
                        s.parameters
                            .get(i)
                            .ok_or(ValidationError::InvalidPosition(i))?
                    } else {
                        s.parameters
                            .iter()
                            .find(|p| &p.name == k)
                            .ok_or_else(|| ValidationError::InvalidName(k.to_string()))?
                    };

                    param.type_.validate_value(v.to_string())
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
