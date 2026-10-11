---
tags:
  - data-and-ai:ai
  - data-and-ai:agents
  - data-and-ai:mcp
  - data-and-ai:rag
  - practices:tools
  - practices:large-codebases
  - data-and-ai:prompt-engineering
  - practices:productivity
level: advanced
category: ai
duration_hours: 40
audience:
  - audiences:developers
  - audiences:architects
  - audiences:team-leads
  - audiences:devops
---

<!-- course: advanced_ai_powered_development -->
# Advanced `AI`-Powered Development

## Description
`AI`-powered development has rapidly evolved beyond simple code completion into a rich ecosystem of
agents, tools, skills, `MCP` servers, `RAG` pipelines, and autonomous coding workflows. Modern developers
must understand how to harness these capabilities to work effectively with large codebases, monoliths,
and complex systems. This course provides deep, hands-on coverage of the full `AI` development stack
as it exists today, from terminal-based `AI` coding agents to building custom `MCP` servers and
`RAG`-augmented development workflows.

The course is designed for experienced developers who already use basic `AI` assistance and want to
move to the next level: working with agentic tools, managing context in massive codebases, building
custom integrations, and understanding the architecture behind modern `AI` development infrastructure.

All topics reflect the state of the art as of early 2026, covering the latest tools, protocols, and
patterns that are actively used in production environments.

## Duration
40 hours / 5 days

## Intended Audience
* Senior developers who want to master advanced `AI`-powered development workflows
* Software architects evaluating `AI` tool integration strategies for large systems
* Tech leads establishing `AI` development practices for their teams
* `DevOps` engineers seeking `AI`-powered automation for complex infrastructure
* Developers working with large codebases or monoliths who need effective `AI` strategies

## Prerequisites
* `Solid` experience in at least one modern programming language (`Python`, `JavaScript`/`TypeScript`, `Java`, Go, or similar)
* Working knowledge of `Git`, `CI/CD`, and modern development workflows
* Basic experience using `AI` coding assistants (e.g., `GitHub Copilot`, `ChatGPT`, Claude)
* Familiarity with APIs, `REST`, and `JSON`
* Understanding of software architecture concepts (`microservices`, monoliths, APIs)

## Required Knowledge
* `Python` Programming (or equivalent experience)
* `HTML` `CSS` `JavaScript` Fundamentals (or equivalent experience)
* `TypeScript` Programming (or equivalent experience)

## Objectives
* Effectively use agentic `AI` coding tools to navigate, understand, and modify large codebases
* Build and deploy custom `MCP` servers to extend `AI` assistants with project-specific capabilities
* Design and implement `RAG` pipelines to give `AI` tools deep knowledge of proprietary codebases and documentation
* Understand and apply the tool-use, skills, and agent patterns in modern `AI` development
* Develop strategies for `AI`-assisted work on monoliths and large-scale systems
* Evaluate and integrate the latest `AI` development tools and frameworks
* Build custom agents for specialized development tasks

## Outline
<!-- chapter: the-modern-ai-development-landscape, duration: 2h -->
* The Modern `AI` Development Landscape
    * Evolution from code completion to agentic development
    * Categories of `AI` development tools
        * Inline completion engines
        * Chat-based assistants
        * Terminal-based agents
        * IDE-integrated agents
        * Autonomous coding agents
    * Key concepts: context `windows`, token limits, and their practical implications
    * The role of models: Claude, `GPT`, Gemini, open-source models
    * Cloud-hosted vs local models for development
    * Cost, latency, and quality tradeoffs
    * Privacy and intellectual property considerations
<!-- chapter: ai-coding-agents, duration: 3h -->
* `AI` Coding Agents
    * What makes an agent different from a chatbot
    * Agent architectures: plan-execute, `ReAct`, tool-use loops
    * Popular agent-based tools
        * `Claude Code` (`CLI` agent)
        * Cursor (agent mode)
        * `Windsurf`
        * `Aider`
        * `Cline`
        * `Copilot Workspace`
    * Agent capabilities
        * `File` reading and editing
        * Running shell commands
        * Searching codebases
        * Running tests and interpreting results
        * `Git` operations
        * Multi-step task execution
    * Working with agent permissions and safety models
    * Configuring agents for your project (.`claude`/, `CLAUDE`.md, rules files)
    * Giving agents project context and conventions
    * Comparing agent effectiveness across tasks
