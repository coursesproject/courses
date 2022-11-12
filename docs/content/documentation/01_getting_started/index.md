---
title: Getting started
---

# Getting started
This page explains the basics of how `courses` can be used. It covers installation, creating a new project, serving the project locally, and setting up a process for deploying the site.

## Installation
1. You need to install rust and cargo to compile the application. Doing so is most easily accomplished by installing *rustup*. Simply follow the instructions on the [installation page](https://rustup.rs/).
2. Install `courses` by running the command `cargo install courses`. 

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