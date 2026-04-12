# Architecture Overview

## Components

* Rust Host (API + orchestration)
* MCP Tools Server
* LLM Backend (Ollama / vLLM)
* Retrieval Layer (RAG)

## Flow

User → Host → LLM → Tools / Retrieval → Response

## Notes

* System is modular
* Backends are configurable via environment variables
