---
tags:
  - languages:java
  - practices:testing
  - practices:tdd
level: intermediate
category: language
duration_hours: 16
audience:
  - audiences:developers
---

<!-- course: java_testing -->
# `Java` Testing

## Description
This course covers the full spectrum of testing in the `Java` ecosystem. From `JUnit 5` architecture and assertions through mocking with `Mockito`, integration testing with `Spring Boot` Test and `TestContainers`, to code coverage analysis with JaCoCo and mutation testing. Students will learn how to write effective, maintainable tests and adopt `TDD` as a development practice.

## Duration
16 hours / 2 days

## Intended Audience
* `Java` developers who want to improve their testing skills and code quality
* Software engineers adopting test-driven development practices
* Team leads seeking to establish testing standards in their teams

## Prerequisites
* Intermediate `Java` programming skills (classes, interfaces, generics, annotations)
* Basic understanding of object-oriented design principles
* Familiarity with a build tool such as Maven or `Gradle`
* Experience with an IDE such as `IntelliJ IDEA` or Eclipse

## Objectives
* Master the `JUnit 5` architecture and its extension model
* Write clear, maintainable unit tests with expressive assertions
* Use `Mockito` to isolate units under test with test doubles
* Implement integration tests using `Spring Boot` Test and `TestContainers`
* Measure and analyze test coverage with JaCoCo
* Apply `TDD` workflow to drive design and implementation
* Understand and apply mutation testing to evaluate test suite quality

## Outline
<!-- chapter: introduction-to-java-testing, duration: 1h -->
* Introduction to `Java` Testing
    * Why testing matters
    * Types of tests: unit, integration, end-to-end
    * The test pyramid
    * Testing terminology and conventions
<!-- chapter: junit-5-architecture, duration: 1h -->
* `JUnit 5` Architecture
    * `JUnit` Platform, Jupiter, and Vintage
    * The Jupiter extension model
    * Test classes and methods
    * Display names and nested tests
    * Conditional test execution
    * Test execution order
<!-- chapter: assertions, duration: 1h -->
* Assertions
    * Basic assertions: assertEquals, assertTrue, assertNull
    * Grouped assertions and assertAll
    * Exception assertions with assertThrows
    * Timeout assertions
    * Third-party assertion libraries: `AssertJ` and `Hamcrest`
    * Writing custom matchers
<!-- chapter: parameterized-tests, duration: 1h -->
* Parameterized Tests
    * @ParameterizedTest annotation
    * Value sources: @ValueSource, @EnumSource, @NullAndEmptySource
    * @CsvSource and @CsvFileSource
    * @MethodSource and @ArgumentsSource
    * Custom argument converters and aggregators
<!-- chapter: test-lifecycle, duration: 1h -->
* Test Lifecycle
    * @BeforeEach, @AfterEach, @BeforeAll, @AfterAll
    * Test instance lifecycle modes
    * Extension lifecycle callbacks
    * Temporary directory extension
    * Resource management in tests
<!-- chapter: mockito-and-test-doubles, duration: 2h -->
* `Mockito` and Test Doubles
    * Types of test doubles: stubs, mocks, spies, fakes
    * Creating mocks with `Mockito`
    * Stubbing method calls
    * Argument matchers and captors
    * Verifying interactions
    * @Mock, @InjectMocks, and @Spy annotations
    * Mocking `static-methods` and constructors
    * `BDD Mockito` style
<!-- chapter: integration-testing, duration: 1h -->
* Integration Testing
    * Integration test strategies
    * Database testing with embedded databases
    * `REST API` testing with MockMvc
    * WebTestClient for reactive endpoints
    * Testing with real `HTTP` clients
<!-- chapter: spring-boot-testing, duration: 1h -->
* `Spring Boot` Testing
    * @SpringBootTest and test slices
    * @DataJpaTest for repository testing
    * @WebMvcTest for controller testing
    * @MockBean and @SpyBean
    * Test configuration and profiles
    * Application context caching
<!-- chapter: testcontainers, duration: 2h -->
* `TestContainers`
    * Introduction to `TestContainers`
    * Setting up containerized databases for tests
    * `TestContainers` with `JUnit 5`
    * Custom container definitions
    * `TestContainers` with `Spring Boot`
    * Testing with `Kafka`, `Redis`, and other services
<!-- chapter: test-coverage-with-jacoco, duration: 2h -->
* Test Coverage with JaCoCo
    * Understanding code coverage metrics
    * Line, branch, and instruction coverage
    * Configuring JaCoCo with Maven and `Gradle`
    * Generating and interpreting coverage reports
    * Setting coverage thresholds
    * Excluding classes and packages from coverage
<!-- chapter: tdd-workflow, duration: 1h -->
* `TDD` Workflow
    * Red-Green-Refactor cycle
    * Writing the test first
    * Incremental design with `TDD`
    * `TDD` and `clean architecture`
    * `When` `TDD` works well and when to adapt
<!-- chapter: mutation-testing, duration: 2h -->
* Mutation Testing
    * What is mutation testing
    * Mutation operators and mutants
    * PIT mutation testing framework
    * Configuring and running PIT
    * Interpreting mutation testing reports
    * Improving test suites based on mutation results

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
