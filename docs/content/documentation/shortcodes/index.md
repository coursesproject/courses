---
title: Shortcodes
---

# Shortcodes

Markdown is intentionally limiting in its functionality (REF). Therefore, like most other static-site generators, 
`courses` supports a special syntax called shortcodes for adding components to your markup. A number of built-in 
shortcodes provide basic features like figures and admonitions. Custom shortcodes can easily be added to any project 
to extend the functionality.

## Syntax
Shortcodes support two syntaxes:

**Inline:** `{{ name(arg1=value, arg2="a string value") }}` 

**Block:**
```
{% name(arg1=value, arg2="a string value") %}

Valid markdown markup

{% end %}
```

The inline variant simply renders the shortcode template with the provided argument values and replaces the 
shortcode with the html output.

The block variant makes it possible to use Markdown content in the template. The markup inside the block delimiters 
is pre-rendered as html and then passed to the shortcode's template in the `body` parameter. *As a result, shortcodes 
typically written using the block syntax can also be written using the inline syntax with the `body` parameter 
specified manually.*

## Built-in codes

### Image

`{{ image(src="my_img.png") }}`

### Admonition

```
{% admon(class=css-color-class) %}
This is some markup that will appear in an admonition box
{% end %}
```

## Custom shortcodes
Each shortcode is defined by a single template file by the same name in a project's `templates/shortcodes` folder. 
As you should verify, the built-in shortcodes are nothing more than templates included by the `courses init` command.

The templates use the Tera templating engine which is easy to use and has excellent documentation ([link]()). The 
syntax is very similar to [Jinja2]() and [Django templates](). 

### Parameters
Parameters are defined implicitly by using them in the template. Courses automatically inserts the values provided 
at the shortcode call-site into the template - the names map one-to-one. For block shortcodes, the body is inserted 
as the variable `body`.

### Other available variables
You can also access other project- and document-related metadata inserted automatically. 

| variable | description |
|----------|-------------|
| config   | Project configuration ([details here]()). |