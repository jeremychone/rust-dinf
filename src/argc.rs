use clap::{Parser, command};

#[derive(Parser)]
#[command(name = "dinf")]
#[command(version)]
#[command(about = "Simple directory information")]
pub struct Args {
	/// Base directory paths to analyze
	pub paths: Vec<String>,

	/// Number of biggest files to display
	#[arg(short = 'n')]
	pub nums: Option<usize>,

	/// globs, comma separated
	#[arg(short = 'g')]
	pub glob: Option<String>,

	/// group by extension
	#[arg(long = "no-ext")]
	pub no_ext: bool,

	/// show only summary (number of files and total size)
	#[arg(short = 's', long = "summary")]
	pub summary: bool,
}
