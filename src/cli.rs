use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(name = "rdu", version, about = "A fast, tree-style disk usage viewer")]
pub struct Cli {
    /// Directory to analyze
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Maximum depth to display
    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,

    /// Recurse into all subdirectories without depth limit
    #[arg(short, long)]
    pub recursive: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Print shell completion script to stdout
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
}

impl Cli {
    pub fn max_depth(&self) -> Option<usize> {
        if self.recursive { None } else { Some(self.depth) }
    }

    pub fn generate_completions(shell: Shell) {
        clap_complete::generate(
            shell,
            &mut Cli::command(),
            "rdu",
            &mut std::io::stdout(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_cli(depth: usize, recursive: bool) -> Cli {
        Cli {
            path: PathBuf::from("."),
            depth,
            recursive,
            no_color: false,
            command: None,
        }
    }

    #[test]
    fn max_depth_default_depth() {
        let cli = make_cli(1, false);
        assert_eq!(cli.max_depth(), Some(1));
    }

    #[test]
    fn max_depth_custom_depth() {
        let cli = make_cli(5, false);
        assert_eq!(cli.max_depth(), Some(5));
    }

    #[test]
    fn max_depth_zero() {
        let cli = make_cli(0, false);
        assert_eq!(cli.max_depth(), Some(0));
    }

    #[test]
    fn max_depth_recursive_ignores_depth() {
        let cli = make_cli(3, true);
        assert_eq!(cli.max_depth(), None);
    }

    #[test]
    fn max_depth_recursive_with_default_depth() {
        let cli = make_cli(1, true);
        assert_eq!(cli.max_depth(), None);
    }
}
