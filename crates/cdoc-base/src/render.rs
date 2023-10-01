use crate::module::Module;
use crate::node::Node;
use anyhow::Result;
use std::io::Write;
use tera::{Context, Tera};

pub struct RenderContext {
    module: Module,
}

// pub trait DocumentRenderer {
//     fn render(&mut self, doc: &Document, ctx: &RenderContext, buf: impl Write);
// }

pub trait Renderer {
    fn render(&mut self, node: &Node, ctx: &RenderContext, buf: impl Write) -> anyhow::Result<()>;

    fn render_inner(&mut self, node: &Node, ctx: &RenderContext) -> Result<String> {
        let mut buf = Vec::new();
        self.render(node, ctx, &mut buf)?;
        Ok(String::from_utf8(buf)?)
    }

    fn render_vec(
        &mut self,
        nodes: &[Node],
        ctx: &RenderContext,
        mut buf: impl Write,
    ) -> Result<()> {
        for node in nodes {
            self.render(node, ctx, &mut buf)?;
        }
        Ok(())
    }

    fn render_vec_inner(&mut self, nodes: &[Node], ctx: &RenderContext) -> Result<String> {
        let mut buf = Vec::new();
        for node in nodes {
            self.render(node, ctx, &mut buf)?;
        }
        Ok(String::from_utf8(buf)?)
    }
}

pub struct GenericRenderer {
    tera: Tera,
}
//
// impl GenericRenderer {
//     fn create_context(&mut self, node: &Node, ctx: &RenderContext) -> Result<Context> {
//         let mut template_context = Context::new();
//         if let Some(children) = &node.children {
//             let inner = self.render_vec_inner(children, ctx)?;
//             template_context.insert("children", &inner);
//         }
//
//         for attr in &node.arguments {}
//
//         Ok(template_context)
//     }
// }
//
// impl Renderer for GenericRenderer {
//     fn render(&mut self, node: &Node, ctx: &RenderContext, buf: impl Write) -> anyhow::Result<()> {
//         self.tera.render(&node.type_id, &Context::new())?;
//         Ok(())
//     }
// }
