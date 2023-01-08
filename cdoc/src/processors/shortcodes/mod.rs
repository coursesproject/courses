use std::collections::HashMap;

use tera::Tera;

pub use parser::*;
pub use processor::*;

mod parser;
mod processor;

pub struct ShortCode {
    pub(crate) name: String,
    pub(crate) parameters: HashMap<String, String>,
}

impl ShortCode {
    pub fn new<C: Into<String>>(name: C) -> Self {
        ShortCode {
            name: name.into(), parameters: HashMap::default()
        }
    }

    pub fn with_param<C: Into<String>, C2: Into<String>>(self, key: C, value: C2) -> Self {
        let mut next = self.parameters.clone();
        next.insert(key.into(), value.into());
        ShortCode {
            name: self.name,
            parameters: next,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ShortCodeRenderer {
    pub(crate) tera: Tera,
    pub(crate) file_ext: String,
}

impl ShortCodeRenderer {

    pub(crate) fn render(
        &self,
        shortcode: &ShortCode,
        ctx: &tera::Context,
    ) -> anyhow::Result<String> {
        let name = format!("{}/{}.tera.{}", self.file_ext, shortcode.name, self.file_ext);

        let mut ctx = ctx.clone();
        for (k, v) in &shortcode.parameters {
            ctx.insert(k, &v);
        }

        let res = self.tera.render(&name, &ctx)?;
        let res = res.replace("\n\n", "\n");
        Ok(res)
    }


}