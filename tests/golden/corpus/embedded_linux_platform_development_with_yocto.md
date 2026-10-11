---
tags:
  - infrastructure:linux
  - hardware-and-embedded:embedded
  - tools:yocto
  - tools:bitbake
  - infrastructure:cross-compilation
  - practices:build-systems
  - practices:ci-cd
level: advanced
category: operating-systems
duration_hours: 24
audience:
  - audiences:developers
  - audiences:embedded-engineers
  - audiences:architects
  - audiences:devops
---

<!-- course: embedded_linux_platform_development_with_yocto -->
# Embedded `Linux` Platform Development with `Yocto`

## Description
The `Yocto` Project has emerged as the industry standard for creating custom embedded `Linux` distributions, powering everything from IoT devices to automotive systems. This comprehensive course teaches participants how to leverage `Yocto`'s powerful build system to create maintainable, reproducible, and production-ready embedded `Linux` systems.

Unlike traditional approaches to embedded `Linux` development, `Yocto` provides a structured, metadata-driven framework that scales from single-board prototypes to complex multi-platform product lines. This course emphasizes practical, hands-on experience with modern `Yocto` workflows, including containerized builds, `CI/CD` integration, and security hardening.

Participants will learn not just how to use `Yocto`, but how to architect sustainable build systems that can evolve with their products over years of development and deployment.

## Duration
24 hours / 3 days

## Intended Audience
* Embedded software engineers transitioning to `Linux`-based systems
* `Linux` developers seeking structured approaches to embedded development
* `DevOps` engineers responsible for embedded build infrastructure
* System architects designing multi-platform embedded products
* Engineers familiar with `Buildroot` or manual builds wanting to adopt `Yocto`
* Technical leads evaluating embedded `Linux` build solutions

## Prerequisites

### Required
* Proficiency with `Linux` command line and shell scripting
* Understanding of compilation, linking, and build processes
* `Git` version control fundamentals
* Basic understanding of `Linux` system architecture (bootloader, kernel, rootfs)

### Recommended
* Experience with any embedded development (RTOS, bare-metal, or `Linux`)
* Familiarity with Makefiles, `CMake`, or other build systems
* Basic `Python` scripting knowledge
* Understanding of cross-compilation concepts
* Experience with virtualization (`QEMU`, `Docker`)

## Required Knowledge
* `Docker` Fundamentals (or equivalent experience)
* `Python` Programming (or equivalent experience)

## Objectives
Upon completion of this course, participants will be able to:
* Design and implement production-ready embedded `Linux` systems using the `Yocto` Project
* Create custom layers and recipes following industry best practices
* Optimize build performance and implement efficient `CI/CD` pipelines
* Implement security hardening and license compliance workflows
* Debug complex build and runtime issues using `Yocto`'s comprehensive toolset
* Deploy OTA update mechanisms for field-deployed devices
* Integrate `Yocto` builds into modern containerized `DevOps` workflows
* Make informed decisions about when to use `Yocto` versus alternatives

## Exercises
* Lab 1: First Build Experience
    * Setting up build environment
    * Building core-image-minimal for `QEMU`
    * Running and interacting with the image
    * Exploring the build directory structure

* Lab 2: Custom Layer Creation
    * Creating a new layer from scratch
    * Adding the layer to bblayers.conf
    * Creating a simple "Hello World" recipe
    * Building and testing the recipe

* Lab 3: Application Recipe Development
    * Writing a recipe for a `C++` application using `CMake`
    * Adding a `Python` application with dependencies
    * Creating a `Node.js` service recipe
    * Handling recipe upgrades and patches

* Lab 4: Kernel Customization
    * Creating kernel configuration fragments
    * Adding out-of-tree kernel modules
    * Modifying `device tree` files
    * Building and deploying custom kernel

* Lab 5: Custom Image Creation
    * Designing a custom embedded `Linux` image
    * Implementing partition layout with WIC
    * Adding custom packages and features
    * Optimizing image size

* Lab 6: `CI/CD` Pipeline Implementation
    * Setting up containerized build environment
    * Creating `GitLab CI` pipeline for `Yocto` builds
    * Implementing build caching strategies
    * Automated testing and deployment

