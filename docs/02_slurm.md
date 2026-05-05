# 🧮 Slurm Cluster Setup (Raspberry + Jetson)

This guide sets up a **distributed Slurm cluster** using Ansible.

Cluster topology:

* 🧠 Raspberry → controller
* ⚡ Jetson → worker (GPU node)

---

# 🎯 Goal

By the end of this guide, you will have:

* Slurm controller + worker running
* job scheduling working
* GPU scheduling (GRES) enabled
* shared storage via NFS

---

# 🧭 Setup Flow

```text
Bootstrap → Identity → Build → Munge → Config → Start → Jobs → GPU → NFS
```

---

# 0️⃣ Gather Facts

Collect system information from all nodes:

```bash id="slurm-0"
ansible-playbook slurm/playbooks/gather_facts.yml
```

---

# 1️⃣ Bootstrap Nodes

Prepare base system (packages, users, environment):

```bash id="slurm-1"
ansible-playbook slurm/playbooks/bootstrap.yml --check -K
ansible-playbook slurm/playbooks/bootstrap.yml -K
```

Rollback (if needed):

```bash id="slurm-1-rollback"
ansible-playbook slurm/playbooks/bootstrap_rollback.yml --check -K
ansible-playbook slurm/playbooks/bootstrap_rollback.yml -K
```

---

# 2️⃣ Configure Slurm Identity

Setup users, permissions, and shared identity:

```bash id="slurm-2"
ansible-playbook slurm/playbooks/slurm_identity.yml --check -K
ansible-playbook slurm/playbooks/slurm_identity.yml -K
ansible-playbook slurm/playbooks/slurm_identity_verify.yml -K
```

Rollback:

```bash id="slurm-2-rollback"
ansible-playbook slurm/playbooks/slurm_identity_rollback.yml --check -K
ansible-playbook slurm/playbooks/slurm_identity_rollback.yml -K
```

---

# 3️⃣ Install Build Prerequisites

Install dependencies for building Slurm:

```bash id="slurm-3"
ansible-playbook slurm/playbooks/slurm_build_prereqs.yml --check -K
ansible-playbook slurm/playbooks/slurm_build_prereqs.yml -K
ansible-playbook slurm/playbooks/slurm_build_prereqs_verify.yml -K
```

Rollback:

```bash id="slurm-3-rollback"
ansible-playbook slurm/playbooks/slurm_build_prereqs_rollback.yml --check -K
ansible-playbook slurm/playbooks/slurm_build_prereqs_rollback.yml -K
```

---

# 4️⃣ Build and Install Slurm

Compile and install Slurm from source:

```bash id="slurm-4"
ansible-playbook slurm/playbooks/slurm_build.yml -K
ansible-playbook slurm/playbooks/slurm_build_verify.yml -K
```

Verify manually:

```bash id="slurm-4-check"
ansible all -b -m shell -a "dpkg -l | egrep 'munge|slurm-wlm'" -K
ansible all -b -m shell -a "systemctl list-unit-files | egrep 'munge|slurm'" -K
```

Rollback:

```bash id="slurm-4-rollback"
ansible-playbook slurm/playbooks/slurm_build_rollback.yml -K
```

---

# 5️⃣ Setup Munge (Authentication)

Munge is required for secure Slurm communication.

```bash id="slurm-5"
ansible-playbook slurm/playbooks/munge_key_setup.yml -K
ansible-playbook slurm/playbooks/munge_key_verify.yml -K
```

Rollback:

```bash id="slurm-5-rollback"
ansible-playbook slurm/playbooks/munge_key_rollback.yml -K
```

---

# 6️⃣ Configure Systemd Services

Install and configure Slurm systemd units:

```bash id="slurm-6"
ansible-playbook slurm/playbooks/slurm_systemd_units.yml -K
```

Rollback:

```bash id="slurm-6-rollback"
ansible-playbook slurm/playbooks/slurm_systemd_units_rollback.yml -K
```

---

# 7️⃣ Apply Slurm Configuration

Deploy cluster configuration:

```bash id="slurm-7"
ansible-playbook slurm/playbooks/slurm_config.yml -K
```

Rollback:

