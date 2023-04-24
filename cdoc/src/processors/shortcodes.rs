use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::ast;
use crate::ast::ShortcodeIdx;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Options, Parser};
use serde::{Deserialize, Serialize};
use tera::Tera;
use thiserror::Error;

use crate::parsers::shortcodes::{parse_shortcode, Rule};
use crate::processors::{MarkdownPreprocessor, PreprocessorConfig, PreprocessorContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcodesConfig;

#[typetag::serde(name = "shortcodes")]
impl PreprocessorConfig for ShortcodesConfig {
    fn build(&self, ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn MarkdownPreprocessor>> {
        Ok(Box::new(Shortcodes {
            tera: ctx.tera.clone(),
            file_ext: ctx.output_format.template_extension().to_string(),
        }))
    }
}

#[derive(Error, Debug)]
pub enum ShortCodeProcessError {
    // #[error("shortcode template error: {:#}", .source)]
    Tera {
        #[from]
        source: tera::Error,
    },
    // #[error("shortcode syntax error: {}", .0)]
    Pest(#[from] Box<pest::error::Error<Rule>>),
}

impl Display for ShortCodeProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortCodeProcessError::Tera { source } => {
                Display::fmt(&source, f)?;
                let mut e = source.source();
                while let Some(next) = e {
                    Display::fmt(&next, f)?;

                    e = next.source();
                }
                Ok(())
            }
            ShortCodeProcessError::Pest(inner) => Display::fmt(&inner, f),
        }
    }
}
//
// #[derive(Debug)]
// pub struct BoundTera {
//     tera: Tera,
//     pattern: String,
// }
//
// impl Serialize for BoundTera {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(&self.pattern)
//     }
// }
//
// struct StringVisitor;
//
// impl<'de> Visitor<'de> for StringVisitor {
//     type Value = String;
//
//     fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
//         formatter.write_str("A string representing the template search pattern")
//     }
//
//     fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Ok(String::from(v))
//     }
//
//     fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Ok(v)
//     }
// }
//
// impl<'de> Deserialize<'de> for BoundTera {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let pattern = deserializer.deserialize_str(StringVisitor)?;
//         Ok(BoundTera {
//             tera: Tera::new(&pattern).map_err(|e| D::Error::custom(e.to_string()))?,
//             pattern,
//         })
//     }
// }

#[derive(Debug)]
pub struct Shortcodes {
    tera: Tera,
    file_ext: String,
}

impl Shortcodes {
    pub fn new(pattern: &str, file_ext: &str) -> Result<Self, tera::Error> {
        Ok(Shortcodes {
            tera: Tera::new(pattern)?,
            file_ext: file_ext.to_string(),
        })
    }

    fn render_inline_template(
        &self,
        shortcode: &str,
        ctx: &tera::Context,
    ) -> anyhow::Result<String> {
        let code = parse_shortcode(shortcode)?;
        let name = format!("{}/{}.tera.{}", self.file_ext, code.name, self.file_ext);

        let mut ctx = ctx.clone();
        for (k, v) in code.parameters {
            ctx.insert(k, &v);
        }

        let res = self.tera.render(&name, &ctx)?;
        let res = res.replace("\n\n", "\n");
        Ok(res)
    }

    fn render_block_template(
        &self,
        shortcode: &str,
        body: &str,
        ctx: &tera::Context,
    ) -> Result<String, anyhow::Error> {
        let code = parse_shortcode(shortcode)?;
        let name = format!("{}/{}.tera.{}", self.file_ext, code.name, self.file_ext);

        let mut ctx = ctx.clone();

        for (k, v) in code.parameters {
            ctx.insert(k, &v);
        }

        let processed = self.process(body, &ctx)?;

        let body_final = if self.file_ext == "html" {
            let parser = Parser::new_ext(&processed, Options::all());
            let mut html = String::new();
            push_html(&mut html, parser);
            html
        } else {
            processed
        };

        ctx.insert("body", &body_final);
        let res = self.tera.render(&name, &ctx)?;
        let res = res.replace("\n\n", "\n");
        Ok(res)
    }
}

