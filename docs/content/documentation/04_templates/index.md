---
title: Templates
---

# Templates

In courses, the layout and style of the output is fully customizable. Everything from the page and menus to individual 
markdown elements are rendered using [Tera](https://tera.netlify.app/) templates.  

## Template folder structure

The template folder has to contain at least the following files:

```text
- templates/
    - builtins/
        - <format>/
            - cell.tera.<format> (code cell)
            - image.tera.<format> (markdown image)
            - link.tera.<format> (markdown link)
            - list_item.tera.<format> (markdown list item)
            - list_ordered.tera.<format> (markdown unordered list)
            - list_unordered.tera.<format> (markdown ordered list)
            - output_error.tera.<format> (notebook output)
            - output_text.tera.<format> (notebook output)
            - output_img.tera.<format> (notebook output - base64 image data)
            - output_svg.tera.<format> (notebook output - svg data)
    - shortcodes/
        ...
    - section.tera.<format>
    ...
    
```

where `<format>` is a placeholder for a given output format (e.g. html). The templates in the `builtins` folder need to 
be present for courses to be able to render the corresponding document elements. The `section.tera.<format>` file is 
used to render each document. 

## Template layouts
The `section` template is responsible for constructing the output for a single document and exists for all output 
formats. This includes navigation and references. Therefore, the template receives a comprehensive set of values 
containing information on the project's structure and content.

The top-level values are:
```text
- project
- config
- current_part
- current_chapter
- current_doc
- doc
```

{% message(title="Tip", color="warning") %}
The default project template includes a section template with navigation. It is likely much easier to use it as a starting
point for custom layouts instead of starting from scratch.
{% end %}

### Project object
The project object (simply named `project`) contains the following elements:

```text
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


## Builtins
The builtin templates are used by the rendering process to render specific document elements. The default template 
provides a neutral starting point for `html` and `md` outputs.



