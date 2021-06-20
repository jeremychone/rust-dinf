use clap::{crate_version, App, Arg};

pub fn argc_app() -> App<'static, 'static> {
	App::new("dinf")
		.version(&crate_version!()[..])
		.about("Simple directory information")
		.arg(Arg::with_name("nums").help("Number of biggest files to display").short("n").takes_value(true))
}
