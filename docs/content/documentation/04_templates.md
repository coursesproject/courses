---
title: Templates
---

# Templates

In courses, the layout and style of the output is fully customizable. Everything from the page and menus to individual 
markdown elements are rendered using [Tera](https://tera.netlify.app/) templates.

The templates are defined in *YAML* files with the template source for each output format being provided either directly 
as strings or as file links. This makes organisation a lot easier, especially since there are a lot of templates. 

Courses contains three kinds of templates: *builtins*, *shortcodes*, and *layouts*. The type determines in what 
situations the template may be used and what kind of metadata it might contain. Shortcodes are by far the most complex 
and are covered in detail in its own section. Builtins specifies templates for individual document elements such as 
code cells or headings. Layouts are currently only used to define a single parent layout used for all pages as the base 
template (only for formats where it is relevant).

## Template folder structure

The template folder should have the following structure and base files:

```yaml, cell
- templates/
    - builtins/
        - cell.yml (code cell)
        - emphasis.yml 
        - hard_break.yml
        - header.yml
        - horizontal_rule.yml
        - image.yml
        - inline_code.yml
        - image.yml (markdown image)
        - link.yml (markdown link)
        - list_item.yml (markdown list item)
        - list_ordered.yml (markdown unordered list)
        - list_unordered.yml (markdown ordered list)
        - math.yml
        - output_error.yml (notebook output)
        - output_text.yml (notebook output)
        - output_img.yml (notebook output - base64 image data)
        - output_svg.yml (notebook output - svg data)
        - paragraph.yml
        - soft_break.yml
        - strong.yml
    - shortcodes/
        ...
    - sections/
        - section.yml
    - sources
        - <linked template files>
    
```

Check the [code for the default template](https://github.com/coursesproject/courses-template-default/tree/main/templates) 
for an example implementation of the builtin templates and layouts. 

## Template structure
The `yml` file for each template follows a common structure as shown below:

```yaml
name: name
description: description

type: [builtin/shortcode/layout]

examples:
  - title: title
    body: [markdown body]


templates:
  html: !String |
    [body]
  markdown: !String |
    [body]
  latex: !File [path]
```

The following elements must always be present for the template to be valid:
- **Name:** A descriptive name for the template. This is mostly useful for generating documentation.
- **Description:** A description of what the template does.
- **Type:** This field must be one of: *builtin*, *layout*, or *shortcode*. 
- **Examples:** Contains a list of examples, each with a title, body, and an optional description. Used for 
  documentation.
- **Templates:** Contains a map consisting of a template for each output format. The format names must match the 
    definition in the `config.yml` file.

### Template sources
Templates can be provided as either a string literal or as a file reference. String literals must be preceded by 
`!String` and references by `!File`. This notation is derived from how rust deserializes the files.

## Builtins
Builtins are the simplest templates and usually only have access to a few values determined by the element it 
represents. 

## Layouts
The `section.yml` template is responsible for constructing the output for a single document and is used for *html* and 
*LaTeX* outputs. For web pages, the template should include any menu's, navigation, and general page setup necessary. 
For LaTeX output, the template should contain the document preamble and any custom commands. Again, the 
[default template](https://github.com/coursesproject/courses-template-default/tree/main/templates) is an excellent 
source for learning more about how these can be set up.

### Variables and metadata
Layouts have access to information on the project structure as well as the individual document that is being processed.

The top-level values are:
```yaml
- config
- ids
- id_map
- doc_meta
```

### Project object
The project object (simply named `project`) contains the following elements:

```yaml
- project_path
- index (page)
- content
    - part1
      - id
      - index (page)
      - chapters
        - chapter1
          - id
          - index (page)
          - documents
            - doc1 (page)
            - ...
          - files (included files)
        - ...
    - ...
```

Each of the elements marked with `page` contain information on a single document. The structure of the `content` element
mirrors that of the files and folders in the `content` project folder.

#### Page object
Represents a rendered document:


```text
- id
- path
- content (rendered output)
```


### Config object
The config object contains the elements of the `config.yml` project file:

```text
- url_prefix
- repository
    - url
- outputs
    - <list of output formats>
- parsers
- custom (custom configuration values)
```

The `url_prefix` is important for formatting urls correctly, for example when using a service like GitHub pages. The 
`custom` object can contain any valid `yaml` and is used to add custom configuration to the system.


### Current (part/chapter/doc)
These three elements (`current_part`, `current_chapter`, and `current_doc`) are the id's of the current document's path. These are most useful for creating navigation where 
the current document is highlighted.

### Current document
The `doc` value contains the page object for the current document. To render the page, simply do `{{ doc.content | safe}}`.




