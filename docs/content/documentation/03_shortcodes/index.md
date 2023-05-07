---
title: Shortcodes
---

# Shortcodes

Markdown is intentionally limiting in its functionality. Therefore, like most other static-site generators,
`courses` supports a special syntax - called *shortcodes* - for adding components to your markup. A number of built-in
shortcodes provide basic features like figures and admonitions. Custom shortcodes can easily be added to any project
to extend the functionality.

Courses supports multiple output formats for shortcodes depending on context. This makes it easy to make shortcodes compatible with both html-output and the markdown output in notebooks. 

## Syntax

Shortcodes support two syntaxes:

**Inline:** `{{ name(arg1=value1, arg2="a string value") }}`

**Block:**

```html
{% name(arg1=value, arg2="a string value") %}

Valid markdown markup

{% end %}
```

The inline variant simply renders the shortcode template with the provided argument values and replaces the shortcode
with the html or markdown output.

The block variant makes it possible to use Markdown content in the template. The markup inside the block delimiters
is pre-rendered as html and then passed to the shortcode's template in the `body` parameter. *As a result, shortcodes
typically written using the block syntax can also be written using the inline syntax with the `body` parameter
specified manually.*

{% message#hej(color=warning, title="Tip") %}

Shortcodes can be expanded over multiple lines to improve readability. For example:

```plain
{{ name(
    arg1=value1, 
    arg2="a string value"
) }}
```

The syntax does not use significant whitespace. It therefore does not matter how you choose to indent each line or even
how many linebreaks there is between arguments.

{% end %}

## Codes in the default template

{{ shortcode_docs }}

## Custom shortcodes

Each shortcode is defined by a single template file by the same name in a project's `templates/shortcodes` folder. The
default codes described above are included when using the default `courses init` command to create a project.

The templates use the Tera templating engine which is easy to use and has
excellent [documentation](https://tera.netlify.app/).

### Parameters

Parameters are defined implicitly by using them in the template. Courses automatically inserts the values provided at
the shortcode call-site into the template - the names map one-to-one. For block shortcodes, the body is inserted as the
variable `body`.

Shortcode arguments are mandatory by default. If a value is used in a template without being defined at the call-site,
Courses returns an error. Optional arguments can be implemented using the Tera `default` function,
e.g. `{{ value | default(2) }}`.

### Other available variables

Courses additionally inserts a number of project and document related variables which can be used by the templates.
Below is an overview of the included elements so far:

| variable | description                               |
|----------|-------------------------------------------|
| project  | Project configuration ([details here]()). |