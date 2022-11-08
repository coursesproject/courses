use pulldown_cmark::Event;
use std::iter::{FilterMap, Map};

#[derive(Debug)]
pub enum Ast<'a> {
    MdEvent(Event<'a>),
    Output(String),
}

impl<'a> From<Event<'a>> for Ast<'a> {
    fn from(e: Event<'a>) -> Self {
        Ast::MdEvent(e)
    }
}

pub fn events_to_ast<'a, I: Iterator<Item = Event<'a>>>(iter: I) -> Map<I, fn(Event) -> Ast> {
    iter.map(|event| Ast::from(event))
}

pub fn ast_to_events<'a, I: Iterator<Item = Ast<'a>>>(
    iter: I,
) -> FilterMap<I, fn(Ast) -> Option<Event>> {
    iter.filter_map(|a| match a {
        Ast::MdEvent(e) => Some(e),
        Ast::Output(_) => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::CowStr;

    #[test]
    fn test_conversion() {
        let events = vec![
            Event::Html(CowStr::Boxed("Hej".to_string().into_boxed_str())),
            Event::HardBreak,
            Event::Rule,
        ];

        let ast_events = events_to_ast(events.clone().into_iter());
        let cm_events = ast_to_events(ast_events);
        let test_events: Vec<_> = cm_events.collect();

        assert_eq!(
            events, test_events,
            "Test that the converted events are equal to the originals."
        );
    }
}
