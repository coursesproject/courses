use cdoc_parser::code_ast::types::CodeContent;
use cdoc_parser::document::CodeOutput;
use cowstr::CowStr;

use cdoc_parser::Span;

use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{CustomType, Dynamic, TypeBuilder};

#[derive(Clone)]
pub(crate) struct ScriptCodeBlock {
    source: CodeContent,
    attributes: Vec<CowStr>,
    outputs: Dynamic,
    display_cell: bool,
    global_idx: usize,
    pos: Span,
}

impl ScriptCodeBlock {
    pub fn new(
        source: &CodeContent,
        attributes: &[CowStr],
        outputs: &Option<&mut CodeOutput>,
        display_cell: bool,
        global_idx: usize,
        pos: &Span,
    ) -> Self {
        ScriptCodeBlock {
            source: source.clone(),
            attributes: attributes.to_vec(),
            outputs: to_dynamic(outputs).unwrap(),
            display_cell,
            global_idx,
            pos: pos.clone(),
        }
    }

    pub fn apply_changes(
        self,
        source: &mut CodeContent,
        tags: &mut Vec<CowStr>,
        outputs: Option<&mut CodeOutput>,
        display_cell: &mut bool,
        global_idx: &mut usize,
    ) -> anyhow::Result<()> {
        *source = self.source;

        *tags = self.attributes;
        *display_cell = self.display_cell;
        *global_idx = self.global_idx;

        if let Some(out) = outputs {
            *out = from_dynamic(&self.outputs)?;
        }

        Ok(())
    }
}

impl CustomType for ScriptCodeBlock {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("CodeBlock")
            .with_get_set(
                "source",
                |s: &mut Self| s.source.clone(),
                |s: &mut Self, v: CodeContent| s.source = v,
            )
            .with_get_set(
                "tags",
                |s: &mut Self| s.attributes.clone(),
                |s: &mut Self, v: Vec<CowStr>| s.attributes = v,
            )
            .with_get_set(
                "outputs",
                |s: &mut Self| s.outputs.clone(),
                |s: &mut Self, v: Dynamic| s.outputs = v,
            )
            .with_get_set(
                "display_cell",
                |s: &mut Self| s.display_cell,
                |s: &mut Self, v: bool| s.display_cell = v,
            )
            .with_get_set(
                "global_idx",
                |s: &mut Self| s.global_idx,
                |s: &mut Self, v: usize| s.global_idx = v,
            );
    }
}
