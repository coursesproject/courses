use katex::Opts;
use crate::extensions::Preprocessor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KaTeXPreprocessorError {}

pub struct KaTeXPreprocessor {
    opts: Opts,
}

impl KaTeXPreprocessor {
    pub fn new(opts: Opts) -> Self {
        KaTeXPreprocessor {
            opts
        }
    }
}

fn find_block(input: &str) -> Option<(usize, usize, usize)> {
    let begin = input.find("$")?;
    let end_delim = if &input[(begin + 1)..(begin + 2)] == "$" { "$$" } else { "$" };

    let end = begin + end_delim.len() + input[begin + end_delim.len()..].find(end_delim)?;

    Some((begin, end, end_delim.len()))
}

impl Preprocessor<katex::Error> for KaTeXPreprocessor {
    fn process(&self, input: &str) -> Result<String, katex::Error> {
        let mut rest = input;
        let mut res = String::new();

        while rest.len() > 0 {
            match find_block(rest) {
                Some((begin, end, delim_len)) => {
                    let pre = &rest[..begin];
                    let post = &rest[(end + delim_len)..];

                    let source = &rest[(begin + delim_len)..end];

                    let mut opts = self.opts.clone();
                    opts.set_display_mode(delim_len == 2);
                    let ktex = katex::render_with_opts(&source, opts)?;

                    res.push_str(pre);
                    res.push_str(&ktex);

                    rest = post;
                }
                None => {
                    res.push_str(rest);
                    rest = ""
                }
            }
        }

        Ok(res)
    }
}