<!-- chapter: working-with-large-codebases-and-monoliths, duration: 4h -->
* Working with Large Codebases and Monoliths
    * Challenges of `AI` in large codebases
        * Context window limitations
        * Navigating unfamiliar code
        * Understanding implicit conventions
        * Cross-module dependencies
    * Strategies for effective `AI` use in monoliths
        * Structured project documentation for `AI` consumption
        * Writing effective `CLAUDE`.md / rules files
        * Modular context feeding
        * Incremental exploration patterns
    * Code navigation with `AI` agents
        * Searching by pattern and semantic meaning
        * Understanding call graphs and dependencies
        * Tracing data flow across modules
    * Refactoring monoliths with `AI` assistance
        * Identifying extraction candidates
        * Safe, incremental refactoring workflows
        * Maintaining test coverage during refactoring
    * `AI`-assisted code archaeology
        * Understanding legacy code
        * Documenting tribal knowledge
        * Mapping undocumented APIs and protocols
<!-- chapter: tool-use-and-function-calling, duration: 4h -->
* Tool Use and Function Calling
    * The tool-use pattern in `LLMs`
        * How `LLMs` invoke external tools
        * Tool definitions and schemas
        * Tool result handling
        * Multi-turn tool-use conversations
    * Designing effective tools for `AI` agents
        * Tool granularity and composability
        * Input/output schema design
        * Error handling and feedback
        * Idempotency and safety
    * Built-in tools in popular agents
        * `File` operations (read, write, edit, glob, `grep`)
        * Shell execution
        * Web search and fetch
        * LSP integration
    * Building custom tools
        * Tool definition formats
        * Implementing tool handlers
        * Testing tools with `AI` agents
        * Versioning and maintaining tools
<!-- chapter: mcp-model-context-protocol, duration: 4h -->
* `MCP` (Model Context Protocol)
    * What is `MCP` and why it matters
        * The problem `MCP` solves
        * `MCP` architecture: hosts, clients, servers
        * Transport mechanisms (stdio, `SSE`, streamable `HTTP`)
    * `MCP` capabilities
        * Tools: actions the `AI` can perform
        * Resources: data the `AI` can read
        * Prompts: reusable prompt templates
        * Sampling: letting servers request `LLM` completions
    * Using existing `MCP` servers
        * Filesystem server
        * `GitHub` server
        * Database servers (`PostgreSQL`, `SQLite`)
        * Slack, `Jira`, Linear servers
        * Browser automation servers (`Puppeteer`, `Playwright`)
    * Building custom `MCP` servers
        * Server implementation in `Python` and `TypeScript`
        * Exposing project-specific tools and resources
        * Authentication and authorization
        * Deployment and lifecycle management
    * `MCP` in team environments
        * Shared `MCP` server configurations
        * Central tool registries
        * Security policies and access control
<!-- chapter: skills-and-slash-commands, duration: 3h -->
* Skills and Slash Commands
    * What are skills in `AI` development tools
    * Built-in skills vs custom skills
    * Creating custom skills
        * Skill definition and structure
        * Parameterized skills
        * Skill composition and chaining
    * Practical skill examples
        * Code review skills
        * Commit message generation
        * PR creation workflows
        * Deployment skills
        * Database migration skills
    * Sharing skills across teams
    * Hooks: automating actions around tool calls
        * Pre and post-execution hooks
        * Notification hooks
        * Validation and guard hooks