* Lab 7: Security Hardening Workshop
    * Enabling and reviewing CVE checking
    * Implementing compiler security flags
    * Creating read-only rootfs
    * Setting up secure boot (simulated)

* Lab 8: OTA Update Implementation
    * Integrating SWUpdate into custom image
    * Creating update packages
    * Implementing A/B partition scheme
    * Testing update and rollback procedures

## Outline
<!-- chapter: foundation-architecture, duration: 2h -->
* Foundation & Architecture
    * Introduction to Embedded `Linux` Build Systems
        * Evolution of embedded `Linux` development approaches
        * Comparative analysis: `Yocto` vs `Buildroot` vs OpenWrt vs manual builds
        * `Yocto` Project ecosystem and governance
        * Industry adoption case studies (Automotive, Industrial IoT, Telecommunications)
        * `When` to choose `Yocto`: project requirements analysis
    * `Yocto` Project Architecture
        * Core components deep dive
            * Poky reference distribution
            * `OpenEmbedded`-Core (OE-Core)
            * `BitBake` build engine
        * Metadata architecture
            * Recipes, classes, and configuration files
            * Layer concept and hierarchy
            * Override and append mechanisms
        * Build flow and task execution model
            * Parsing phase
            * Task generation and dependencies
            * Execution and parallelization
    * Development Environment Setup
        * Host system requirements and optimization
            * `CPU`, memory, and storage considerations
            * `Linux` distribution selection
        * Container-based development with CROPS
            * `Docker` setup for consistent environments
            * Volume management for builds
        * Essential tools installation
            * Required packages and dependencies
            * Optional productivity tools
        * Initial build workspace configuration

<!-- chapter: bitbake-deep-dive, duration: 3h -->
* `BitBake` Deep Dive
    * `BitBake` Fundamentals
        * Recipe anatomy and structure
            * Header variables (DESCRIPTION, LICENSE, SRC_URI)
            * Task definitions and overrides
            * Installation directives
        * Variable operations
            * Immediate vs deferred expansion
            * Operators (=, ?=, ??=, +=, =+, .=, =.)
        * Classes and inheritance
            * Common classes (`autotools`, `cmake`, python3)
            * Creating custom classes
        * Configuration files
            * local.conf essentials
            * bblayers.conf management
            * site.conf for team settings
    * Advanced `BitBake` Concepts
        * Override syntax and conditional metadata
            * Machine-specific overrides
            * Distribution overrides
            * Runtime vs build-time conditionals
        * Event handling system
            * Event types and handlers
            * Custom event implementation
        * Shared state cache (sstate)
            * Cache mechanism internals
            * Hash generation and validation
            * Cache sharing strategies
        * Fetcher system
            * Supported protocols (`git`, `http`, `svn`, etc.)
            * Mirror configuration
            * Shallow clones and bandwidth optimization
    * Debugging `BitBake`
        * Essential debugging commands
            * `bitbake` -e for environment inspection
            * `bitbake` -g for dependency graphs
            * `bitbake` -`DDD` for verbose output
        * `bitbake`-layers tool mastery
            * Layer inspection and manipulation
            * Recipe search and analysis
        * Dependency visualization
            * Task dependency graphs
            * Package dependency analysis
            * GraphViz integration
        * Common issues and solutions
            * Circular dependencies
            * Missing dependencies
            * Hash mismatches

<!-- chapter: practical-image-development, duration: 3h -->
* Practical Image Development
    * Building Your First Images
        * `QEMU`-based development workflow
            * Supported architectures (`ARM`, `x86`, MIPS, etc.)
            * `QEMU` configuration and optimization
            * Network and storage setup
        * Core image types
            * core-image-minimal
            * core-image-base
            * core-image-sato
            * core-image-weston
        * Package management systems
            * RPM with DNF
            * DEB with `APT`
            * IPK with OPKG
            * Choosing the right package manager
    * Image Customization Strategies
        * Creating custom image recipes
            * Inheriting from core-image
            * Package selection strategies
            * IMAGE_INSTALL vs CORE_IMAGE_EXTRA_INSTALL
        * Feature configuration
            * IMAGE_FEATURES (debug-tweaks, splash, etc.)
            * DISTRO_FEATURES (`systemd`, `wayland`, `x11`, etc.)
            * MACHINE_FEATURES (`wifi`, `bluetooth`, touchscreen, etc.)
            * Feature dependencies and conflicts
        * Rootfs customization
            * Post-processing hooks
            * User and group management
            * Default configuration files
            * `Systemd` vs SysVinit configuration
    * `BSP` Development
        * Machine configuration files
            * Hardware description variables
            * Kernel and bootloader selection
            * Image format specification
        * Bootloader integration
            * `U-Boot` configuration and customization
            * `GRUB` for `x86` platforms
            * Device-specific bootloaders
        * Hardware adaptation
            * `GPIO` and peripheral configuration
            * Display and touchscreen setup
            * Network interface configuration
            * Power management settings

