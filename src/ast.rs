use std::collections::{HashMap, LinkedList};
use pulldown_cmark::{Event, Tag};
// use crate::ast::Block::Blank;
//
// pub struct Document {
//     elements: Vec<Block>
// }
//
// type Attributes = HashMap<String, String>;
//
// pub enum Block {
//     Code(Attributes, String),
//     Heading(u64, Attributes, String),
//     Html(String),
//     ShortcodeInline(String, HashMap<String, String>),
//     ShortcodeBlock(String, HashMap<String, String>, Vec<Block>),
//     Paragraph(Vec<Inline>),
//     List(Option<u64>, Vec<Block>),
//     Blank
// }
//
// pub enum Inline {
//     Emphasis(String),
//     Strong(String),
//     Link(String, String),
//     Image(String, String),
//     Text(String),
//     Code(String),
//     SoftBreak,
//     HardBreak,
//     Rule
// }
//
// impl Document {
//     fn get_next_block<I: IntoIterator<Item=Event>>(iter: I) -> (I, Block) {
//
//     }
//
//     pub fn from_events<I: IntoIterator<Item=Event>>(iter: I) -> Self {
//         let mut elems = Vec::new();
//         let mut tags = LinkedList::new();
//         let mut current_elem = Blank;
//         for e in iter {
//             match e {
//                 Event::Start(tag) => {
//                     tags.push_front(tag);
//                     match tag {
//                         Tag::Paragraph => {}
//                         Tag::Heading(_, _, _) => {}
//                         Tag::BlockQuote => {}
//                         Tag::CodeBlock(_) => {}
//                         Tag::List(_) => {}
//                         Tag::Item => {}
//                         Tag::FootnoteDefinition(_) => {}
//                         Tag::Table(_) => {}
//                         Tag::TableHead => {}
//                         Tag::TableRow => {}
//                         Tag::TableCell => {}
//                         Tag::Emphasis => {}
//                         Tag::Strong => {}
//                         Tag::Strikethrough => {}
//                         Tag::Link(_, _, _) => {}
//                         Tag::Image(_, _, _) => {}
//                     }
//                 },
//                 Event::End(_) => (),
//                 Event::Text(txt) => {}
//                 Event::Code(_) => {}
//                 Event::Html(_) => {}
//                 Event::FootnoteReference(_) => {}
//                 Event::SoftBreak => {}
//                 Event::HardBreak => {}
//                 Event::Rule => {}
//                 Event::TaskListMarker(_) => {}
//             }
//         }
//
//         Document {
//             elements: elems
//         }
//     }
// }