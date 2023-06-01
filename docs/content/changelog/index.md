---
title: Changelog
layout:
    hide_sidebar: true
---

# Project changelog

## 0.6.x

### 0.6.1
- Properly implemented enumerated and nested lists for markdown output.
- Improved error messages for the pulldown_cmark to internal Ast conversion.
- Changed notebook build format to include resource folder (may change again).
- Fixed parsing problem for shortcode arguments with markdown strings.
- Many other small bug fixes.
- Cleanup of unused code and some general refactoring.

### 0.6.0
- Templates are now specified in .yml files with common metadata. The template sources are either embedded in the .yml file or linked in an external file.
- Templates can be documented through the new syntax. The default template includes an example of how this can be used to produce shortcode documentation.
- Templates now support positional arguments.
- Template arguments are validated. The name and value is checked. Values types are currently either anything or one of a list of values (an enumeration essentially).
- Templates now receive information on shortcodes present in the current document. This makes it possible to create table of content lists with custom shortcodes.
- Rendering and generation pipelines are unified across input and output formats. The notebook input is handled via a separate renderer that wraps the generic renderer internally.
- The system is close to output-format agnostic. Format is now a trait that can be implemented outside the cdoc crate. Perhaps this will be moved to a config file in the future.
- The building process is now parallelized using the rayon library. Additionally, many unnecessary clones have been removed. Rendering now uses the Write trait instead of returning heap strings.

## 0.5.0
This update is mainly a large refactoring of the document processing system. The Element abstraction over the pulldown_cmark 
Event type was fully removed and replaced with the Ast.

- Ast now supports full document specification for all formats.
- Code cells for notebook outputs.
- Shortcode elements for proper nested shortcode and markdown rendering.
- Math elements for eliminating pulldown_cmark math parsing problems.

## 0.4.0

- LaTeX output support
- Nested shortcodes (through Ast representation)

## 0.3.x

### 0.3.1
This update only fixes broken links in the README.md file.


### 0.3.0
This update refactored the whole document processing pipeline to use a custom Ast type that greatly simplifies implementation of custom features.

- Ast type for internal document representation.
  - Former pulldown_cmark Event based extensions migrated to Ast.
  - All renderers migrated to Ast
- Added *editable* and *interactive* fields to document metadata to allow for definition of interactive code cells.

## 0.2.x

### 0.2.1
- Improved error handling with color-coded terminal output and better formatting
- Auto-reload now works for frontmatter as well.
- Various smaller bug fixes.

### 0.2.0
This update refactored the main *courses* crate into two distinct crates, *cdoc* for document processing and *courses* for project configuration and handling.

## 0.1.x

### 0.1.1
Bug fix and refactoring release, no new features.

### 0.1.0
This is the first release on crates.io.

- Static site and notebook generation from markdown and notebook source files.
- Basic project configuration.
- Automatic rebuild upon project changes (entire project).
- Initial shortcode support (cannot be nested, cannot contain certain markdown elements).
- Initial KaTeX support.
- Initial exercise spec support.