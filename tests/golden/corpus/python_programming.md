---
tags:
  - languages:python
  - concepts:programming
  - practices:scripting
  - practices:automation
  - concepts:oop
level: beginner
category: language
duration_hours: 40
audience:
  - audiences:developers
  - audiences:sysadmins
  - audiences:data-scientists
  - audiences:devops
---

<!-- course: python_programming -->
<!-- Track gap: Python automation with Selenium (basic Selenium integration example) -->
# `Python` Programming

## Description
`Python` is an easy-to-use interpreted language that has steadily gained in
popularity over the last few years in a wide spectrum of applications,
ranging from `AI` to `Web Services`.
`Python` is also powerful, portable, object-oriented open source programming language
for writing stand alone programs, quick scripts, and prototypes for large applications.
This course provides an in-depth and hands-on introduction to the `Python`
programming language, as well as the most common `Python` application domains and tools.

This course deals with `Python` version 3. `Python` version 2 is not discussed.

## Duration
40 hours / 5 days

## Intended Audience
* Developers who need a flexible and multi purpose scripting language at their disposal
* System administrators who need the powerful scripting capabilities of `Python`.
* `DevOps` people who need `Python` for their day to day operations.
* Developers who plan to use `Python` as their primary programming language to build large scale systems.
* Data scientists and `Machine Learning` experts who wish to enhance their toolkits with `Python`.

## Prerequisites
* Prior programming experience in another language such as Go, `Java`, `C#` or C/`C++` is an advantage.

## Objectives
* Understand how `Python` works behind the scenes
* Write short scripts for automation and testing
* Get the basic skills to start writing larger `Python` systems
* Could automate their work where-ever possible using `Python`
* Could use `Python` for testing, web testing, building and deploying software
* Will have the basic knowledge to start developing large systems using `Python`

## Exercises
* Done with `PyCharm` community on either `Windows` or `Linux` as the students need.

## Outline
<!-- chapter: introduction-to-python, duration: 2h -->
* Introduction to `Python` (1 hour)
    * History of `Python`
    * Why `Python`?
    * `Python` is an `OO` language
    * `Python` is a dynamic language (what does that mean?)
    * Basic `OO` principles (for those who need them)
    * Comparison of `Python` with other programming languages in terms of performance,
      garbage collection, multi-`threading`, learning curve and more
<!-- chapter: your-first-python-program, duration: 2h -->
* Your first `Python` program (1 hour)
    * How to install the `Python` programming environment
        * Difference versions of `Python` and which to choose
        * Sometimes you don't get to choose
    * Using `python` interactively
    * Your first script
    * Running your script
    * `IDEs` and tools for `Python`
<!-- chapter: basic-types-and-operators, duration: 4h -->
* Basic types and operators (2 hours)
    * Why do we need basic types
        * `int`
        * float
    * `str`
        * Different quotes (`'`, `"`, `"""`)
        * Concatenating strings
        * String formatting and formatting aids
        * String slices
        * Splitting and joining
                * `Unicode` in `Python`
        * `bool`
<!-- chapter: basic-data-structures, duration: 10h -->
* Basic Data structures (4 hours)
    * list
        * Literal syntax
        * `OO` syntax
        * Slices
        * Catenating
        * Deleting parts of lists
        * Correct iteration
    * dict
        * Literal syntax
        * `OO` syntax
        * getting keys
        * Deleting keys
        * getting values
        * Merging
        * Correct iteration
    * set
        * Literal syntax
        * `OO` syntax
        * Merging
        * Intersecting
        * Subtracting
    * tuple
        * creating
        * iterating
        * tuples are immutable (sort of)
    * performances of these data structures
    * Toolkit iteration
        * enumerate
        * `zip`
<!-- chapter: basic-statements, duration: 3h -->
* Basic statements (2 hours)
    * Assignments
    * Expressions
    * print
    * Conditionals if
    * Loops (`while-loop`, `for-loop`)
    * range
    * break and continue
    * pass
