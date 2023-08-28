---
title: Exercise definitions
exercises: false
---

# Code tools
Courses allows you to tag source code blocks using a special syntax hiding in the comments of the host language (so far only Python is supported, but this will change soon). So far, the placeholder/solution syntax is the only fully implemented function and it makes it possible to define a single source for documents that contain elements that have to be hidden from the recipients. 

*This concept is planned to be expanded to support automatic testing of solutions and grading of user submitted code.*


## Exercise definitions

hello
```
#| code <<
print("solution")
#| placeholder
# print("hello")
#| >>
```