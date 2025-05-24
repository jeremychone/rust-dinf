// region:    --- Modules

mod argc;
mod dir_info; // Added new module
mod error;
mod exec;
mod support;

pub use error::{Error, Result}; // Re-export for the crate, consistent with Rust10x error best practices

use argc::Args; // For args parsing struct
use clap::Parser; // For the .parse() method trait

// endregion: --- Modules

fn main() {
	if let Err(err) = run_app() {
		eprintln!("ERROR - {}", err);
		std::process::exit(1);
	}
}

fn run_app() -> Result<()> {
	let args = Args::parse();
	let options = exec::Options::from_args(args)?;
	exec::run(options)
}
