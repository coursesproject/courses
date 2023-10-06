use crate::renderers::extensions::{RenderExtension, RenderExtensionConfig};
use crate::renderers::newrenderer::ElementRenderer;
use crate::renderers::Document;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{CodeBlock, Command, Inline, Parameter, Value};
use cdoc_parser::document::{CodeOutput, Image, OutputValue};
use cdoc_parser::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
//
