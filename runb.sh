#!/bin/bash

# This script runs the 'jupyter notebook' command with the passed arguments

which python
which jupyter-notebook
jupyter-notebook "$@"