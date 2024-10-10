use clap::{Parser, Subcommand};

/// A command-line debugger argument parser
#[derive(Debug, PartialEq, Eq, Parser)]
#[command(name = "Debugger")]
#[command(about = "A command-line debugger", long_about = None)]
pub struct DbgArgs {
    #[command(subcommand)]
    pub sub_command: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    /// Continue the process execution
    Continue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    fn should_parse() -> Result<()> {
        let line = "continue";
        let mut lines = vec![""]; // Push exe item as dummy.
        lines.extend(line.split_whitespace());

        match DbgArgs::try_parse_from(lines) {
            Ok(args) => {
                let expected = DbgArgs {
                    sub_command: SubCommand::Continue,
                };
                assert_eq!(args, expected)
            }
            Err(err) => panic!("{err}"),
        };
        Ok(())
    }
}