<!-- chapter: layer-management-recipes, duration: 3h -->
* Layer Management & Recipes
    * Layer Architecture Best Practices
        * Creating maintainable layers
            * Layer structure and naming conventions
            * README and licensing documentation
            * Layer.conf configuration
        * Priority and override management
            * BBFILE_PRIORITY settings
            * Override precedence rules
            * Debugging override conflicts
        * Inter-layer dependencies
            * LAYERDEPENDS configuration
            * Version compatibility
            * Layer compatibility testing
    * Recipe Development Workshop
        * Modern build system integration
            * `CMake` recipe patterns
            * `Meson` build system
            * Cargo for Rust projects
            * Go module support
        * Language-specific recipes
            * `Python` applications with `setuptools`
            * `Node.js` and `npm` packages
            * `Java` applications
        * Proprietary software integration
            * Binary-only packages
            * License flag configuration
            * Commercial toolchain integration
        * `Git`-based development
            * AUTOREV for continuous integration
            * Shallow clones for large repositories
            * Submodule handling
            * Private repository authentication
    * Package Groups and Variants
        * Creating logical software bundles
            * Package group recipes
            * Task packages
            * Meta packages
        * Product variant management
            * Multiple image configurations
            * Conditional package inclusion
            * Feature-based variants
        * Dependency management
            * Runtime vs build dependencies
            * Recommended vs required packages
            * Virtual package providers

<!-- chapter: kernel-device-integration, duration: 2h -->
* Kernel & Device Integration
    * Kernel Configuration Management
        * Kernel recipe types
            * `linux`-`yocto` advantages and features
            * Custom kernel integration
            * Vendor kernel adaptation
        * Configuration techniques
            * Configuration fragments (.cfg files)
            * defconfig management
            * menuconfig integration
            * Configuration validation
        * `Device tree` management
            * DTS/DTB compilation
            * Overlay support
            * Runtime `device tree` modification
            * Multi-platform device trees
    * Device Drivers and Firmware
        * Out-of-tree module development
            * Module recipe templates
            * Kernel version dependencies
            * Module signing and security
        * Firmware management
            * Firmware loading mechanisms
            * License considerations
            * Update strategies
        * Hardware initialization
            * Early boot configuration
            * Platform-specific initialization
            * Peripheral power sequencing

<!-- chapter: development-workflow-tools, duration: 2h -->
* Development Workflow & Tools
    * Modern Development Tools
        * devtool workflow
            * modify/upgrade/add commands
            * Workspace management
            * Patch generation and management
        * recipetool capabilities
            * Automated recipe creation
            * Dependency detection
            * License scanning
        * `SDK` generation and usage
            * Standard `SDK` vs Extensible `SDK`
            * Cross-compilation environment
            * Sysroot management
            * IDE integration (`VS Code`, Eclipse, `CLion`)
    * Image Creation Tools
        * WIC (`OpenEmbedded` Image Creator)
            * Partition layout definition
            * Bootloader installation
            * Multiple storage device support
            * Custom .wks files
        * Container image generation
            * `Docker`/`OCI` image creation
            * Minimal container images
            * Multi-stage builds
        * Multi-configuration builds
            * Multiconfig syntax and usage
            * Building for multiple machines
            * Container and VM combinations
    * Testing and Quality Assurance
        * ptest framework
            * Test integration in recipes
            * Runtime test execution
            * Results collection and analysis
        * testimage framework
            * `QEMU`-based testing
            * Hardware testing setup
            * Custom test case development
        * Continuous testing strategies
            * Automated build verification
            * Regression testing
            * Performance benchmarking

