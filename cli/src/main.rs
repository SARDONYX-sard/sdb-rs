mod args;
mod error;
#[cfg(feature = "tracing")]
mod logger;

use std::process::exit;

use crate::error::Result;
use args::app::AppArgs;
use args::dbg::{DbgArgs, SubCommand};
use clap::Parser;
use nix::sys::wait::WaitStatus;
use rustyline::{error::ReadlineError, DefaultEditor};
use sdb::Process;

fn handle_command(process: &mut Process, line: &str) -> Result<()> {
    let mut lines = vec![""]; // HACK: Push exe item as dummy.
    lines.extend(line.split_whitespace());
    let args = DbgArgs::try_parse_from(lines)?;

    match args.sub_command {
        SubCommand::Continue => {
            process.resume()?;
            print_stop_reason(process);
        }
    }
    Ok(())
}

fn print_stop_reason(process: &Process) {
    println!("Process {} ", process.pid);
    match process.state {
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
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
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

fn main() -> Result<()> {
    let args = AppArgs::parse();

    if let Some(pid) = args.pid {
        match Process::attach(pid) {
            Ok(process) => print_and_exit(main_loop(process)),
            Err(err) => {
                eprint!("{err}");
                exit(-1)
            }
        }
    }

    if let Some(program_path) = args.program_path {
        match Process::launch(&program_path) {
            Ok(process) => print_and_exit(main_loop(process)),
            Err(err) => {
                eprint!("{err}");
                exit(-1)
            }
        }
    }
    Ok(())
}

fn print_and_exit(result: Result<()>) {
    match result {
        Ok(_) => exit(0),
        Err(err) => {
            eprint!("{err}");
            exit(-1)
        }
    }
}
