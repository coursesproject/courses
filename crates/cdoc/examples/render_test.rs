use cdoc::renderers::base::ElementRenderer;
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::xml_writer::write_elements_to_xml;
use cdoc_base::node::Node;
use cdoc_parser::raw::{parse_to_doc, ComposedMarkdown};
use std::fs;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let src = include_str!("../../cdoc-parser/examples/sample.md");
    let code = include_str!("base_funcs.rhai");

    let mut renderer = ElementRenderer::new(code)?;

    let raw = parse_to_doc(src)?;
    let composed = ComposedMarkdown::from(raw.src);
    let mut nodes: Vec<Node> = Vec::from(composed);

    let mut file = File::create("sample_out.xml")?;
    write_elements_to_xml(&nodes, &mut file)?;

    renderer.walk_elements(&mut nodes)?;

    let mut file = File::create("sample_out_eval.xml")?;
    write_elements_to_xml(&nodes, &mut file)?;

    // fs::write("sample_out.html", out)?;

    Ok(())
}
