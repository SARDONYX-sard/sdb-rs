mod args;
mod error;
#[cfg(feature = "tracing")]
mod logger;

use crate::error::Result;
use args::app::AppArgs;
use args::dbg::{DbgArgs, SubCommand};
use clap::Parser;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use rustyline::{error::ReadlineError, DefaultEditor};
use sdb::process::{wait_on_signal, Process};
use std::fmt::Display;
use std::process::exit;

fn handle_command(process: &mut Process, line: &str) -> Result<()> {
    let mut lines = vec![""]; // HACK: Push exe item as dummy.
    lines.extend(line.split_whitespace());
    let args = DbgArgs::try_parse_from(lines)?;

    match args.sub_command {
        SubCommand::Continue => {
            process.resume()?;
            let status = wait_on_signal(process.pid)?;
            print_stop_reason(&process.pid, status);
        }
    }
    Ok(())
}

fn print_stop_reason(pid: &Pid, status: WaitStatus) {
    println!("Process {} ", pid);
    match status {
        WaitStatus::Exited(_pid, info) => println!("exited with status {info}"),
        WaitStatus::Stopped(_pid, signal) => println!("stopped with signal {signal}"),
        other => println!("{other:?}"),
    }
}

fn main_loop(mut process: Process) -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("sdb> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str())?;
                if let Err(err) = handle_command(&mut process, &line) {
                    eprintln!("{err}");
                    continue;
                };
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "file-history")]
    rl.save_history("history.txt");
    Ok(())
}

fn main() {
    let args = AppArgs::parse();

    if let Some(pid) = args.pid {
        let process = map_err_exit(Process::attach(pid));
        map_err_exit(main_loop(process));
    }

    if let Some(program_path) = args.program_path {
        let process = map_err_exit(Process::launch(&program_path, false));
        map_err_exit(main_loop(process));
    }
}

fn map_err_exit<T, Err: Display>(result: Result<T, Err>) -> T {
    match result {
        Ok(any) => any,
        Err(err) => {
            eprintln!("{err}");
            exit(-1)
        }
    }
}
