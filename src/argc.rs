use clap::{crate_version, Arg, Command};

pub fn argc_app() -> Command {
	Command::new("rec")
		.version(crate_version!())
		.about("Simple directory information")
		.arg(
			Arg::new("paths")
				.help("Base directory paths to analyze")
				.num_args(1..)
				.required(false),
		)
		.arg(
			Arg::new("nums")
				.help("Number of biggest files to display")
				.short('n')
				.num_args(1),
		)
		.arg(Arg::new("glob").help("globs, comma separated").short('g').num_args(1))
		.arg(
			Arg::new("no-ext")
				.action(clap::ArgAction::SetTrue)
				.long("no-ext")
				.help("group by extension"),
		)
}
