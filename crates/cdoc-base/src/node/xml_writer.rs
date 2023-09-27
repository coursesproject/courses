use crate::node::{Attribute, Document, Element};
use std::io;
use std::io::Write;
use xml::writer::Result;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

impl Document {
    pub fn write_xml(&self, writer: impl Write) -> Result<()> {
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(writer);

        for element in &self.content {
            element.write_xml(&mut writer)?;
        }

        Ok(())
    }
}

pub fn write_elements_to_xml(elements: &Vec<Element>, writer: impl Write) -> Result<()> {
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(writer);

    writer
        .write(XmlEvent::start_element("document").ns("parameter", "document.command.parameter"))?;

    for element in elements {
        element.write_xml(&mut writer)?;
    }

    writer.write(XmlEvent::end_element())?;

    Ok(())
}

impl Attribute {
    pub fn to_string(&self) -> String {
        match self {
            Attribute::Int(n) => n.to_string(),
            Attribute::Float(n) => n.to_string(),
            Attribute::String(s) => s.clone(),
            Attribute::Enum(v) => v.clone(),
            Attribute::Flag => "".to_string(),
        }
    }
}

impl Element {
    pub fn write_xml<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<()> {
        match self {
            Element::Plain(s) => {
                writer.write(XmlEvent::start_element("text"))?;
                writer.write(XmlEvent::characters(s.as_str()))?;
                writer.write(XmlEvent::end_element())
            }
            Element::Node(node) => {
                let mut start = XmlEvent::start_element(node.type_id.as_str());
                let attr: Vec<(String, String)> = node
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();
                for (k, v) in &attr {
                    start = start.attr(k.as_str(), &v);
                }
                writer.write(start)?;
                if let Some(children) = &node.children {
                    for c in children {
                        c.write_xml(writer)?;
                    }
                }
                writer.write(XmlEvent::end_element())
            }
        }
    }
}