<!-- chapter: build-optimization-ci-cd, duration: 3h -->
* Build Optimization & `CI/CD`
    * Build Performance Optimization
        * Parallelization configuration
            * BB_NUMBER_THREADS tuning
            * PARALLEL_MAKE optimization
            * System resource management
        * Shared state optimization
            * sstate cache configuration
            * Hash Equivalence Server setup
            * Network cache sharing
            * Cache pruning strategies
        * Build cluster configuration
            * Distributed builds with Icecream
            * `Kubernetes`-based build farms
            * Resource scheduling
    * `CI/CD` Integration
        * Pipeline implementation
            * `GitLab CI` configuration
            * `Jenkins` pipeline scripts
            * `GitHub Actions` workflows
            * `Bitbucket` Pipelines
        * Artifact management
            * Build artifact storage
            * Package repository setup
            * Image registry integration
        * Release automation
            * Version management
            * Changelog generation
            * Release note automation
    * Container-based Workflows
        * CROPS deployment
            * Container configuration
            * Volume management
            * Performance optimization
        * Registry management
            * Base image creation
            * Layer caching strategies
            * Security scanning
        * `Kubernetes` integration
            * Job scheduling
            * Resource allocation
            * Distributed caching

<!-- chapter: security-compliance, duration: 3h -->
* Security & Compliance
    * Security Hardening
        * CVE management
            * CVE checking configuration
            * Vulnerability database updates
            * Patch management workflow
            * Security advisory integration
        * Compiler hardening
            * Security flags configuration
            * Stack protection
            * FORTIFY_SOURCE
            * Address sanitization
        * Secure boot implementation
            * Chain of trust establishment
            * Key management
            * Signature verification
            * Measured boot
        * Read-only rootfs
            * Configuration techniques
            * Overlay filesystem setup
            * Persistent storage management
    * License Compliance
        * License tracking
            * License manifest generation
            * License compatibility checking
            * Commercial license handling
        * `SPDX` integration
            * `SBOM` generation
            * Dependency tracking
            * Vulnerability correlation
        * Compliance reporting
            * Automated report generation
            * Legal review workflows
            * Archive generation for GPL compliance
    * Supply Chain Security
        * Binary reproducibility
            * Reproducible builds configuration
            * Build environment isolation
            * Timestamp and path handling
        * Dependency verification
            * Checksum validation
            * Signature verification
            * Mirror security
        * Security scanning
            * Static analysis integration
            * Runtime security testing
            * Container scanning

<!-- chapter: advanced-topics-production, duration: 3h -->
* Advanced Topics & Production
    * Production Deployment
        * OTA update systems
            * SWUpdate integration
            * Mender configuration
            * RAUC implementation
            * Custom update solutions
        * Partition schemes
            * A/B update strategies
            * Recovery partition design
            * Failsafe mechanisms
        * Factory provisioning
            * Initial device programming
            * Serialization and customization
            * Test and validation
            * Certificate installation
    * Debugging Production Issues
        * Remote debugging
            * `GDB` server configuration
            * Symbol management
            * Core dump collection
        * System monitoring
            * Resource usage tracking
            * Log aggregation
            * Metrics collection
        * Performance analysis
            * Profiling tools integration
            * Bottleneck identification
            * Optimization strategies
    * `Yocto` Ecosystem
        * Industry-specific distributions
            * AGL (Automotive Grade `Linux`)
            * O-RAN (Telecom)
            * Industrial automation profiles
        * Layer ecosystem
            * meta-`openembedded`
            * meta-virtualization
            * meta-security
            * `BSP` layers
        * Commercial support
            * Vendor distributions
            * Professional services
            * Long-term support options

## Links
* Official `Yocto` Project documentation (latest release)
* `OpenEmbedded` layer index
* Bootlin's embedded `Linux` training materials
* Community forums and mailing lists
* Sample code repository with all lab solutions
* Quick reference cards for `BitBake` syntax and common tasks

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
