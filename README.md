# Courses

Courses is a publishing system for interactive content written in `rust`. Its aimed primarily at code-related learning 
resources, including books, course materials, and blogs.

Features:
- Supports interactive source files (Jupyter Notebooks) as well as regular markup(Markdown).
- Define tasks and their solutions directly in source files. Courses is made specifically with learning resources in mind.
- Uses a simple template-like syntax (shortcodes) for applying built-in and custom components, including figures and admonitions.

## The purpose

Courses was created to solve a specific set of problems not covered by similar tools like Jupyterbook and Quarto. 
These tools are based on large ecosystems for performing the publishing process. In contrast, `courses` is 
light-weight, self-contained and is very easy to extend in fundamental ways from both a developer and user perspective.

## An integrated format for defining exercises. 
Most modern learning materials use exercises or tasks in some capacity. For technical materials involving code, 
keeping track of the solutions and any student provided code becomes increasingly complex and error-prone as a project 
grows in size. The naive approach of keeping a separate solution version is problematic because it needs to also 
copy the exercise definition. Any changes made to one version must therefore manually be copied to the other. 
Additionally, there's no easy way to test the published version (i.e. without the solution).

In courses, the exercise definitions and solutions (in content files or code files) can be defined in the same source 
file using a simple syntax to 
specify sections as e.g. the solution. Instead of having to manually keep track of changes, the tool uses the 
complete definition to test the solution and to build the output with a placeholder. Creating new exercises using 
this method is easy. You start from a solution and simply mark of the parts that should not be visible in the published output.

### More features
There are 

## Speed is a feature
Courses is fast enough to make a real-time editing workflow possible. The built-in automatic rebuild and reload 
mechanism makes any change to the source files appear almost instantaneously in the browser. Even full builds 
typically take less than a second to complete, even for large projects.

You should be able to focus on writing content. A faster tool gets out of your way and gives you feedback as you 
make changes.


## Similar projects
Courses is far from the only tool for publishing learning resources and scientific material to the web. `Courses` has 
a very distinct approach and a distinct set of goals. The following comparison might be helpful if you need help to 
decide between the tools.

### Jupyterbook
This project focuses highly on tight integration with the Jupyter Notebook ecosystem and formats. It is especially 
great for allowing interaction with notebooks using existing technologies like JupyterHub. It uses sphinx as a 
back-end which may be a positive or negative depending on your requirements. The custom markdown syntax it supports 
(`myst-markdown`) has many convenient features but is incompatible with regular Jupyter Notebook editors (like 
VSCode and JupyterLab). Because the entire project is built in Python, building is slow. Additionally, there's no 
convenient automatic reload function.

### Quarto

### Static-site generators (e.g. Hugo, Jekyll)
These tools are very generic and allow for customization of almost every aspect of the published webpage. However, 
they don't support the Jupyter Notebook format directly. It is of course possible to use a conversion tool like 
`jupytext` to generate Markdown from your `.ipynb` files. 