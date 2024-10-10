use clap::ArgGroup;
use std::path::PathBuf;

#[cfg(feature = "color")]
use self::color::get_styles;
#[cfg(feature = "tracing")]
use crate::logger::LogLevel;

/// CLI command arguments
#[derive(Debug, clap::Parser)]
#[clap(version, about, author)]
#[clap(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["pid", "program_path"]),
))]
#[cfg_attr(feature = "color", clap(styles=get_styles()))]
#[clap(arg_required_else_help = true, args_conflicts_with_subcommands = true)]
pub(crate) struct AppArgs {
    /// Program exe path
    pub program_path: Option<PathBuf>,

    /// ID of the process to debug
    #[clap(short)]
    pub pid: Option<i32>,

    // --logger (Global options)
    #[cfg(feature = "tracing")]
    #[clap(global = true, long, display_order = 101)]
    #[clap(ignore_case = true, default_value = "error")]
    /// Log level to be recorded in logger
    pub log_level: LogLevel,

    #[cfg(feature = "tracing")]
    #[clap(global = true, long, display_order = 102)]
    /// Output path of log file
    pub log_file: Option<PathBuf>,
}
