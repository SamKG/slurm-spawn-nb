# slurm-spawn-nb

## What is this repo for?
Setting up and running jupyter notebooks in slurm is tedious and finnicky. It involves the following steps:
1) allocating a slurm node (salloc command)
2) running jupyter-notebook on the node
3) ssh forwarding to the login node (so you can actually use it if the node is airgapped)
4) using the authentication token to log in to the notebook

This program automates all of that, while letting you easily configure your slurm job.

## How do I use it?

### Installation
1) Clone this repo
2) Install rust (https://www.rust-lang.org/tools/install)
3) Run `cargo install --path .` in the repo directory
4) (if not already done) add `~/.cargo/bin` to your PATH variable (recommended to do this in ~/.bashrc)

### Usage
1) Change to the directory you want to run the notebook in
2) Create a config file (see [argfile.toml](argfile.toml) for an example)
3) Activate your conda/venv environment
4) Run salloc-nb 

The output should display a URL. You can use this one of two ways:
1) With VScode, simply open the .ipynb file you want to run, and select the kernel in the top right. Choose "Existing Jupyter Server" and "Enter the URL of the running Jupyter Server", and copy-paste the URL from the output of salloc-nb. You will only need to do this once, and VScode will remember the server. Now you will be able to run the notebook as normal on the allocated node.
2) If you are using your browser, you will need to ssh forward the port to access from your local machine (VScode should do this automatically). Then navigate to the URL in your browser.

