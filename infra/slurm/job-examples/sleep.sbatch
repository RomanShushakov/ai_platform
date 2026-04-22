#!/bin/bash
#SBATCH --job-name=sleep
#SBATCH --output=slurm-%j.out
#SBATCH --partition=debug

echo "starting on $(hostname)"
sleep 10
echo "done"
