---
tags:
  - languages:assembly
  - hardware-and-embedded:x86
  - infrastructure:linux
  - infrastructure:low-level
  - practices:performance
level: advanced
category: language
duration_hours: 32
audience:
  - audiences:developers
---

<!-- course: assembly_programming_using_gas -->
# `Assembly` programming using gas

## Description
This comprehensive course on `Assembly` Programming on `Linux` using GAS (GNU Assembler) offers a deep dive into the world of low-level programming. Designed for both beginners and those with some programming experience,
the course takes students on a journey from the fundamentals of `assembly` language to advanced topics in system-level programming. Starting with an introduction to `assembly` concepts and the `x86`/`x86`-64 architectures, learners will quickly progress to writing and debugging their first `assembly` programs. The curriculum covers crucial areas such as data manipulation, arithmetic operations, control flow, and memory management, all within the context of the `Linux` operating system. Students will have gained practical experience in writing efficient `assembly` code, interfacing with C programs, and leveraging advanced features like SIMD instructions. With a strong emphasis on hands-on projects and real-world applications, this course equips participants with the skills needed to optimize performance-critical code, understand system internals, and even venture into areas like reverse engineering and low-level security analysis.

## Duration
32 hours / 4 days

## Intended Audience
* systems programmers and reverse engineers
* developers who need to understand low-level code execution

## Prerequisites
* experience with C programming
* basic understanding of computer architecture

## Objectives
* understand the core concepts and principles of `Assembly`` programming using gas
* gain practical knowledge of Introduction to `Assembly` Language
* gain practical knowledge of Setting Up the Development Environment
* gain practical knowledge of Basic `Assembly` Concepts

## Outline
<!-- chapter: introduction-to-assembly-language, duration: 1h -->
* Introduction to `Assembly` Language
    * What is `assembly` language?
    * Advantages and disadvantages of `assembly` programming
    * Overview of `x86` and `x86`-64 architectures
<!-- chapter: setting-up-the-development-environment, duration: 1h -->
* Setting Up the Development Environment
    * Installing necessary tools (`GCC`, GAS, `GDB`)
    * Configuring text editor or IDE for `assembly`
<!-- chapter: basic-assembly-concepts, duration: 1h -->
* Basic `Assembly` Concepts
    * Registers and memory
    * Instruction set architecture (`ISA`)
    * Addressing modes
<!-- chapter: gnu-assembler-gas-syntax, duration: 1h -->
* GNU Assembler (GAS) Syntax
    * AT&T syntax vs. Intel syntax
    * Directives and pseudo-ops
    * Labels and comments
<!-- chapter: writing-your-first-assembly-program, duration: 1h -->
* Writing Your First `Assembly` Program
    * "Hello, World!" in `assembly`
    * Assembling and linking
    * Running and debugging
<!-- chapter: data-types-and-data-movement, duration: 2h -->
* Data Types and Data Movement
    * Integer representations
    * Floating-point numbers
    * MOV and other data transfer instructions
<!-- chapter: arithmetic-and-logical-operations, duration: 2h -->
* Arithmetic and Logical Operations
    * Addition, subtraction, multiplication, and division
    * Bitwise operations
    * Shift and rotate instructions
<!-- chapter: flow-control, duration: 2h -->
* Flow Control
    * Unconditional jumps
    * Conditional branches
    * Loops and iteration
<!-- chapter: functions-and-the-stack, duration: 2h -->
* Functions and the Stack
    * Function calls and returns
    * Stack frame and `local-variables`
    * Passing arguments and returning values
<!-- chapter: working-with-arrays-and-strings, duration: 2h -->
* Working with Arrays and Strings
    * Declaring and accessing arrays
    * String operations
    * SIMD instructions for vector operations
<!-- chapter: system-calls-and-file-i-o, duration: 2h -->
* System Calls and `File` `I/O`
    * `Linux` system call interface
    * `File` operations (open, read, write, close)
    * Standard input/output
<!-- chapter: memory-management, duration: 1h -->
* Memory Management
    * Static, stack, and heap allocation
    * Using `malloc` and free from `assembly`
<!-- chapter: optimization-techniques, duration: 2h -->
* Optimization Techniques
    * Common optimization strategies
    * Instruction pipelining and branch prediction
    * Profiling and benchmarking
<!-- chapter: inline-assembly-in-c, duration: 2h -->
* Inline `Assembly` in C
    * Embedding `assembly` in C programs
    * Extended ASM syntax
    * Constraints and clobbers
<!-- chapter: advanced-topics, duration: 2h -->
* Advanced Topics
    * Floating-point operations
    * SIMD programming (`SSE`, AVX)
    * Multi-`threading` basics
<!-- chapter: real-world-applications-and-projects, duration: 2h -->
* Real-world Applications and Projects
    * Implementing simple algorithms
    * Writing device drivers
    * Reverse engineering and security applications
<!-- chapter: debugging-and-tools, duration: 2h -->
* Debugging and Tools
    * Using `GDB` for `assembly` programs
    * Objdump and other binary analysis tools
    * Performance profiling
<!-- chapter: best-practices-and-coding-standards, duration: 2h -->
* Best Practices and Coding Standards
    * Code organization and commenting
    * Naming conventions
    * Error handling in `assembly`
<!-- chapter: comparison-with-other-architectures, duration: 1h -->
* Comparison with Other Architectures
    * `ARM` `assembly` basics
    * RISC vs. CISC architectures
<!-- chapter: future-of-assembly-programming, duration: 1h -->
* Future of `Assembly` Programming
    * Role of `assembly` in modern software development
    * Emerging architectures and instruction sets

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