<!-- chapter: functions, duration: 2h -->
* Functions (4 hours)
    * Why do we need functions
    * defining functions with the def keyword
    * default values for arguments
    * arbitrary argument lists: *args
    * keyword arguments: **kwargs
    * `lambda` expressions
    * Scoping: global vs function scopes
    * Nested functions and closures
<!-- chapter: using-modules, duration: 2h -->
* Using Modules (2 hours)
    * Why do we need modules
    * Namespaces
    * The various ways of importing other modules
    * Reloading modules
    * Module search path
    * Compiling modules
    * The standard library of modules and it's documentation
    * The dir function
<!-- chapter: classes, duration: 4h -->
* Classes (4 hours)
    * Why do we need classes
    * `Python` Namespaces
    * Object properties
    * The class statement and class object
        * Constructing classes
    * Creating instances
    * Writing methods
    * Special methods __ and operator overloading
    * Instance objects
    * Method objects
    * Using class methods
    * Inheritance in `Python`
    * Mixing classes in `Python`
    * Operator overloading
    * Namespace lookup rules
    * Design using classes
    * Properties and decorators
<!-- chapter: exceptions, duration: 2h -->
* Exceptions (4 hours)
    * Why do we need exceptions
    * How are exceptions used
    * Handling exceptions
    * Raising exceptions
    * User defined exceptions
    * Catching modes
    * Cleanup actions with finally
<!-- chapter: functional-python, duration: 1h -->
* Functional `Python` (2 hours)
    * Iterators
    * Generators
<!-- chapter: systems-programming-in-python, duration: 3h -->
* Systems programming in `Python` (4 hours)
    * IO
        * formatted printing
        * using files
        * reading and writing text files
        * reading and writing `JSON` or `YAML` files
    * Using the `subprocess` module
    * Using the `multiprocessing` module
    * Using the `threading` module
<!-- chapter: using-third-party-modules-and-packaging-modules, duration: 1h -->
* Using third party modules and packaging modules (2 hours)
    * Built in `Python` modules
        * pip
    * Downloading, installing and using modules off the net
<!-- chapter: creating-modules, duration: 2h -->
* Creating Modules (2 hours)
    * Directory structure
    * __init__.py file and code in it
    * Using nearby modules
    * Global code in your module
    * Writing functions and classes in your module
    * Documenting your module using `pydoc`
        * Testing your modules using `pytest`
    * Distributing modules using `setuptools` and `distutils`
<!-- chapter: technology-around-python, duration: 2h -->
* Technology around `Python` (2 hours)
    * Using the `Python` debugger
    * Using the `Python` profiler
    * Using different repositories than `PyPI`
    * Using `virtualenv` and other tools

## Installations
Each student should have:

* The most recommended is an `Ubuntu` 24:04 machine, real or virtual.
The reason for this is that most companies production environment is `Linux` and not `Windows` or `MacOS`
and so it is better to practice on an environment as similar as possible to the production environment
as opposed to an environment used to read emails.
* 4 GB `RAM` for each machine is enough. This is not much.
* Free, wide band, access to the internet from all machines with no weird corporate firewalls that might stop us from installing software and `Python` packages via pip.
* Username and password of a user that has `sudo` privileges on the machine.
* [https://www.linuxvmimages.com/images/ubuntu-2204/](https://www.linuxvmimages.com/images/ubuntu-2204/)
* While `Ubuntu` is the best to exercise `Python` users who want to exercise on `Windows` or on `MacOS` are welcome to do so and I will help them out with issues.
* No need to install `Python` yourself as I will help the students to install `Python` when the course starts.
* Students who wish to practice on `Windows` may do so, but be sure that you can install `Python` on your machine
(some companies and enterprises prohibit installation of software on developers machines).
Be aware that `Python` was designed with `UNIX` in mind and so some systems programming features
may not work correctly on `Windows` but impacts this beginners course very little.
* Students who wish to practice on `MacOS` may do so as well. Again, `make` sure that you have `Python` installed on your laptop (usually it is already installed) or that you have the privilege to install it.
* Recommended development environments: `vscode` with `Python` plugins, `PyCharm` community edition from `JetBrains`.

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
