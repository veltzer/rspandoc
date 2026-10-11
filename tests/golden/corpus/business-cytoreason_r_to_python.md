---
tags:
  - languages:python
  - tools:pandas
  - tools:numpy
  - concepts:oop
  - concepts:concurrency
  - concepts:parallel-computing
  - concepts:optimization
  - data-and-ai:data-science
  - data-and-ai:data-engineering
  - data-and-ai:machine-learning
  - practices:logging
  - practices:profiling
level: intermediate
category: language
duration_hours: 16
audience:
  - audiences:developers
  - audiences:data-scientists
---

<!-- course: cytoreason_r_to_python -->
# CytoReason R to `Python`

## Description
This is a customized syllabus for CytoReason.

This course is intended for R programmers who need to become productive in `Python` for
data science and bioinformatics work. Rather than teaching `Python` from scratch, the course
constantly maps `Python` concepts back to their R equivalents — 0-based indexing versus
1-based, list comprehensions versus `apply`, dictionaries versus named lists, `pandas` versus
`dplyr`, and so on — so that existing R knowledge is leveraged rather than discarded.

The emphasis of the course is on `Python`, `numpy`, `pandas` and statistical work with
`scikit-learn`. The course works through a realistic data pipeline: cleaning data, building
`DataFrame` pipelines, merging clinical metadata with sequence count matrices, dimensionality
reduction with `PCA`, parsing memory-heavy biological file formats, optimizing and parallelizing
heavy computation, and finally packaging the whole thing as a robust command-line tool.

Each module pairs a focused teaching block with a hands-on exercise that ports or rebuilds a
realistic R workflow in `Python`.

## Duration
16 hours / 2 days

## Intended Audience
* R programmers moving to `Python` for data science or bioinformatics work.
* Data scientists who need to read, maintain and write `Python` data pipelines.

## Prerequisites
* Working knowledge of R, including `dplyr` / tidyverse style data manipulation.
* General familiarity with data analysis concepts such as data frames, joins and aggregations.

## Objectives
* Map core R concepts and idioms onto their `Python` equivalents.
* Replace tidyverse data pipelines with idiomatic `pandas`.
* Merge, reshape and clean real-world data sets in `pandas`.
* Set up reproducible `Python` environments and model data with classes.
* Run `scikit-learn` pipelines including scaling and `PCA`.
* Parse large biological files efficiently using `Biopython` and generators.
* Optimize and parallelize heavy numerical computation.
* Turn an analysis workflow into a robust command-line tool.

## Outline

<!-- chapter: python-for-r-users, duration: 2h -->
* `Python` for R users
    * 0-based indexing and what changes because of it
    * Lists, tuples and slicing versus R vectors
    * List comprehensions versus `apply` family functions
    * Dictionaries versus named lists
    * Strings, formatting and f-strings
    * Functions, default arguments and keyword arguments
    * Reading and cleaning a basic data set
    * Exercise: porting a basic R data-cleaning script to `Python`

<!-- chapter: pandas-for-dplyr-users, duration: 2h -->
* `pandas` for `dplyr` users
    * The `DataFrame` and `Series` data structures
    * Selecting columns and rows
    * `loc` versus `iloc`
    * Filtering rows by condition
    * Adding and transforming columns
    * Method chaining versus the pipe operator
    * Mapping common `dplyr` verbs onto `pandas`
    * Exercise: replicating a tidyverse pipeline in `pandas`

<!-- chapter: data-merging-and-reshaping, duration: 2h -->
* Data merging and reshaping
    * Joins with `merge` and the different join types
    * Concatenating and stacking data
    * Aggregations with `groupby`
    * Pivoting, melting and reshaping
    * Handling missing data
    * Indexes and multi-indexes
    * Exercise: merging clinical metadata with sequence count matrices

<!-- chapter: environments-and-oop, duration: 2h -->
* Environments and `OOP`
    * Installing packages with pip and `conda`
    * Virtual environments and why they matter
    * Reproducible environments and dependency files
    * Classes versus `S3` / `S4` objects
    * Attributes, methods and the `self` parameter
    * Constructors and basic data modeling with classes
    * Exercise: setting up a reproducible environment and writing a custom data class

<!-- chapter: dimensionality-reduction, duration: 2h -->
* Dimensionality reduction
    * The `scikit-learn` estimator and transformer `API`
    * Data scaling and standardization
    * `PCA` concepts and how to apply it
    * Explained variance and choosing components
    * Visualizing results with `matplotlib`
    * Exercise: running a `PCA` pipeline on the merged matrices and visualizing variance

<!-- chapter: specialized-bio-tooling, duration: 2h -->
* Specialized bio-tooling
    * Overview of `Biopython`
    * Parsing `FASTA` and `VCF` files
    * Iterators and generator expressions
    * Why generators matter for memory-heavy files
    * Streaming records instead of loading everything into memory
    * Exercise: parsing memory-heavy `FASTA` / `VCF` files efficiently

<!-- chapter: code-optimization-and-parallel-processing, duration: 2h -->
* Code optimization and parallel processing
    * `numpy` arrays and vectorization
    * Replacing loops with vectorized operations
    * Profiling code to find the real bottlenecks
    * Parallelism with `joblib`
    * The `multiprocessing` module and the `GIL`
    * Distributing a heavy calculation across multiple `CPU` cores
    * Exercise: profiling code and distributing a heavy calculation across multiple `CPU` cores

<!-- chapter: pipeline-automation, duration: 2h -->
* Pipeline automation
    * Parsing command-line arguments with `argparse`
    * Structuring a script as a reusable tool
    * Error handling and exceptions
    * Logging with the `logging` module
    * Making a workflow robust and reproducible
    * Exercise: converting the `PCA` workflow into a robust command-line tool (`CLI`)

## Installations
Each student should have:

* An `Ubuntu` 24:04 machine, real or virtual.
* 4 GB `RAM` for each machine is enough. This is not much.
* Free, wide band, access to the internet from all machines with no weird corporate firewalls that might stop us from installing software and `Python` packages via pip.
* Username and password of a user that has `sudo` privileges on the machine.
* [https://www.linuxvmimages.com/images/ubuntu-2204/](https://www.linuxvmimages.com/images/ubuntu-2204/)
* Users who want to exercise on `Windows` or on `MacOS` are welcome to do so and I will help them to fix issues but a minority of the more advanced exercises may not work on those machine.

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
