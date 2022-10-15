// use crate::config::{Document, Format};
// use crate::notebook::Notebook;
// use pulldown_cmark::{Event, Options, Parser};
// use std::fs;
// use std::fs::File;
// use std::io::BufReader;
//
// use anyhow::Result;
//
// pub fn parse_pd(doc: Document) -> Result<Box<dyn Iterator<Item = Event<'static>>>> {
//     let mut options = Options::empty();
//
//     Ok(match doc.format {
//         Format::Notebook => {
//             let bf = BufReader::new(File::open(doc.path)?);
//             let nb: Notebook = serde_json::from_reader(bf)?;
//             Box::new(nb.into_iter())
//         }
//         Format::Markdown => {
//             let input = fs::read_to_string(doc.path)?;
//             let parser = Parser::new_ext(&input, options);
//             Box::new(parser)
//         }
//     })
// }
