---
tags:
  - languages:julia
  - concepts:programming
  - data-and-ai:data-science
level: intermediate
category: language
duration_hours: 32
audience:
  - audiences:developers
  - audiences:data-scientists
---

<!-- course: julia_programming -->
# Julia Programming

## Description
Julia is a high-level, high-performance programming language designed for technical computing.
It combines the ease of use of `Python` with the speed of C, making it ideal for scientific
computing, data science, and `machine learning`. This course covers Julia from its fundamentals
through advanced topics including its powerful type system, multiple dispatch, metaprogramming,
parallel and `GPU` computing, and the rich ecosystem of packages for data analysis and machine
learning. Students will learn to write performant, idiomatic Julia code and leverage the
language's unique strengths.
The course includes hands on exercises.

## Duration
32 hours / 4 days

## Intended Audience
* Developers looking to adopt a high-performance language for technical computing.
* Data scientists seeking a faster alternative to `Python` or R.
* Researchers and engineers working on numerical simulation, optimization, or `machine learning`.

## Prerequisites
* Programming experience in at least one language such as `Python`, `MATLAB`, R, or C.
* Basic understanding of linear algebra and statistics is helpful.
* Familiarity with command-line interfaces.

## Required Knowledge
* `Python` Programming (or equivalent experience)

## Objectives
* Write idiomatic and performant Julia code
* Leverage multiple dispatch and the type system effectively
* Use metaprogramming to generate efficient code
* Perform parallel and `GPU` computing in Julia
* Work with data using DataFrames.jl and visualization libraries
* Build `machine learning` models with `Flux`.jl
* Create and publish Julia packages

## Outline
<!-- chapter: getting-started-with-julia, duration: 2h -->
* Getting Started with Julia
    * What is Julia and why use it?
    * Installation and environment setup
    * The Julia REPL and workflow
    * Variables, expressions, and basic types
    * Strings and string interpolation
    * Control flow: if/else, for, while
    * First program and basic `I/O`
<!-- chapter: functions-and-multiple-dispatch, duration: 2h -->
* Functions and Multiple Dispatch
    * Defining functions (standard and short form)
    * Anonymous functions and closures
    * Multiple dispatch fundamentals
    * Method definitions and dispatch rules
    * Variadic and keyword arguments
    * Function composition and piping
<!-- chapter: type-system, duration: 3h -->
* Type System
    * `Abstract-types` and type hierarchy
    * Concrete types and primitive types
    * Composite types (structs: mutable and immutable)
    * Parametric types
    * Union types
    * Type annotations and conversions
    * Designing with the type system
<!-- chapter: arrays-and-linear-algebra, duration: 3h -->
* Arrays and Linear Algebra
    * `Array` creation and initialization
    * Indexing, slicing, and views
    * Broadcasting
    * Comprehensions and generators
    * Matrix operations and linear algebra
    * Sparse arrays
    * The LinearAlgebra standard library
<!-- chapter: metaprogramming, duration: 2h -->
* Metaprogramming
    * Expressions and symbols
    * Quoting and interpolation
    * Macros: definition and usage
    * Generated functions
    * Code introspection (@code_lowered, @code_native)
    * Common macros and when to use metaprogramming
<!-- chapter: modules-and-packages, duration: 2h -->
* Modules and Packages
    * Modules: definition, export, and import
    * The Pkg package manager
    * Project environments and Project.`toml`
    * Finding and using packages
    * Creating your own package
    * Package versioning and compatibility
<!-- chapter: i-o-and-data-formats, duration: 2h -->
* `I/O` and Data Formats
    * `File` `I/O` (text and binary)
    * `CSV`.jl for tabular data
    * `JSON`.jl and `YAML`.jl
    * JLD2.jl for Julia-native serialization
    * Working with databases
    * `HTTP` and web requests
<!-- chapter: performance-tips-and-optimization, duration: 2h -->
* Performance Tips and Optimization
    * Type stability and its importance
    * Benchmarking with BenchmarkTools.jl
    * Profiling and identifying bottlenecks
    * Memory allocation and avoiding allocations
    * Performance annotations (@inbounds, @simd, @fastmath)
    * Writing C-speed Julia code
<!-- chapter: parallel-and-distributed-computing, duration: 2h -->
* Parallel and Distributed Computing
    * Multithreading with Threads.@threads
    * Thread safety and atomic operations
    * Task-based concurrency
    * Distributed computing with Distributed.jl
    * pmap and distributed arrays
    * Parallel patterns and best practices
<!-- chapter: gpu-computing, duration: 2h -->
* `GPU` Computing
    * Introduction to `GPU` programming
    * CUDA.jl for NVIDIA GPUs
    * Writing `GPU` kernels
    * `Array` operations on the `GPU`
    * Performance considerations for `GPU` code
<!-- chapter: data-science-with-julia, duration: 2h -->
* Data Science with Julia
    * DataFrames.jl: creating and manipulating data frames
    * Data transformation and aggregation
    * Split-apply-combine operations
    * Joins and reshaping
    * Missing data handling
<!-- chapter: visualization, duration: 2h -->
* Visualization
    * Plots.jl: basic and advanced plotting
    * Plot backends and customization
    * Makie.jl for interactive and publication-quality graphics
    * Statistical visualizations
<!-- chapter: machine-learning-with-flux-jl, duration: 2h -->
* `Machine Learning` with `Flux`.jl
    * Introduction to `Flux`.jl
    * Building neural network layers
    * Training loops and loss functions
    * Optimizers and gradient descent
    * Model evaluation and saving
    * Common architectures (dense, convolutional, recurrent)
<!-- chapter: interoperability, duration: 2h -->
* Interoperability
    * Calling C and `Fortran` from Julia
    * PyCall.jl: calling `Python` from Julia
    * RCall.jl: calling R from Julia
    * Wrapping external libraries
<!-- chapter: testing-and-package-development, duration: 2h -->
* Testing and Package Development
    * Unit testing with Test standard library
    * Test sets and test organization
    * Continuous integration for Julia packages
    * Documentation with Documenter.jl
    * Package development workflow
    * Publishing to the General registry

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
