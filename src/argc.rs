use clap::{crate_version, Arg, Command};

pub fn argc_app() -> Command {
	Command::new("rec")
		.version(crate_version!())
		.about("Simple directory information")
		.arg(
			Arg::new("nums")
				.help("Number of biggest files to display")
				.short('n')
				.num_args(1),
		)
}
