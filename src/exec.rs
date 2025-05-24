use crate::Result;
use crate::argc::Args;
use crate::dir_info::process_dir_info;
use size::Size;

// region:    --- Constants

const DEFAULT_DIR: &str = "./";
const TOP_NUMS: usize = 5;

// endregion: --- Constants

// region:    --- Types

pub struct Options {
	pub paths: Vec<String>,
	pub nums: usize,
	pub glob: Option<Vec<String>>,
	pub no_ext: bool,
	pub summary: bool,
}

// endregion: --- Types

// region:    --- Public Functions

impl Options {
	pub fn from_args(args: Args) -> Result<Options> {
		// -- paths
		let paths = if args.paths.is_empty() {
			vec![DEFAULT_DIR.to_string()]
		} else {
			args.paths
		};

		// -- nums
		let nums: usize = match args.nums {
			None => TOP_NUMS,
			Some(nums) => nums,
		};

		// -- glob
		let glob = args
			.glob
			.map(|glob| glob.split(',').map(|s| s.to_string()).collect::<Vec<String>>());

		// -- no_ext
		let no_ext = args.no_ext;

		// -- summary
		let summary = args.summary;

		Ok(Options {
			paths,
			nums,
			glob,
			no_ext,
			summary,
		})
	}
}

pub fn run(options: Options) -> Result<()> {
	for (i, path) in options.paths.iter().enumerate() {
		if i > 0 {
			println!("\n");
		}

		exec_single_path(path, &options)?;
	}

	Ok(())
}

// endregion: --- Public Functions

// region:    --- Private Functions

fn exec_single_path(path_str: &str, options: &Options) -> Result<()> {
	let dir_info = process_dir_info(path_str, options)?;

	if options.summary {
		println!(
			"{:<15}: {} files, {}",
			dir_info.path_processed,
			dir_info.total_numbers,
			Size::from_bytes(dir_info.total_size),
		);
		return Ok(());
	}

	println!(
		"==== Directory info on '{}'\n\n{:>15}: {}\n{:>15}: {}",
		dir_info.path_processed,
		"Number of files",
		dir_info.total_numbers,
		"Total size",
		Size::from_bytes(dir_info.total_size),
	);

	if let Some(ext_stats_data) = &dir_info.ext_stats {
		println!("\n== Top {} biggest size by extension", options.nums); // Title uses configured nums

		for ext_stat in ext_stats_data.top_by_ext.iter() {
			println!("{:<10} - {}", Size::from_bytes(ext_stat.size).to_string(), ext_stat.ext);
		}

		if ext_stats_data.others_size > 0 {
			println!(
				"{:<10} - (others)",
				Size::from_bytes(ext_stats_data.others_size).to_string()
			);
		}
	}

	if !dir_info.top_files.is_empty() {
		println!("\n== Top {} biggest files", dir_info.top_files.len()); // Title uses actual count of files found
		for entry_info in dir_info.top_files.iter() {
			println!(
				"{:<10} - {}",
				Size::from_bytes(entry_info.size).to_string(),
				entry_info.path.as_str()
			);
		}
	}

	println!("\n=====");

	Ok(())
}

// endregion: --- Private Functions
