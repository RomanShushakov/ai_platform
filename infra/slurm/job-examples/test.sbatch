#!/bin/bash
#SBATCH --job-name=test
#SBATCH --partition=debug

echo "Running on:"
hostname
echo "User:"
whoami
echo "Date:"
date
