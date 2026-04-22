#!/bin/bash
#SBATCH --job-name=hostname
#SBATCH --output=slurm-%j.out
#SBATCH --partition=debug

set -x
pwd
hostname
