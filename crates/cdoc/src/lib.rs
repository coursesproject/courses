//! CDoc is a tool for rendering markdown-based document formats (markdown and jupyter notebooks) to
//! user-defined output formats. Output formats are entirely user defined through templates.

/// Defines input/output format configuration types.
pub mod config;

/// Defines types for loading content files and parsing them to the internal format. Can be extended.
pub mod loader;

/// Provides a type for applying preprocessors to documents.
pub mod parser;

/// Contains preprocessors for documents. Currently, only used for the exercise syntax.
pub mod preprocessors;

/// Contains renderer types that can be used for raw outputs and notebook-based outputs. Can be extended.
pub mod renderers;

pub mod package;

/// Provides a template manager for easily rendering the different types of templates supported by cdoc.
pub mod templates;
