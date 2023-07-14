//! CDoc is a tool for rendering markdown-based document formats (markdown and jupyter notebooks) to
//! user-defined output formats. Output formats are entirely user defined through templates.

/// Defines the internal markup format.
pub mod ast;

/// Defines input/output format configuration types.
pub mod config;

/// Defines the document format which combines metadata with content.
pub mod document;

/// Defines types for loading content files and parsing them to the internal format. Can be extended.
pub mod loader;

/// Defines types for serializing/deserializing Jupyter Notebooks.
pub mod notebook;

/// Provides a type for applying preprocessors to documents.
pub mod parser;

/// Contains pest.rs based parsers for various functions.
pub mod parsers;

/// Contains preprocessors for documents. Currently, only used for the exercise syntax.
pub mod processors;

/// Contains renderer types that can be used for raw outputs and notebook-based outputs. Can be extended.
pub mod renderers;

/// Provides a template manager for easily rendering the different types of templates supported by cdoc.
pub mod templates;
