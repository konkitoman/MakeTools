mod compiler_commands;
mod programs;

use std::{
    io::{BufRead, BufReader, Write},
    process::Command,
};

use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::programs::Programs;

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    #[command(about = "Will be used insted of make!")]
    Build { args: Vec<String> },
    #[command(about = "Create compiler_commands.json")]
    CompileCommands { args: Vec<String> },
}

#[derive(Parser, Debug)]
#[command(version)]
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
            Commands::CompileCommands { args } => {
                let output = std::process::Command::new("make")
                    .args(args)
                    .arg("-n")
                    .output()
                    .unwrap();
                let buffer = String::from_utf8(output.stdout.to_vec()).unwrap();
                let commands = buffer.split("\\\n").collect::<String>();
                let filtered_commends = commands
                    .split('\n')
                    .filter(|s| is_compile_cmd(s))
                    .collect::<Vec<&str>>();

                use compiler_commands::Command;
                let mut compile_commands = Vec::new();

                let mut programs = Programs::default();

                for command in filtered_commends {
                    let mut input_files: Vec<&str> = Vec::new();
                    let mut output_file: Option<&str> = None;

                    let mut args = command
                        .split(' ')
                        .filter(|e| !e.is_empty())
                        .collect::<Vec<&str>>();
                    let compiler = args.first().unwrap();
                    let compiler = programs.find(*compiler).unwrap();
                    *args.first_mut().unwrap() = compiler.to_str().unwrap();

                    for i in 1..args.len() {
                        if args[i] == "-o" {
                            output_file = Some(args[i + 1]);
                        } else if args[i].starts_with('-') {
                        } else if args[i] != "." && !args[i].is_empty() {
                            input_files.push(args[i]);
                        }
                    }

                    let current_dir = std::env::current_dir().unwrap();

                    let input_files: Vec<&&str> = input_files
                        .iter()
                        .filter(|e| {
                            e.ends_with(".c")
                                || e.ends_with(".cpp")
                                || e.ends_with(".cc")
                                || e.ends_with(".c++")
                                || e.ends_with(".cxx")
                                || e.ends_with(".C")
                        })
                        .collect();
                    let input_files = input_files
                        .iter()
                        .map(|e| current_dir.join(e))
                        .map(|e| e.to_str().unwrap().to_owned())
                        .collect::<Vec<String>>();
                    let Some(output_file) = output_file else{continue};
                    let output_file = current_dir.join(output_file).to_str().unwrap().to_owned();

                    for input_file in input_files {
                        compile_commands.push(Command {
                            arguments: args.iter().map(|e| e.to_string()).collect(),
                            directory: current_dir.to_str().unwrap().to_owned(),
                            file: input_file,
                            output: output_file.clone(),
                        });
                    }
                }

                let compiler_commands = serde_json::to_string_pretty(&compile_commands).unwrap();
                let mut file = std::fs::File::options()
                    .write(true)
                    .create(true)
                    .open("compile_commands.json")
                    .unwrap();
                file.write_all(compiler_commands.as_bytes()).unwrap();
                println!("Compiler commands created succesfuly!");
            }
        }
    }
}

fn is_compile_cmd(s: &str) -> bool {
    let Some(s) = s.trim().split(' ').next() else{return false};
    matches!(
        s,
        "gcc"
            | "g++"
            | "clang"
            | "clang++"
            | "x86_64-w64-mingw32-gcc"
            | "x86_64-w64-mingw32-g++"
            | "x86_64-w64-mingw32-clang"
            | "x86_64-w64-mingw32-clang++"
    )
}

fn main() {
    let context = MakeTools::parse();
    context.run();
}
