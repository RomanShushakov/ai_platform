#!/bin/bash
#SBATCH --job-name=python-version
#SBATCH --output=slurm-%j.out
#SBATCH --partition=debug

python3 --version
which python3
hostname
