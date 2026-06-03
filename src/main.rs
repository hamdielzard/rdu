mod cli;
mod display;
mod format;
mod scanner;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let args = Cli::parse();

    if let Some(Command::Completions { shell }) = args.command {
        Cli::generate_completions(shell);
        return;
    }

    if args.no_color {
        colored::control::set_override(false);
    }

    let display_name = args.path.to_string_lossy().into_owned();
    let path = args.path.canonicalize().unwrap_or(args.path.clone());

    match scanner::scan(&path, args.max_depth()) {
        Ok(root) => {
            let stats = scanner::Stats::from_entry(&root);
            println!("{}", display::render_tree(&root, &display_name));
            println!();
            println!(
                "{}  •  {} files  •  {} directories",
                format::format_size(root.size),
                stats.file_count,
                stats.dir_count,
            );
        }
        Err(e) => {
            eprintln!("rdu: {}", e);
            std::process::exit(1);
        }
    }
}
