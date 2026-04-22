# 🖥 Infra Track: Slurm + Shared Storage + Future K3s

This directory tracks the infrastructure side of the AI platform project.

The goal is not just to run a chatbot, but to understand how:

- online serving
- batch compute
- shared storage
- deployment platforms

fit together in a small real lab.

---

# 🎯 Current Infra Direction

We use a split architecture:

- **Rust AI platform** = online serving/orchestration plane
- **Slurm cluster** = offline/batch compute plane
- **shared storage (next step: NFS)** = bridge between them
- **future K3s** = always-on application deployment plane

This lets us keep the online app simple and responsive, while moving heavier jobs into scheduled batch execution.

---

# 🧩 Roles of Each Layer

## 1. Rust AI Platform

The Rust app is the main serving layer.

It handles:

- chat/API requests
- tool orchestration
- RAG query-time retrieval
- model backend integration
- future admin workflows

It can run:

- locally
- in Docker Compose
- later in K3s

## 2. Slurm

Slurm is used for offline and heavier jobs such as:

- rebuilding RAG artifacts
- embedding/index generation
- LoRA/QLoRA training
- evaluation runs
- preprocessing / conversion jobs

Slurm is **not** replacing the app.  
It is the compute backend for batch work.

## 3. Shared Storage (Jetson-based NFS)

Shared storage will be provided by the Jetson node via NFS.

Rationale:

- Jetson has significantly larger disk (256GB vs 64GB on Raspberry)
- Slurm jobs execute on Jetson, so writes stay local (better performance)
- avoids network write bottlenecks
- more realistic small-lab architecture

The Raspberry Pi will mount the same path via NFS.

Planned shared path:

- `/home/roman/workdir` (same path on both nodes)

This ensures:

- identical paths across cluster
- no path translation in Slurm jobs
- easy integration with Docker/K3s later

# 📦 Storage Layout (Planned)

Shared NFS directory (Jetson):

- `/home/roman/workdir/slurm` → job scripts
- `/home/roman/workdir/rag` → RAG artifacts
- `/home/roman/workdir/models` → LoRA/adapters/checkpoints
- `/home/roman/workdir/logs` → job logs

This directory will be mounted:

- on Raspberry (Slurm controller)
- later into Docker containers
- later into K3s pods

## 4. Future K3s

K3s will later run the always-on application side:

- Rust host/API
- tools server
- maybe UI
- possibly inference services

K3s will mount shared storage so the app can read artifacts and model outputs produced by Slurm jobs.

---

# 🧠 Retrieval Strategy Decision

For now, **JSON RAG artifacts are fully enough**.

Current approach:

- offline chunking
- offline embeddings generation
- JSON artifacts:
  - `chunks.json`
  - `embeddings.json`
  - `manifest.json`

Why this is enough now:

- very small scale
- easy to inspect and debug
- simple versioning
- no extra infrastructure required

A vector database such as Qdrant may be added later, but it is **not needed yet**.

Planned progression:

- **now**: JSON artifact-based retrieval
- **later**: optional Qdrant backend if online vector search becomes necessary

---

# 🖥 Current Hardware Topology

## Nodes

- **Laptop**
  - development machine
  - Ansible control node
  - curl / testing client
  - future local UI access

- **Raspberry Pi 4**
  - Slurm controller
  - NFS client
  - likely future K3s server/control-plane

- **Jetson Orin Nano**
  - Slurm worker
  - GPU execution node
  - NFS server (shared storage)
  - likely future K3s worker

---

# ✅ Current Working Milestone

We successfully built and ran a minimal Slurm cluster with GPU-aware scheduling.

## What works now

### Slurm cluster
- same Slurm version built from source on both nodes
- custom install under `/opt/slurm`
- custom systemd units
- controller on Raspberry
- worker on Jetson

### Identity
- explicit `munge` and `slurm` users/groups
- matching UID/GID across nodes

### Munge
- package-based Munge installation
- shared `munge.key`
- successful auth

### Slurm config
- controller and worker communicate correctly
- `scontrol ping` works
- `sinfo` works
- `srun hostname` runs on Jetson

### GPU-aware scheduling
- `GresTypes=gpu`
- Jetson advertised with `Gres=gpu:1`
- `gpu_probe_gres.sbatch` runs successfully
- job confirms:
  - execution on Jetson
  - visible NVIDIA device nodes
  - working `tegrastats`
  - Slurm GRES request honored

---

# ⚠ Current Limitation Discovered

Batch job outputs are currently written on the execution node’s local filesystem.

Example:

- job submitted from Raspberry
- job runs on Jetson
- output file appears on Jetson local disk

This is expected without shared storage.

This is the main reason the next step is NFS.

---

# 🧪 Current Example Jobs

Current job examples include:

- `hostname.sbatch`
- `sleep.sbatch`
- `python_version.sbatch`
- `gpu_probe.sbatch`
- `gpu_probe_gres.sbatch`

These validate:

- scheduler basics
- batch execution
- worker execution
- GPU visibility
- GRES-based GPU scheduling

---

# 🧭 Planned Architecture

## Online path
User request → Rust app → retrieval/tools/model backend → response

## Offline path
Operator or future app trigger → Slurm job → artifacts/models/logs written to shared storage → app consumes results

Example future offline jobs:

- RAG rebuild
- embedding refresh
- LoRA fine-tuning
- evaluation pipeline

---

# 🛠 Future Batch Execution Model

## RAG rebuild
A Slurm job will later run a batch artifact generation flow, likely containerized.

Possible pattern:

- input markdown/docs from shared storage
- batch job runs indexer
- outputs new artifact version to shared storage
- app reloads or switches to new artifact set

## LoRA / fine-tuning
A Slurm job will later run a training container or batch environment and write:

- adapters/checkpoints
- logs
- metrics
- eval results

to shared storage.

The app layer can later load or expose those outputs.

---

# 📦 Container Direction

## Application side
- Docker Compose now / K3s later

## Batch side
- likely Apptainer for Slurm jobs later

This gives a realistic split:

- K3s for always-on services
- Slurm for scheduled compute jobs

---

# 🚀 Next Roadmap

## 1. Freeze current Slurm/GPU milestone
Document current state and known limitations.

## 2. Install NFS
Use Raspberry as NFS server and Jetson as NFS client.

Goal:
- shared `/home/roman/workdir` (or another chosen shared path)

## 3. Re-run Slurm jobs with shared storage
Confirm:
- job scripts visible from both nodes
- output files visible from Raspberry immediately

## 4. Put the Rust app on K3s
Run the always-on app stack on Raspberry/Jetson via K3s.

## 5. Mount NFS into the app
App reads:
- RAG artifacts
- model outputs
- logs
- future checkpoints

## 6. Optional later step: Qdrant
Add Qdrant only if JSON artifacts become too static or limiting.

---

# 🧠 Design Principles

We intentionally keep scope sane.

## Current principles
- Rust remains the backbone
- offline JSON RAG artifacts are enough
- Slurm handles batch/offline work
- K3s is for always-on service deployment later
- shared storage is the bridge between the two planes
- avoid unnecessary cloud complexity
- prefer explicit, understandable infra steps
- prefer Ansible-managed reproducible configuration

---

# 📌 Summary

Current status:

- Slurm cluster works
- GPU-aware scheduling works
- offline compute path is real
- shared storage is the next missing piece
- K3s is the next platform step after NFS
- JSON RAG artifacts remain the right choice for now
