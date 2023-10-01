use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tera::Context as TCtx;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrecompiledFormat {
    Html,
    Markdown,
    LaTeX,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrecompiledTemplate {
    Paragraph,
    Emphasis,
    Strong,
    HardBreak,
    SoftBreak,
}

impl PrecompiledTemplate {
    pub fn render(
        &self,
        format: &PrecompiledFormat,
        ctx: &TCtx,
        mut buf: impl Write,
    ) -> anyhow::Result<()> {
        match self {
            PrecompiledTemplate::Paragraph => match format {
                PrecompiledFormat::Html => write!(
                    &mut buf,
                    "<p>{}</p>",
                    ctx.get("body")
                        .ok_or(anyhow!("missing value"))?
                        .as_str()
                        .unwrap()
                )
                .context("rendering"),
                PrecompiledFormat::Markdown => writeln!(
                    &mut buf,
                    "{}",
                    ctx.get("value")
                        .ok_or(anyhow!("missing value"))?
                        .as_str()
                        .unwrap()
                )
                .context("rendering"),
                _ => Err(anyhow!(format!("unsupported format {:?}", format))),
            },
            PrecompiledTemplate::Emphasis => match format {
                PrecompiledFormat::Html => write!(
                    &mut buf,
                    "<em>{}</em>",
                    ctx.get("value")
                        .ok_or(anyhow!("missing value"))?
                        .as_str()
                        .unwrap()
                )
                .context("rendering"),
                PrecompiledFormat::Markdown => write!(
                    &mut buf,
                    "_{}_",
                    ctx.get("value")
                        .ok_or(anyhow!("missing value"))?
                        .as_str()
                        .unwrap()
                )
                .context("rendering"),
                _ => Err(anyhow!(format!("unsupported format {:?}", format))),
            },
            PrecompiledTemplate::Strong => match format {
                PrecompiledFormat::Html => write!(
                    &mut buf,
                    "<strong>{}</strong>",
                    ctx.get("value")
                        .ok_or(anyhow!("missing value"))?
                        .as_str()
                        .unwrap()
                )
                .context("rendering"),
                PrecompiledFormat::Markdown => write!(
                    &mut buf,
                    "**{}**",
                    ctx.get("value")
                        .ok_or(anyhow!("missing value"))?
                        .as_str()
                        .unwrap()
                )
                .context("rendering"),
                _ => Err(anyhow!(format!("unsupported format {:?}", format))),
            },
            PrecompiledTemplate::HardBreak => match format {
                PrecompiledFormat::Html => write!(&mut buf, "<br>",).context("rendering"),
                PrecompiledFormat::Markdown => write!(&mut buf, "\n\n",).context("rendering"),
                _ => Err(anyhow!(format!("unsupported format {:?}", format))),
            },
            PrecompiledTemplate::SoftBreak => match format {
                PrecompiledFormat::Html => write!(&mut buf, "",).context("rendering"),
                PrecompiledFormat::Markdown => writeln!(&mut buf).context("rendering"),
                _ => Err(anyhow!(format!("unsupported format {:?}", format))),
            },
        }?;
        Ok(())
    }
}
