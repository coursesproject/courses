use anyhow;
use cdoc_base::document::Document;
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::Node;
use extism_pdk::*;

#[plugin_fn]
pub fn name(_input: ()) -> FnResult<Json<String>> {
    Ok(Json("my_ext".to_string()))
}

#[plugin_fn]
pub fn process(mut input: Json<Document<Vec<Node>>>) -> FnResult<Json<Document<Vec<Node>>>> {
    let mut v = Visit;
    v.walk_elements(&mut input.0.content)?;
    Ok(input)
}

pub struct Visit;

impl NodeVisitor for Visit {
    fn visit_plain(&mut self, text: &mut String) -> anyhow::Result<()> {
        *text = "hej".to_string();
        Ok(())
    }
}