```bash id="slurm-7-rollback"
ansible-playbook slurm/playbooks/slurm_config_rollback.yml -K
```

---

# 8️⃣ Start Slurm Cluster

Start controller and worker services:

```bash id="slurm-8"
ansible-playbook slurm/playbooks/slurm_start.yml -K
```

Rollback:

```bash id="slurm-8-rollback"
ansible-playbook slurm/playbooks/slurm_start_rollback.yml -K
```

---

## 🔍 Verify Cluster (on Raspberry)

```bash id="slurm-verify"
 /opt/slurm/bin/scontrol ping
 /opt/slurm/bin/sinfo
 /opt/slurm/bin/srun hostname
```

Expected:

* controller responds
* worker node visible
* job runs successfully

---

# 9️⃣ Run Example Jobs

Deploy test jobs:

```bash id="slurm-9"
ansible-playbook slurm/playbooks/slurm_job_examples.yml -K
```

Submit job (on Raspberry):

```bash id="slurm-job"
cd ~/workdir/slurm/job-examples
/opt/slurm/bin/sbatch test.sbatch
/opt/slurm/bin/scontrol show job <JOB_ID>
/opt/slurm/bin/squeue
```

Check output (on Jetson):

```bash id="slurm-job-output"
cd ~/workdir/slurm/job-examples
ls *.out
```

Rollback:

```bash id="slurm-9-rollback"
ansible-playbook slurm/playbooks/slurm_job_examples_rollback.yml -K
```

---

# 🔟 Enable GPU Scheduling (GRES)

Configure GPU support:

```bash id="slurm-10"
ansible-playbook slurm/playbooks/slurm_gres.yml -K
ansible-playbook slurm/playbooks/slurm_gres_restart.yml -K
ansible-playbook slurm/playbooks/slurm_gres_verify.yml -K
```

Test GPU job:

```bash id="slurm-gpu-test"
cd ~/workdir/slurm/job-examples
/opt/slurm/bin/sbatch gpu_probe_gres.sbatch
```

Check output on Jetson.

Rollback:

```bash id="slurm-10-rollback"
ansible-playbook slurm/playbooks/slurm_gres_rollback.yml -K
```

---

# 1️⃣1️⃣ Setup NFS (Shared Storage)

Configure shared storage:

```bash id="slurm-11"
ansible-playbook slurm/playbooks/nfs_server.yml -K
ansible-playbook slurm/playbooks/nfs_client.yml -K
ansible-playbook slurm/playbooks/nfs_verify.yml -K
```

---

## 🔍 Verify NFS

### On Raspberry:

```bash id="nfs-check-rasp"
mount | grep /home/roman/nfs
touch /home/roman/nfs/nfs-test-from-raspberry.txt
ls -l /home/roman/nfs
```

### On Jetson:

```bash id="nfs-check-jetson"
ls -l /home/roman/nfs
cat /home/roman/nfs/nfs-test-from-raspberry.txt
```

---

## 🧪 Run Jobs via NFS

```bash id="nfs-jobs"
cd ~/nfs/slurm/job-examples
/opt/slurm/bin/sbatch hostname.sbatch
sleep 2
ls -la
cat slurm-*.out
```

Rollback:

```bash id="slurm-11-rollback"
ansible-playbook slurm/playbooks/nfs_client_rollback.yml -K
ansible-playbook slurm/playbooks/nfs_server_rollback.yml -K
```

---

# 1️⃣2️⃣ NFS-based Job Examples

Deploy job examples directly on NFS:

```bash id="slurm-12"
ansible-playbook slurm/playbooks/slurm_job_examples_nfs.yml -K
```

Run:

```bash id="slurm-nfs-job"
cd ~/nfs/slurm/job-examples
ls
/opt/slurm/bin/sbatch hostname.sbatch
sleep 2
cat slurm-*.out
```

Rollback:

```bash id="slurm-12-rollback"
ansible-playbook slurm/playbooks/slurm_job_examples_nfs_rollback.yml -K
```

---

# 📌 Summary

You now have a fully working **distributed Slurm cluster**:

* controller + worker ✔
* job scheduling ✔
* GPU scheduling ✔
* shared storage via NFS ✔

This enables:

👉 batch pipelines for RAG, embeddings, and training