<!-- chapter: rag-retrieval-augmented-generation-for-development, duration: 5h -->
* `RAG` (Retrieval-Augmented Generation) for Development
    * Why `RAG` matters for development
        * Overcoming context window limits
        * Grounding `AI` in project-specific knowledge
        * Reducing hallucinations
    * `RAG` architecture for codebases
        * Document chunking strategies for code
        * Embedding models for source code
        * Vector databases (Pinecone, `Weaviate`, `ChromaDB`, `Qdrant`)
        * Hybrid search: vector + keyword
    * Indexing codebases
        * `File`-level vs function-level vs chunk-level indexing
        * Handling multiple languages and file types
        * Incremental indexing and keeping indexes fresh
        * Metadata extraction and enrichment
    * `RAG` for documentation
        * Indexing internal wikis, READMEs, design docs
        * Indexing `API` documentation
        * Indexing Slack/Teams conversations and decision logs
    * Building a `RAG` pipeline
        * Ingestion pipeline design
        * Query construction and rewriting
        * Retrieval and re-ranking
        * Context `assembly` and prompt construction
    * `RAG` evaluation and optimization
        * Measuring retrieval quality
        * Measuring answer quality
        * Iterating on chunking and embedding strategies
<!-- chapter: prompt-engineering-for-advanced-workflows, duration: 2h -->
* Prompt Engineering for Advanced Workflows
    * System prompts and persona design
    * Chain-of-thought and step-by-step reasoning
    * Few-shot and many-shot prompting
    * Structured output: `JSON` mode, schemas, and constrained generation
    * Prompt chaining and multi-stage pipelines
    * Prompt templating and reuse
    * Context management strategies
        * Priority-based context `assembly`
        * Summarization and compression
        * Sliding window approaches
    * Evaluating and iterating on prompts
    * Prompt security: injection attacks and defenses
<!-- chapter: building-custom-agents, duration: 5h -->
* Building Custom Agents
    * Agent `design patterns`
        * Single-turn vs multi-turn agents
        * Supervisor and sub-agent architectures
        * Parallel agent execution
        * Human-in-the-loop patterns
    * Agent frameworks
        * `Anthropic Agent SDK`
        * `LangChain` / `LangGraph`
        * `CrewAI`
        * `AutoGen`
        * `OpenAI Agents SDK`
    * Implementing development agents
        * Code review agents
        * Test generation agents
        * Documentation agents
        * Migration agents
        * Monitoring and alerting agents
    * Agent memory and state
        * Short-term conversation memory
        * Long-term persistent memory
        * Project knowledge bases
        * Learning from corrections
    * Agent evaluation and testing
        * Benchmarking agent performance
        * Regression testing for agents
        * Monitoring agent behavior in production
<!-- chapter: ai-in-the-development-lifecycle, duration: 4h -->
* `AI` in the Development Lifecycle
    * `AI`-assisted planning and design
        * Requirements analysis
        * Architecture exploration
        * `API` design
        * Data modeling
    * `AI`-powered code review
        * Automated review agents
        * Style and convention enforcement
        * Security scanning
        * Performance analysis
    * `AI`-assisted testing
        * Unit test generation
        * Integration test design
        * Fuzzing and property-based testing
        * Visual regression testing
    * `AI` in `CI/CD` pipelines
        * Automated PR checks with `AI`
        * Intelligent test selection
        * Deployment risk analysis
        * Automated rollback decisions
    * `AI` for incident response
        * Log analysis and anomaly detection
        * Root cause analysis
        * Remediation suggestion
        * Post-mortem generation
<!-- chapter: advanced-topics, duration: 4h -->
* Advanced Topics
    * Multi-model strategies
        * Using different models for different tasks
        * Model routing and fallback
        * Cost optimization across models
    * Fine-tuning for development tasks
        * `When` to fine-tune vs prompt engineer
        * Creating training datasets from codebases
        * Evaluating fine-tuned models
    * Local models for development
        * Running models locally with Ollama, `LM Studio`, llama.cpp
        * Privacy-sensitive development workflows
        * Offline development with `AI`
        * Hardware requirements and performance
    * `AI` governance and compliance
        * Code ownership and `AI`-generated code
        * License implications
        * Audit trails and reproducibility
        * Organizational policies
    * Scaling `AI` development practices
        * Team onboarding and training
        * Standardizing tool configurations
        * Measuring and improving `AI` adoption
        * Building internal `AI` development platforms

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
