use std::env;

use clap::Clap;

#[macro_use]
mod macros;
mod cli;
mod error;

fn main() {
    let args = cli::Args::parse();

    match args.verbose {
        0 => match args.quiet {
            0 => env::set_var("RUST_LOG", "{{crate_name}}=info"),
            1 => env::set_var("RUST_LOG", "{{crate_name}}=warn"),
            2 => env::set_var("RUST_LOG", "{{crate_name}}=error"),
            _ => env::set_var("RUST_LOG", "{{crate_name}}=off"),
        },
        1 => env::set_var("RUST_LOG", "{{crate_name}}=debug"),
        _ => env::set_var("RUST_LOG", "{{crate_name}}=trace"),
    }
}
