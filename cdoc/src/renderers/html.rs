use crate::document::EventDocument;
use crate::renderers::Renderer;
use pulldown_cmark::html;

struct HtmlRenderer {}

impl Renderer for HtmlRenderer {
    fn render(&self, doc: &EventDocument) -> String {
        let iter = doc.to_events();
        let mut output = String::new();
        html::push_html(&mut output, iter);
        output
    }
}