impl MarkdownPreprocessor for Shortcodes {
    fn name(&self) -> String {
        "Shortcode processor".to_string()
    }

    fn process(&self, input: &str, ctx: &tera::Context) -> Result<String, anyhow::Error> {
        let mut rest = input;
        let mut offset = 0;

        let mut result = String::new();

        let blocks = ast::find_all_blocks(input);

        while !rest.is_empty() {
            match ast::find_shortcode(rest) {
                None => {
                    result.push_str(rest);
                    rest = "";
                }

                Some(info) => {
                    match info {
                        ShortcodeIdx::Inline(start, end) => {
                            match blocks
                                .iter()
                                .find(|(bs, be)| bs < &(start + offset) && be >= &(end + offset))
                            {
                                None => {
                                    let pre = &rest[..start];
                                    let post = &rest[(end + 2)..];
                                    let tmp_name = rest[(start + 2)..(end - 1)].trim();

                                    let res = self.render_inline_template(tmp_name, ctx)?;

                                    result.push_str(pre);
                                    result.push_str(&res);

                                    rest = post; // Start next round after the current shortcode position
                                    offset += end + 2;
                                }
                                Some((_, block_end)) => {
                                    let relative = *block_end - offset;
                                    let pre = &rest[..relative];
                                    result.push_str(pre);
                                    rest = &rest[relative..];
                                    offset += relative;
                                }
                            }
                        }
                        ShortcodeIdx::Block { def, end } => {
                            match blocks
                                .iter()
                                .find(|(bs, be)| bs < &(def.1 + offset) && be > &(end.0 + offset))
                            {
                                None => {
                                    let pre = &rest[..def.0];
                                    let post = &rest[(end.1 + 2)..];

                                    let tmp_name = rest[(def.0 + 2)..(def.1 - 1)].trim();
                                    let body = rest[(def.1 + 2)..end.0].trim();

                                    let res = self.render_block_template(tmp_name, body, ctx)?;

                                    result.push_str(pre);
                                    result.push_str(&res);
                                    result.push('\n');

                                    rest = post; // Start next round after the current shortcode position
                                    offset += end.1 + 2;
                                }

                                Some((_, block_end)) => {
                                    let relative = *block_end - offset;
                                    let pre = &rest[..relative];
                                    result.push_str(pre);
                                    rest = &rest[relative..];
                                    offset += relative;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

impl Display for Shortcodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::find_shortcode;

    #[test]
    fn test_extract_inline() {
        let input = "This is some text {{ shortcode(arg=val) }} and some more text";
        let spec = find_shortcode(input).expect("Shortcode not found");

        match spec {
            ShortcodeIdx::Block { .. } => panic!("Wrong code type. Should be inline"),
            ShortcodeIdx::Inline(start, end) => {
                assert_eq!(start, 18);
                assert_eq!(end, 40);
            }
        }
    }

    #[test]
    fn test_extract_block() {
        let input = "This {% block(arg=val) %} is some text {% end %} and some more text";
        let spec = find_shortcode(input).expect("Shortcode not found");

        match spec {
            ShortcodeIdx::Block { def, end } => {
                assert_eq!(def.0, 5);
                assert_eq!(def.1, 23);
                assert_eq!(end.0, 39);
                assert_eq!(end.1, 46);
            }
            ShortcodeIdx::Inline(_, _) => panic!("Wrong code type. Should be block."),
        }
    }

    #[test]
    fn test_block_error() {
        let err_block_end = "This {% block(arg=val) %} is some text {% end and some more text";
        let err_block_start = "This {% block(arg=val) is some text {% end %} and some more text";
        let err_inline_start = "This is some text { shortcode(arg=val) }} and some more text";
        let err_inline_start2 = "This is some text shortcode(arg=val) }} and some more text";

        let msg: &str =
            "Invalid shortcode syntax should return None, but a code was returned instead.";
        assert!(find_shortcode(err_block_end).is_none(), "{}", msg);
        assert!(find_shortcode(err_block_start).is_none(), "{}", msg);
        assert!(find_shortcode(err_inline_start).is_none(), "{}", msg);
        assert!(find_shortcode(err_inline_start2).is_none(), "{}", msg);
    }
}
