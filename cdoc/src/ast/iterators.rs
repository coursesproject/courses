use crate::ast::{math_block_md, AEvent, ATag, Ast, Block, Inline};

fn iter_inlines(inlines: &[Inline]) -> Vec<AEvent> {
    inlines.iter().flat_map(|i| i.clone().into_iter()).collect()
}

fn iter_blocks(blocks: &[Block]) -> Vec<AEvent> {
    blocks
        .iter()
        .flat_map(|block| block.clone().into_iter())
        .collect()
}

fn wrap_events(tag: ATag, mut events: Vec<AEvent>) -> std::vec::IntoIter<AEvent> {
    let mut res = vec![AEvent::Start(tag.clone())];
    res.append(&mut events);
    res.append(&mut vec![AEvent::End(tag)]);
    res.into_iter()
}

impl IntoIterator for Inline {
    type Item = AEvent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Inline::Text(s) => vec![AEvent::Text(s)].into_iter(),
            Inline::Emphasis(inner) => wrap_events(ATag::Emphasis, iter_inlines(&inner)),
            Inline::Strong(inner) => wrap_events(ATag::Strong, iter_inlines(&inner)),
            Inline::Strikethrough(inner) => wrap_events(ATag::Strikethrough, iter_inlines(&inner)),
            Inline::Code(s) => vec![AEvent::Code(s)].into_iter(),
            Inline::SoftBreak => vec![AEvent::SoftBreak].into_iter(),
            Inline::HardBreak => vec![AEvent::HardBreak].into_iter(),
            Inline::Rule => vec![AEvent::Rule].into_iter(),
            Inline::Html(s) => vec![AEvent::Html(s)].into_iter(),
            Inline::Image(tp, url, alt, inner) => {
                wrap_events(ATag::Image(tp, url, alt), iter_inlines(&inner))
            }
            Inline::Link(tp, url, alt, inner) => {
                wrap_events(ATag::Link(tp, url, alt), iter_inlines(&inner))
            }
            Inline::Math(s) => vec![AEvent::Text(s)].into_iter(),
        }
    }
}

impl IntoIterator for Block {
    type Item = AEvent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // id.map(|s| s.as_str()) <> classes.into_iter().map(|s| s.as_str()).collect()
        match self {
            Block::Heading { lvl, inner, .. } => {
                wrap_events(ATag::Heading(lvl, None, vec![]), iter_inlines(&inner))
            }
            Block::Paragraph(inner) => wrap_events(ATag::Paragraph, iter_inlines(&inner)),
            Block::Plain(inline) => inline.into_iter(),
            Block::BlockQuote(inner) => wrap_events(ATag::BlockQuote, iter_inlines(&inner)),
            Block::CodeBlock { source, .. } => {
                let string = format!("<pre><code>{}</code></pre>", source);
                vec![AEvent::Html(string)].into_iter()
            }
            Block::List(idx, items) => {
                let item_events = items.into_iter().flat_map(|inner| inner.into_iter());
                let full_iter = vec![AEvent::Start(ATag::List(idx))]
                    .into_iter()
                    .chain(item_events)
                    .chain(vec![AEvent::End(ATag::List(idx))]);
                let v: Vec<AEvent> = full_iter.collect();
                v.into_iter()
                // TODO: Change iter type to dynamic
            }
            Block::ListItem(inner) => wrap_events(ATag::Item, iter_blocks(&inner)),
            // Block::Html(s) => vec![AEvent::Html(s.into_boxed_str().to_string())].into_iter(),
            Block::Math(s, display_block, trailing_space) => {
                let s = math_block_md(&s, display_block, trailing_space);
                vec![AEvent::Text(s)].into_iter()
            }
            Block::Shortcode(s) => vec![].into_iter(), // unsupported
        }
    }
}

impl IntoIterator for Ast {
    type Item = AEvent;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter().flat_map(|block| block.into_iter()))
    }
}

pub(crate) enum InnerContent {
    Blocks(Vec<Block>),
    Inlines(Vec<Inline>),
}

impl InnerContent {
    pub(crate) fn into_blocks(self) -> Vec<Block> {
        if let InnerContent::Blocks(b) = self {
            b
        } else {
            panic!("Expected blocks")
        }
    }

    pub(crate) fn into_inlines(self) -> Vec<Inline> {
        if let InnerContent::Inlines(i) = self {
            i
        } else {
            panic!("Expected inlines")
        }
    }

    pub(crate) fn blocks_mut(&mut self) -> &mut Vec<Block> {
        if let InnerContent::Blocks(b) = self {
            b
        } else {
            panic!("Expected blocks")
        }
    }

    #[allow(unused)]
    fn inlines_mut(&mut self) -> &mut Vec<Inline> {
        if let InnerContent::Inlines(i) = self {
            i
        } else {
            panic!("Expected inlines")
        }
    }

    pub(crate) fn push_inline(&mut self, item: Inline) {
        match self {
            InnerContent::Blocks(b) => b.push(Block::Plain(item)),
            InnerContent::Inlines(i) => i.push(item),
        }
    }
}
