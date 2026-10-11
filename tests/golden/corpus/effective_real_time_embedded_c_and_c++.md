---
tags:
  - hardware-and-embedded:embedded
  - infrastructure:real-time
  - languages:c
  - languages:c++
level: advanced
category: embedded
duration_hours: 32
audience:
  - audiences:embedded-engineers
  - audiences:developers
---

<!-- course: effective_real_time_embedded_c_and_c++ -->
# Effective Real Time Embedded C and `C++`

## Description
Many software engineers work with C and `C++` programming languages for real time and embedded
development. This course aims to give them a fuller appreciation of the more advanced aspects of the
languages in the real time and embedded environments.
Programming real time embedded systems `make` many special demands on the software engineer.
These include understanding compile, link and run-time issues, which are included in the course. It also
covers real time programming issues that are specific to the C and `C++` languages.
Most of the course is relevant to both C and `C++` software engineers, though some special points
relevant to `C++` and object oriented are also presented.
The course is specifically aimed at issues relevant to real-time embedded software engineers.

## Duration
4 days / 32 hours

## Intended Audience
The course is designed for software engineers who want to improve their use of the C and / or `C++`
programming languages in a real time, embedded environment.

## Prerequisites
* A good grasp of the fundamentals of C and/or `C++`
* A good grasp of real time and embedded is an advantage.

## Objectives
* understand the core concepts and principles of Effective Real Time Embedded C and `C++`
* gain practical knowledge of Introduction
* gain practical knowledge of Embedded C
* gain practical knowledge of Elements of C/```C++``

## Outline
<!-- chapter: introduction, duration: 1h -->
* Introduction
<!-- chapter: embedded-c, duration: 1h -->
* Embedded C
    * Real time C
    * Embedded C
    * Writing code in Kernel Space
<!-- chapter: elements-of-c-c, duration: 3h -->
* Elements of C/`C++`
    * Type Specifiers and Qualifiers
    * Type coercion and conversion
    * Struct layout
    * Uses of union
    * Bitfields
    * Enum v #define
    * Typedef
    * Unscrambling declarations (cdecl)
    * Floating-point and fixed-point number systems
    * IEEE Standard 754
    * Importance of good structure
    * Quality and style
<!-- chapter: effective-pointers, duration: 2h -->
* Effective Pointers
    * Arrays and pointers; compatibility and incompatibility
    * The principle of dynamic memory allocation
    * Void pointers
    * Implementing basic data structures: pointers, arrays, queues, stacks, lists
    * Function pointers: basics, callbacks, state machine
    * Optimizations with pointers
    * Variable sized structures
<!-- chapter: memory-management-architecture, duration: 1h -->
* Memory Management Architecture
    * Caching
    * Hardware memory management
    * Sections of a program
    * Mapping of program in memory
    * Dynamic memory allocation
<!-- chapter: intertask-communication, duration: 2h -->
* Intertask Communication
    * Intertask Synchronization: Race Conditions and Critical Sections
    * Shared Memory
    * Locks
    * Mutexes
    * Semaphores
    * Deadlocks
    * Signals
    * Message Queues
<!-- chapter: the-toolchain, duration: 2h -->
* The Toolchain
    * Pre-processor compiler directives and cross compilers
    * Macros v functions
    * The Linker and memory sections
    * Using the toolchain to your advantage
    * What happens before main
    * What is happening at runtime
    * Estimation of stack requirements
<!-- chapter: optimizations, duration: 1h -->
* Optimizations
    * Tips for optimizing embedded C/`C++` code
<!-- chapter: writing-safer-c, duration: 1h -->
* Writing Safer C
    * `MISRA`-C and `CWE`
    * Compiler warnings
    * Hard learned pitfalls
    * Coding style and banned `API`
    * Preparing for debug
<!-- chapter: hardware-programming-model, duration: 1h -->
* Hardware Programming Model
    * Interrupts - handlers and dispatchers
    * Hardware and Software interrupts
    * Contexts
<!-- chapter: timing, duration: 1h -->
* Timing
    * The challenge in measuring time
    * Hardware and Software timers
    * Watchdog timer
    * Real time debug vs. time
<!-- chapter: object-oriented-c, duration: 1h -->
* Object oriented C
    * Implementing `OOP` with C (OOC)
    * Mixing C and `C++`
<!-- chapter: inheritance-and-oo-design, duration: 1h -->
* Inheritance and `OO` Design
    * Inheritance models
    * The use of `virtual-functions`
<!-- chapter: embedded-cpp, duration: 1h -->
* Embedded `C++`
    * C vs. `C++`
    * Benefits of embedded `C++` (EC++)
    * Restrictions for EC++
<!-- chapter: c-resource-management, duration: 2h -->
* `C++` Resource Management
    * Managing contained members
    * The Rule of the Big Three
    * Garbage collection
    * Smart pointers
    * Resource managing classes
<!-- chapter: templates-and-generic-programming, duration: 2h -->
* Templates and Generic Programming
    * Understand the two meanings of typename
    * Know how to access names in templatized base classes
    * Parameter-independent code
    * Traits
    * Template metaprogramming
<!-- chapter: the-standard-template-library, duration: 2h -->
* The Standard Template Library
    * `STL` Components
    * Containers
    * Iterators
    * Algorithms
    * Using emplace
    * Manipulating Algorithms
    * Container Elements
    * Extending the `STL`
<!-- chapter: input-output-using-stream-classes, duration: 4h -->
* Input/Output Using Stream Classes
    * Common Background of `I/O` Streams
    * Fundamental Stream Classes and Objects
    * Standard Stream Operators << and >>
    * Standard Input/Output Functions
    * Manipulators
    * Formatting
    * Internationalization
    * File Access
    * Connecting Input and Output Streams
    * Stream Classes for Strings
    * Input/Output Operators for User-Defined Types
    * The Stream Buffer Classes
    * Performance Issues
<!-- chapter: peripherals-extra, duration: 3h -->
* Peripherals (extra)
    * General Purpose Input/Output (`GPIO`)
    * Universal Asynchronous Receiver-Transmitter (`UART`)
    * Analog-to-Digital Converter (ADC)
    * Serial Peripheral Interface (`SPI`)
    * Inter-Integrated Circuit (`I2C`)
    * Direct Memory Access (`DMA`)
    * Nested Vector Interrupt Controller (NVIC)
    * General Purpose Timers (TIM)
    * System Tick Timer (SysTick)

[//1]: <> (Logtel course # 3411)

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
