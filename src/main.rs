use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use regex::Regex;
use std::{
    fs,
    io::{BufRead, Write},
    path::Path,
};

use clap::Parser;
use colored::Colorize;
use serde::Deserialize;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, trailing_var_arg = true)]
struct Args {
    /// Path to file containing args
    #[clap(short, long, default_value = "argfile.toml")]
    argfile: String,

    /// Port to run jupyter notebook on
    #[clap(short, long, default_value = "8889")]
    notebook_port: u16,

    /// Token to use for jupyter notebook
    #[clap(short, long)]
    token: Option<String>,

    /// Local port to forward to node
    #[clap(short, long, default_value = "58889")]
    local_port: u16,

    /// Args to pass to salloc
    #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
    salloc_args: Vec<String>,
}

#[derive(Parser, Debug, Deserialize, Clone)]
#[clap()]
struct SallocArgs {
    #[clap(long)]
    time: Option<String>,
    #[clap(long)]
    mem_per_cpu: Option<String>,
    #[clap(long)]
    gres: Option<String>,
    #[clap(long)]
    nodes: Option<String>,
    #[clap(long)]
    cpus_per_task: Option<String>,
    #[clap(long)]
    mem: Option<String>,
    #[clap(long)]
    constraint: Option<String>,
}

impl SallocArgs {
    fn get_args(self: &Self) -> Vec<String> {
        let mut args = vec![];
        if let Some(time) = &self.time {
            args.push(format!("--time={}", time));
        }
        if let Some(mem_per_cpu) = &self.mem_per_cpu {
            args.push(format!("--mem-per-cpu={}", mem_per_cpu));
        }
        if let Some(gres) = &self.gres {
            args.push(format!("--gres={}", gres));
        }
        if let Some(nodes) = &self.nodes {
            args.push(format!("--nodes={}", nodes));
        }
        if let Some(cpus_per_task) = &self.cpus_per_task {
            args.push(format!("--cpus-per-task={}", cpus_per_task));
        }
        if let Some(mem) = &self.mem {
            args.push(format!("--mem={}", mem));
        }
        if let Some(constraint) = &self.constraint {
            args.push(format!("--constraint={}", constraint));
        }
        args
    }
}

#[derive(Debug, Deserialize)]
struct Argfile {
    notebook_port: Option<u16>,
    token: Option<String>,
    local_port: Option<u16>,
    salloc: SallocArgs,
}

fn main() {
    let mut args = Args::parse();
    let mut salloc_args = SallocArgs::parse_from(args.salloc_args.clone());

    let argfile = Path::new(&args.argfile);

    if argfile.exists() {
        let fileargs: Argfile = toml::from_str(&fs::read_to_string(argfile).unwrap()).unwrap();
        salloc_args.constraint = fileargs.salloc.constraint;
        salloc_args.cpus_per_task = fileargs.salloc.cpus_per_task;
        salloc_args.gres = fileargs.salloc.gres;
        salloc_args.mem = fileargs.salloc.mem;
        salloc_args.mem_per_cpu = fileargs.salloc.mem_per_cpu;
        salloc_args.nodes = fileargs.salloc.nodes;
        salloc_args.time = fileargs.salloc.time;

        if let Some(notebook_port) = fileargs.notebook_port {
            args.notebook_port = notebook_port;
        }
        if let Some(token) = fileargs.token {
            args.token = Some(token);
        }
        if let Some(local_port) = fileargs.local_port {
            args.local_port = local_port;
        }
    }

    assert!(
        salloc_args.time.is_some(),
        "You must specify a salloc time!"
    );

    println!("{} {:?}", "salloc args:".blue(), salloc_args.get_args());
    println!("{} {:?}", "jupyter args:".blue(), args.clone());

    let mut token_args = vec![];
    if let Some(ref token) = args.token {
        token_args.push(format!("--NotebookApp.token={}", token.to_string()));
    };

    // let mut nb_command = vec![];

    // nb_command.push("jupyter-notebook".to_string());
    // nb_command.push(format!("--port={}", args.notebook_port));
    // nb_command.push("--no-browser".to_string());
    // nb_command.push("--ip=127.0.0.1".to_string());

    // nb_command.extend(token_args);

    // launch salloc
    let salloc_cmd = std::process::Command::new("srun")
        .args(salloc_args.get_args())
        .arg("-v")
        .arg("--preserve-env")
        .arg("/scratch/gpfs/samyakg/Research_2023/slurm-spawn-nb-rs/runb.sh")
        .arg(format!("--port={}", args.notebook_port))
        .arg("--no-browser")
        .arg("--ip=127.0.0.1")
        .args(token_args)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    println!("srun command id {}", salloc_cmd.id());

    let mut salloc_stderr = std::io::BufReader::new(salloc_cmd.stderr.unwrap());

    let mut node_name: Option<String> = None;
    let mut notebook_token: Option<String> = None;
    while node_name.is_none() || notebook_token.is_none() {
        let mut buf = String::new();
        salloc_stderr.read_line(&mut buf).unwrap();

        if buf.trim().len() > 0 {
            println!("{}", buf.trim().yellow());
        }

        let regex = Regex::new(r".*Nodes\s+([^\s]+).*").unwrap();
        regex.captures(&buf).map(|cap| {
            node_name = Some(cap[1].to_string());
            println!(
                "{}",
                format!("Running on node: {}", cap[1].to_string()).green()
            );
        });

        let token_regex = Regex::new(r".*token=([^\s]+).*").unwrap();
        token_regex.captures(&buf).map(|cap| {
            notebook_token = Some(cap[1].to_string());
            println!("{}", format!("Found token: {}", cap[1].to_string()).green());
        });
    }

    // forward port to node

    println!(
        "{}",
        format!(
            "Launched notebok! Access with url: http://localhost:{}/?token={}",
            args.local_port,
            args.token.unwrap_or(notebook_token.unwrap())
        )
        .green()
    );

    let _ssh_cmd = std::process::Command::new("ssh")
        .arg("-N")
        .arg("-L")
        .arg(format!(
            "localhost:{}:localhost:{}",
            args.local_port, args.notebook_port
        ))
        .arg(node_name.unwrap())
        .spawn()
        .unwrap();

    loop {
        let mut buf = String::new();
        salloc_stderr.read_line(&mut buf).unwrap();

        if buf.trim().len() > 0 {
            println!("{}", buf.trim().yellow());
        }
    }
}
