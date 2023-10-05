---
title: Project organisation and configuration
---

# Project organisation and configuration

All Courses projects have the following elements:
- `build/` contains the project outputs (webpage, notebooks, etc.)
- `resources/` contains all static resources used in the templates or content.
- `content/` contains all source files for generating content, whether they are documents or scripts.
- `templates/` contains layout templates as well as shortcode templates.
- `config.yml` is the global project configuration. This is the only explicit configuration file.

Once the site layouts and shortcodes have been completed, the `content/` folder is where most further customization happens. The organization of `content/` directly determines the organisation of the final webpage and other outputs.

Here's an example of what a project's directory may look like:
```plain
- config.yml
- build/
- resources/
- content/
    - part-a/
        - index.md
        - chapter1/
            - index.md
            - section-01.md
        - chapter2/
            - index.ipynb
            - section-01.ipynb
            - section-02.md
            - myscript.py
    - part-b/
        - index.md
        - chapter1/
            - index.md
        - chapter2/
            - index.md
- templates/
    - index.tera.html
    - section.tera.html
    - shortcodes/
        - html/
            - image.tera.html
        - md/
            - image.md.html
```

## Content organisation

Courses projects are currently limited to four levels of documents: *the project*, *parts*, *chapters*, and *sections* (this may change in the future). Each level has a corresponding document. In the case of parts, chapters, or an entire project, these documents are always named `index` (and then either the `.md` or `.ipynb` extension) inside the corresponding level folder. Since *sections* do not have children, they are placed on the same level as chapter documents but with arbitrary names. The above example have folders named after their respective levels to exemplify how this works in practice.

#message(title=Note, color=info){
The name `index` is used because these documents are often used as overview pages for the next document level. 
}


## Configuring content
Courses has only as single global configuration file, `config.yml`, that only contains globally relevant information. Content configuration is instead specified in the individual content files using the `yaml` language. In markdown  files, this is done using the *frontmatter syntax*. Example:

```plain
---
this: is
yaml:
    - item
---

# Regular markdown
Some text...
```

In notebooks (`.ipynb` files) it is done by adding a `raw` cell to the very top of the document with the `yaml`-configuration inside.

### Configuration options 
Document configurations consist of a number of possible fields, most of which have default values. This means you can usually leave out most options. The full set of options currently are:
```yaml
title: # String (required)
draft: # boolean
exercises: # boolean
code_solutions: # boolean
cell_outputs: # boolean
interactive: # boolean
editable: # boolean
layout:
  hide_sidebar: true # boolean
exclude_outputs: # list
```
with only the `title` being required.

- `draft`: Only show this page in draft mode.
- `exercises`: Enable/disable parsing of the exercise placeholder/solution syntax in the document. This option is only useful for showing the actual syntax instead of parsing it, as is done on the page for its documentation.
- `code_solutions`: Include parsed solutions in output
- `cell_outputs`: Toggle the notebook cell outputs for the whole document. It is useful for exercise-like documents with outputs created during testing that should not be included in the outputs.
- `interactive`: Used for interactive pages using Pyodide. Note that Pyodide has to be set up for this to work.
- `editable`: Used in conjunction with the interactive flag to make a cell editable.
- `layout`: Options for changing the webpage layout. Currently only supports hiding the sidebar.
- `exclude_outputs`: Disable output generation for listed formats.

## Global configuration
The `config.yml` is used for changing _settings related to the project as a whole. See the [default template](https://github.com/coursesproject/courses-template-default) 
for an example.

The configuration file includes the following elements:
- `url_prefix` (optional): Use this if urls need a prefix, i.e. if the site is not hosted at the root of a domain.
- `repository` (optional): Path to the site's repository.
- `profiles` (optional): A list of build profiles. If left empty, default *release* and *draft* profiles are created.
- `scripts` (optional): Define scripts similar to how *npm* works. 
- `notebook_meta`: Metadata that is copied into every notebook output.


### Profiles
Profiles make it possible to create multiple sets of build _settings for creating different outputs. For example, the 
default *draft* profile includes documents marked as *drafts* while the *release* profile does not. 

A profile consists of the following elements:
- `mode` (optional): Must be either *draft* or *release*. Default is *draft*.
- `parser` (optional): Parser configuration, see details below.
- `formats`: Define output formats.

Here's an example of a customized *release* profile:
```yml
release: # Name of the profile
  mode: release
  parser: # Parser _settings
    preprocessors:
      - exercises # The default preprocessor
    solutions: false # Don't include exercise solutions
  formats:
    - html: {} # Output html. Uses the built-in html format
    - dynamic: # Create a dynamic format
        name: jupyterlab
        extension: ipynb # Output file extension
        template_prefix: nb_formatted # Template prefix to read from
        renderer: # Renderer.
          notebook: # Must either be "generic" or "notebook"
```

#### Output formats
The format specification is quite complex to allow for customisation. The following regular formats can be added like 
`html` in the example above:

- `html`
- `notebook`
- `md`
- `tex`

Each of these formats specify which kind of template to use, which output file extension to use, and whether to use the 
regular renderer or the notebook renderer. Custom (dynamic) formats specify each of these manually as shown in the 
example.


## Build process and outputs
When you build a courses project, the tool generates a webpage as well as a directory of processed notebooks and other source files. This makes using Courses for course content very easy, since the generated notebooks are optimized for distribution. The notebooks are subjected to the same processing pipeline which parses the placeholder/solution syntax and renders shortcode templates. The only difference is that the output are `.ipynb` files instead of web-pages.

### Web process
The generated web-pages are rendered using the layout files in `templates/`. The result is a folder `build/web/` which contains everything necessary for deploying the site, including the content of the `resources/` folder. You can therefore upload the output directly to any static-site host provider such as GitHub Pages or Amazon S3. 

### Notebook process
Notebooks are generated by applying the placeholder/solution syntax to all code cells and then rendering shortcodes using the markdown templates (the ones in `templates/shortcodes/`). Having separate templates for `html` and `markdown` outputs makes it easy to write documents with complex elements such as *images* and *admonitions* on the webpage without ending up with a notebook filled with `html`. 


### Other files
It is often useful to include additional code files or data files for use in the actual content. Courses therefore copies all files not ending in `.md` or `.ipynb` directly from the *content* folder to the `build/source` output folder.


