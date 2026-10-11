---
tags:
  - languages:typescript
  - concepts:web-development
level: intermediate
category: language
duration_hours: 32
audience:
  - audiences:developers
---

<!-- course: angular -->
# Angular Development

## Description
This course offers a thorough exploration of the Angular framework for building scalable, enterprise-grade web applications. Starting with Angular architecture and a `TypeScript` refresher, the course covers components, templates, data binding, directives, services, dependency injection, routing, forms, `HTTP` communication, `RxJS`, state management with NgRx, testing, and deployment. Students will gain the skills to build, test, and deploy production-ready Angular applications.

## Duration
32 hours / 4 days

## Intended Audience
* Frontend developers building enterprise-scale single-page applications
* `JavaScript` or `TypeScript` developers adopting the Angular framework
* Full-stack developers who need to build robust client-side applications
* Development teams migrating from AngularJS to modern Angular
* Software engineers working in organizations that use Angular as their primary frontend framework

## Prerequisites
* Good understanding of `HTML`, `CSS`, and `JavaScript` (`ES6`+ features)
* Basic knowledge of `TypeScript` (types, interfaces, classes, decorators)
* Familiarity with `npm` and command-line tools
* Understanding of `HTTP` protocol and `REST API` concepts
* Experience with basic web development and browser developer tools

## Required Knowledge
* `HTML` `CSS` `JavaScript` Fundamentals (or equivalent experience)
* `TypeScript` Programming (or equivalent experience)

## Objectives
* Understand Angular architecture, modules, and the component-based model
* Build components with templates, data binding, directives, and pipes
* Implement services and leverage Angular dependency injection system
* Configure client-side routing, navigation, guards, and resolvers
* Build reactive forms and template-driven forms with validation
* Communicate with backend services using HttpClient and interceptors
* Work with `RxJS` observables for reactive data handling
* Manage application state using NgRx
* Test Angular applications with Jasmine, Karma, and Cypress
* Optimize performance with lazy loading and production builds
* Style applications using `Angular Material`

## Outline
<!-- chapter: angular-architecture-and-setup, duration: 2h -->
* Angular Architecture and Setup
    * Overview of the Angular framework and its philosophy
    * Angular application architecture and building blocks
    * Setting up the development environment with `Angular CLI`
    * Creating, serving, and building an Angular project
    * Understanding the project structure and configuration files
    * Workspaces, libraries, and monorepo support
<!-- chapter: typescript-refresher, duration: 2h -->
* `TypeScript` Refresher
    * Types, interfaces, and type aliases
    * Classes, access modifiers, and inheritance
    * Generics and utility types
    * Decorators and metadata
    * Enums, union types, and type narrowing
<!-- chapter: components-and-templates, duration: 2h -->
* Components and Templates
    * Creating components with `Angular CLI`
    * Component decorator, metadata, and lifecycle hooks
    * Template syntax and template expressions
    * Content projection with ng-content
    * View encapsulation and component styles
    * Component interaction: @Input, @Output, and EventEmitter
    * Standalone components and their benefits
<!-- chapter: data-binding, duration: 2h -->
* Data Binding
    * Interpolation binding
    * Property binding and attribute binding
    * Event binding and event handling
    * Two-way binding with ngModel
    * Binding best practices and change detection strategy
<!-- chapter: directives, duration: 2h -->
* Directives
    * Structural directives: *ngIf, *ngFor, *ngSwitch
    * The new @if, @for, @switch control flow syntax
    * Attribute directives: ngClass, ngStyle
    * Building custom structural and attribute directives
    * Host binding and host listening
<!-- chapter: pipes, duration: 1h -->
* Pipes
    * Built-in pipes: date, currency, percent, async, and more
    * Creating custom pipes
    * Pure vs impure pipes
    * Chaining pipes and parameterized pipes
<!-- chapter: services-and-dependency-injection, duration: 2h -->
* Services and Dependency Injection
    * Creating and providing services
    * The Angular dependency injection system
    * Hierarchical injectors and provider scopes
    * providedIn and tree-shakable providers
    * Injection tokens and factory providers
    * Using services for cross-component communication
<!-- chapter: modules-and-application-structure, duration: 2h -->
* Modules and Application Structure
    * NgModules and their role in Angular
    * Feature modules and shared modules
    * Standalone components vs module-based architecture
    * Organizing a large-scale Angular application
<!-- chapter: routing-and-navigation, duration: 3h -->
* Routing and Navigation
    * Setting up the `Angular Router`
    * Route configuration, path parameters, and query parameters
    * Child routes and nested routing
    * Router outlet and named outlets
    * Programmatic navigation
    * Route guards: CanActivate, CanDeactivate, CanLoad
    * Resolvers for pre-fetching data
    * Lazy loading modules and routes
<!-- chapter: forms, duration: 2h -->
* Forms
    * Template-driven forms with ngModel
    * Reactive forms with FormGroup, FormControl, and FormArray
    * Form validation: built-in and custom validators
    * `Async` validators and cross-field validation
    * Dynamic forms and form builder
    * Error messages and user feedback
<!-- chapter: http-client-and-backend-communication, duration: 2h -->
* `HTTP` Client and Backend Communication
    * Setting up HttpClient and making requests
    * Handling GET, POST, PUT, DELETE operations
    * Request and response interceptors
    * Error handling and retry strategies
    * Typed responses and response transformation
    * `File` upload handling
<!-- chapter: rxjs-and-reactive-programming, duration: 2h -->
* `RxJS` and Reactive Programming
    * Introduction to observables and the observer pattern
    * Core `RxJS` operators: map, filter, switchMap, mergeMap, concatMap
    * Subjects and multicasting
    * Combining observables: combineLatest, forkJoin, `zip`
    * Error handling with catchError and retry
    * Unsubscribing and memory leak prevention
<!-- chapter: state-management-with-ngrx, duration: 2h -->
* State Management with NgRx
    * Introduction to the `Redux` pattern and NgRx
    * Store, actions, reducers, and selectors
    * Effects for handling side effects
    * Entity adapter for collection management
    * NgRx DevTools for debugging
<!-- chapter: testing-angular-applications, duration: 2h -->
* Testing Angular Applications
    * Unit testing with Jasmine and Karma
    * Testing components, services, and pipes
    * Mocking dependencies and `HTTP` requests
    * Testing forms and routing
    * End-to-end testing with Cypress
    * Test coverage and best practices
<!-- chapter: angular-material-and-ui-components, duration: 2h -->
* `Angular Material` and UI Components
    * Setting up `Angular Material` in a project
    * Common components: buttons, cards, dialogs, tables, and forms
    * Theming and customization
    * Accessibility considerations
    * CDK (Component Dev Kit) utilities
<!-- chapter: deployment-and-production-optimization, duration: 2h -->
* Deployment and Production Optimization
    * Production builds and ahead-of-time compilation
    * Bundle analysis and tree shaking
    * Lazy loading for performance
    * Environment configuration and feature flags
    * Deployment strategies: `Docker`, cloud platforms, static hosting
    * Performance monitoring and optimization techniques

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
