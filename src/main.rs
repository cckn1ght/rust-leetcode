#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
pub mod util;

pub mod fetcher;
pub mod helper;
pub mod templates;
pub mod commands;

use clap::{App, load_yaml, AppSettings};
use commands::*;

fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();
    match matches.subcommand() {
        ("setup", Some(matches)) => run_setup_command(matches),
        ("random", Some(matches)) => run_random_command(matches),
        ("solve", Some(matches)) => run_solve_command(matches),
        _ => unreachable!("The cli parser should prevent reaching here"),
    }
}
