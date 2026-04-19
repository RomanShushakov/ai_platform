# Infra Lab: Slurm + Ansible + Apptainer

This directory contains the infrastructure track of the AI Platform Lab.

The goal here is not only to build an application, but to practice how AI-related systems can be deployed, scheduled, automated, and operated across small heterogeneous hardware.

---

## Purpose

The main Rust AI platform remains the application/control-plane part of the project.

This `infra/` directory is for the infrastructure side:

- cluster setup
- automation
- scheduling
- portable runtime environments
- later integration with the Rust platform

The focus is intentionally practical and incremental.

---

## Current Direction

The next major phase of the project is:

- **Slurm first**
- **Ansible-managed setup**
- **Apptainer later for portable job environments**
- **k3s later, not now**

---

## Planned Topology

### Roles

- **Laptop**
  - development machine
  - Git / coding / testing
  - Ansible control node
  - SSH client
  - not part of the Slurm cluster for now

- **Raspberry Pi 4**
  - Slurm controller
  - runs `slurmctld`
  - cluster management node

- **Jetson Orin Nano**
  - Slurm worker node
  - runs `slurmd`
  - GPU-capable compute node

---

## Why this topology

- laptop = development
- Raspberry Pi = controller
- Jetson = compute node

---

## Why Slurm First

- batch execution
- scheduled jobs
- GPU-aware resource management
- HPC-style workflows

---

## Why Ansible

- repeatable setup
- infra as code
- less error-prone
- production-relevant skills

---

## Why Apptainer Later

- portable environments
- reproducibility
- HPC alignment

---

## Planned Directory Layout

```text
infra/
  README.md
  ansible/
    inventory/
    group_vars/
    playbooks/
    roles/
  slurm/
    configs/
    job-examples/
  apptainer/
    definitions/
    notes/
  notes/
```

---

## Implementation Plan

### Phase 1 — Platform Baseline

- Rust AI platform working
- RAG working
- MCP working
- Docker + local mode working

### Phase 2 — Infra Baseline

- define inventory
- bootstrap playbooks
- connectivity setup

### Phase 3 — Minimal Slurm Cluster

- Pi = controller
- Jetson = worker
- test jobs:
  - hostname
  - sleep
  - simple script

### Phase 4 — GPU Experiments

- partitions
- GPU jobs
- ML scripts

### Phase 5 — Apptainer

- containerized jobs
- reproducible environments

### Phase 6 — Integration

- Slurm job submission from Rust platform
- batch embeddings
- evaluation jobs

---

## Immediate Next Goal

Create:

- initial Ansible structure
- inventory for Pi + Jetson

---

## Notes

- keep steps small
- test each layer
- prefer reproducibility
- avoid overengineering
- keep notes in repo
- laptop stays outside cluster
