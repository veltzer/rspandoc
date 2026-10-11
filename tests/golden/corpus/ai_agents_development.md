---
tags:
  - data-and-ai:agents
  - data-and-ai:llm
  - data-and-ai:ai
  - languages:python
level: intermediate
category: ai
duration_hours: 16
audience:
  - audiences:developers
  - audiences:data-engineers
  - audiences:architects
---

<!-- course: ai_agents_development -->
# `AI` Agents Development

## Description
This two day course covers the design and implementation of autonomous `AI` agents powered by large language models. Students will learn to build agents that can reason, plan, use tools, and collaborate in multi-agent systems using modern frameworks such as `LangGraph`, `CrewAI`, and `AutoGen`. The course addresses agent memory, state management, evaluation strategies, guardrails, and production deployment.

## Duration
16 hours / 2 days

## Intended Audience
* Software developers building `AI`-powered applications
* Data engineers integrating `AI` agents into pipelines
* Software architects designing agent-based systems

## Prerequisites
* Proficiency in `Python` programming
* Basic understanding of `LLMs` and their APIs
* Familiarity with prompt engineering concepts

## Required Knowledge
* `Python` Programming (or equivalent experience)

## Objectives
* Understand the architecture and `design patterns` of `AI` agents.
* Build autonomous agents capable of reasoning, planning, and tool use.
* Implement multi-agent systems with coordination and communication.
* Apply guardrails and evaluation techniques to ensure agent reliability.
* Deploy agents to production environments.

## Outline
<!-- chapter: introduction-to-ai-agents, duration: 2h -->
* Introduction to `AI` Agents
    * What are `AI` agents and why they matter
    * Agent architectures: `ReAct`, plan-and-execute, reflexion
    * Single agent vs. multi-agent paradigms
    * Overview of the agent ecosystem
<!-- chapter: tool-use-and-function-calling, duration: 2h -->
* Tool Use and Function Calling
    * Defining and registering tools
    * Function calling with `OpenAI`, Anthropic, and open-source models
    * Tool selection and error handling
    * Building custom tool integrations
<!-- chapter: agent-frameworks, duration: 2h -->
* Agent Frameworks
    * Building agents with `LangGraph`
    * Orchestrating teams with `CrewAI`
    * Multi-agent conversations with `AutoGen`
    * Comparing frameworks and choosing the right one
<!-- chapter: memory-and-state-management, duration: 2h -->
* Memory and State Management
    * Short-term vs. long-term memory
    * Conversation history and context `windows`
    * Persistent state with vector stores and databases
    * Stateful workflows and checkpointing
<!-- chapter: multi-agent-systems, duration: 2h -->
* Multi-Agent Systems
    * Agent communication protocols
    * Task decomposition and delegation
    * Supervisor and hierarchical patterns
    * Collaborative problem-solving
<!-- chapter: evaluation-and-testing, duration: 2h -->
* Evaluation and Testing
    * Evaluating agent performance
    * Benchmarking and metrics
    * Tracing and observability
    * Regression testing for agents
<!-- chapter: guardrails-and-safety, duration: 2h -->
* Guardrails and Safety
    * Input and output validation
    * Content filtering and moderation
    * Preventing prompt injection in agents
    * Human-in-the-loop approval workflows
<!-- chapter: production-deployment, duration: 2h -->
* Production Deployment
    * Scaling agent workloads
    * `Async` and streaming execution
    * Monitoring and logging in production
    * Cost management and optimization

## Copyright
Mark Veltzer [mark.veltzer@gmail.com](mailto:mark.veltzer@gmail.com), © 2026
