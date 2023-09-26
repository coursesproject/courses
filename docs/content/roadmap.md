---
title: Roadmap
layout:
    hide_sidebar: true
---

# Roadmap

## 0.8.x
This version focuses on the experience of writing and testing code.

<div class="columns is-multiline banner">


#card(Improved code split syntax, color=primary) {
Simplify and improve the code split syntax by making placeholders optional and by removing unnecessary elements.
}

{% card(title="Improved code split syntax", color=primary) %}
Simplify and improve the code split syntax by making placeholders optional and by removing unnecessary elements.
{% end %}

{% card(title="Code validation", color=primary) %}
Courses should support running in-document code validation/tests. This will likely be implemented as langauge-specific extensions.
{% end %}



</div>

## 0.9.x
This version will focus on decoupling the templates from individual projects and provide a system for easily extending 
Courses functionality.

<div class="columns is-multiline banner">

{% card(title="Improved themes/templates", color=danger) %}
Themes and templates should be decoupled from each individual project, similar to Hugo's implementation.
{% end %}

{% card(title="Extension system", color=primary) %}
Script-based extension system. Will likely use Python or Rhai.
{% end %}

</div>

## Future
The rest of the items are not prioritized and may change significantly.

<div class="columns is-multiline banner">

{% card(title="Interactive code cells", color=warning) %}
It is currently possible to make cells interactive by implementing the logic client-side. However, it should also be 
possible to use some kind of server-side technology for running code.
{% end %}

{% card(title="Multiple language support", color=primary) %}
Some of Courses core features are currently tied to the Python language. This should be made agnostic.
{% end %}

{% card(title="Cell output formatting", color=danger) %}
Similar to the Quarto implementation
{% end %}

{% card(title="Cell visibility/inclusion controls", color=warning) %}
Make cells easy to hide or to remove completely from the output.
{% end %}

{% card(title="Interactive cell outputs", color=warning) %}
Somehow make Courses work with Jupyter widgets.
{% end %}


</div>