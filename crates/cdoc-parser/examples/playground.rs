use cdoc_base::node::xml_writer::write_elements_to_xml;
use cdoc_base::node::Element;
use cdoc_parser::raw::{parse_to_doc, ComposedMarkdown};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let input = include_str!("sample.md");

    let raw = parse_to_doc(input)?;
    let composed = ComposedMarkdown::from(raw.src);
    let nodes: Vec<Element> = Vec::from(composed);

    // if let Element::Node(node) = &nodes[0] {
    //     println!("node {}", node.type_id);
    // }

    println!("writing output");
    let mut file = File::create("sample_out.xml")?;
    write_elements_to_xml(&nodes, &mut file)?;

    Ok(())
}
