mod split;
mod ffmpeg;

use std::path::PathBuf;
use clap::{ArgMatches, Command, Subcommand};

fn cli() -> Command {
    Command::new("badgerclips")
        .about("Utilities for dealing with video clips")
        .subcommand_required(true)
        .subcommand(split::command())
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("split", sub_matches)) => {
            split::run(sub_matches);
        }
        _ => {}
    }
}
