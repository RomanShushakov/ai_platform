#!/bin/bash
#SBATCH --job-name=gpu-probe
#SBATCH --output=/home/roman/workdir/slurm/job-examples/slurm-%j.out
#SBATCH --partition=debug

set -x

echo "=== hostname ==="
hostname

echo "=== user ==="
whoami

echo "=== date ==="
date

echo "=== kernel ==="
uname -a

echo "=== nvidia device nodes ==="
ls -l /dev/nvhost* /dev/nvmap /dev/nvidia* 2>/dev/null || true

echo "=== tegrastats path ==="
which tegrastats || true

echo "=== tegrastats sample ==="
timeout 3 tegrastats || true

echo "=== python ==="
python3 --version || true

echo "=== torch cuda probe ==="
python3 - <<'PY'
try:
    import torch
    print("torch_version=", torch.__version__)
    print("cuda_available=", torch.cuda.is_available())
    if torch.cuda.is_available():
        print("device_count=", torch.cuda.device_count())
        print("device_name=", torch.cuda.get_device_name(0))
except Exception as e:
    print("torch_probe_error=", repr(e))
PY
