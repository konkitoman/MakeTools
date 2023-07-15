mod compiler_commands;

use std::{
    io::{BufRead, BufReader, Write},
    process::Command,
    time::Duration,
};

use clap::{Parser, Subcommand};
use colored::{ColoredString, Colorize};

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
    #[arg(short = 'm')]
    make_command: Option<String>,
    #[command(subcommand)]
    cmd: Commands,
}

impl MakeTools {
    pub fn run(self) {
        let make = self.make_command.unwrap_or(String::from("make"));
        match self.cmd {
            Commands::Build { args } => {
                let output = Command::new(&make)
                    .args(args.clone())
                    .arg("-n")
                    .output()
                    .unwrap();
                let buffer = String::from_utf8(output.stdout.to_vec()).unwrap();
                // Commands that is needed to run!
                let ctintr = buffer
                    .split('\n')
                    .filter(|s| is_compile_cmd(s))
                    .collect::<Vec<&str>>();
                let len = ctintr.len();

                let mut make_process = Command::new(&make)
                    .args(args)
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .unwrap();

                let output = make_process.stdout.take().unwrap();
                let reader = BufReader::new(output);

                let mut current_time = std::time::SystemTime::now();
                let mut elapsed_times = Vec::new();

                let mut count = 0;
                for new in reader.lines().flatten() {
                    if let Ok(elapsed) = current_time.elapsed() {
                        elapsed_times.push(elapsed.as_secs_f64())
                    }
                    current_time = std::time::SystemTime::now();
                    let sum: f64 = elapsed_times.iter().sum();
                    let avg = sum / elapsed_times.len() as f64;
                    let remaining_time = avg * (len - count) as f64;
                    if is_compile_cmd(&new) {
                        count += 1;
                    }

                    let remaining_time = format_time(remaining_time);

                    let eta = if remaining_time.is_empty() {
                        String::new()
                    } else {
                        format!(" Eta {remaining_time}")
                    };

                    let progress = count as f64 / len as f64 * 100.0;
                    let progress = if progress.is_nan() {
                        "Completed".green().to_string()
                    } else {
                        let progress = format!("{progress:.2}%").green();
                        format!("[{count}/{len}] {progress}")
                    };

                    println!("{progress}{eta} :{new}",)
                }

                let sum: f64 = elapsed_times.iter().sum();
                println!("Completed in {}", format_time(sum));
            }
            Commands::CompileCommands { args } => {
                println!("Please wait!");
                let output = std::process::Command::new(&make)
                    .args(args)
                    .arg("-n")
                    .output()
                    .expect(format!("You probably not have {make} on your system, You need install \"{make}\" and then try again!").as_str());
                let buffer = String::from_utf8(output.stdout.to_vec()).unwrap();
                // this is for preventing gcc -o main \
                // main.c
                // this is putting every argument on the same line
                let commands = buffer.split("\\\n").collect::<String>();
                // get all lines and find if is a compiler command!
                let filtered_commends = commands
                    .split('\n')
                    .filter(|s| is_compile_cmd(s))
                    .collect::<Vec<&str>>();

                use compiler_commands::Command;
                let mut compile_commands = Vec::new();

                for command in filtered_commends {
                    let mut input_files: Vec<&str> = Vec::new();
                    let mut output_file: Option<&str> = None;

                    // get replace local compiler path with the fill path
                    let mut args = command
                        .split(' ')
                        .filter(|e| !e.is_empty())
                        .collect::<Vec<&str>>();
                    let compiler = args.first().unwrap();
                    let compiler = match std::process::Command::new("which").arg(compiler).output()
                    {
                        Ok(output) => {
                            if output.status.success() {
                                match String::from_utf8(output.stdout) {
                                    Ok(str) => str.trim().to_owned(),
                                    Err(err) => panic!("Cannot parse output form which: {err:?}"),
                                }
                            } else {
                                panic!("Cannot find: {}", compiler);
                            }
                        }
                        Err(_) => {
                            panic!("Error accured when running which");
                        }
                    };

                    *args.first_mut().unwrap() = compiler.as_str();

                    // getting the input files and output file
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
                    // get the full path of every file
                    let input_files = input_files
                        .iter()
                        .map(|e| current_dir.join(e).to_str().unwrap().to_owned())
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
                    .truncate(true)
                    .open("compile_commands.json")
                    .unwrap();
                file.write_all(compiler_commands.as_bytes()).unwrap();
                println!("compile_commands.json was created succesfuly!");
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

fn format_time(secs: f64) -> ColoredString {
    let minutes = secs / 60.0;
    let hours = minutes / 60.0;
    let days = minutes / 24.0;

    let secs = secs as u32 % 60;
    let minutes = minutes as u32 % 60;
    let hours = hours as u32 % 24;
    let days = days as u32;

    if days > 0 {
        format!("Days {days}:{hours:00}:{minutes:00}")
    } else if hours > 0 {
        format!("Hours {hours}:{minutes:00}:{secs:00}")
    } else if minutes > 0 {
        format!("Minutes {minutes:00}:{secs:00}")
    } else if secs > 0 {
        format!("Secs {secs:00}")
    } else {
        String::new()
    }
    .blue()
}

fn main() {
    let context = MakeTools::parse();
    context.run();
}
