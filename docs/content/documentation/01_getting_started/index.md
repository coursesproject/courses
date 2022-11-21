---
title: Getting started
---

# Getting started
This page explains the basics of how `courses` can be used. It covers installation, creating a new project, serving the project locally, and setting up a process for deploying the site.

## Installation
 For now however, you still have to compile Courses yourself. 

1. You need to install rust and cargo to compile the application. Doing so is most easily accomplished by installing *rustup*. Simply follow the instructions on the [installation page](https://rustup.rs/).
2. Install `courses` by running the command `cargo install courses`.

### What about binaries
It is our goal to eventually provide pre-compiled binaries for each of the three large operating systems (Windows, 
Mac OS, and Linux) on their typical platforms. In practice, this does alleviate the need to download and use various 
build-tools but it also means we loose the straightforward process for updating provided by `cargo`. The solution is 
to use platform-specific package managers but this requires both setup and additional testing whenever an update is 
pushed. We simply choose to use our limited resources on developing the main application for now. While it might 
deter some use of the platform, this might actually be beneficial since Courses is still in early development.  

## Basic usage
Get an overview 

```
courses -h
```

```text
Usage: courses [OPTIONS] <COMMAND>

Commands:
  serve    
  build    
  test     
  publish  
  help     Print this message or the help of the given subcommand(s)

Options:
  -p, --path <PATH>  
  -h, --help         Print help information
  -V, --version      Print version information
```


```bash
courses build
```

## Creating a new project

```
courses init 
```