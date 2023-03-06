use std::{
    io::{BufRead, BufReader},
    process::Command,
};

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    #[command(about = "Will be used insted of make!")]
    Build { args: Vec<String> },
}

#[derive(Parser, Debug)]
struct MakeTools {
    #[command(subcommand)]
    cmd: Commands,
}

impl MakeTools {
    pub fn run(self) {
        match self.cmd {
            Commands::Build { args } => {
                let output = Command::new("make")
                    .args(args.clone())
                    .arg("-n")
                    .output()
                    .unwrap();
                let buffer = String::from_utf8(output.stdout.to_vec()).unwrap();
                let need_to_run = buffer
                    .split('\n')
                    .filter(|s| is_compile_cmd(s))
                    .collect::<Vec<&str>>();
                let len = need_to_run.len();

                let mut make_process = Command::new("make")
                    .args(args)
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .unwrap();

                let output = make_process.stdout.take().unwrap();
                let reader = BufReader::new(output);

                let mut count = 0;

                for new in reader.lines().flatten() {
                    if is_compile_cmd(&new) {
                        count += 1;
                    }
                    println!(
                        "[{}/{}] {} :{new}",
                        count,
                        len,
                        format!("{:.2}%", count as f64 / len as f64 * 100.0).green(),
                    )
                }
            }
        }
    }
}

fn is_compile_cmd(s: &str) -> bool {
    s.contains("gcc")
        || s.contains("g++")
        || s.contains("clang")
        || s.contains("clang++")
        || s.contains("x86_64-w64-mingw32-gcc")
        || s.contains("x86_64-w64-mingw32-g++")
}

fn main() {
    let context = MakeTools::parse();
    context.run();
}
