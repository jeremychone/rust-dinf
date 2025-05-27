use crate::Result;
use crate::argc::Args;
use crate::dir_info::{DirInfo, process_dir_info};
use crate::support::{format_num, format_size};
use std::fs;

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
		// -- Determine initial paths
		let initial_paths = if args.paths.is_empty() {
			vec![DEFAULT_DIR.to_string()]
		} else {
			args.paths
		};

		// -- Expand to children if requested
		let final_paths = if args.children {
			let mut children_paths = Vec::new();
			for path_str in &initial_paths {
				let read_dir = fs::read_dir(path_str)?;
				for entry_result in read_dir {
					let entry = entry_result?;
					let path = entry.path();
					if path.is_dir() {
						let path_s = path
							.to_str()
							.ok_or_else(|| crate::Error::PathNotUtf8(format!("{:?}", path)))?
							.to_string();
						children_paths.push(path_s);
					}
				}
			}
			children_paths
		} else {
			initial_paths
		};

		// -- nums
		let nums: usize = match args.nums {
			None => TOP_NUMS,
			Some(n) => n,
		};

		// -- glob
		let glob = args.glob.map(|g| g.split(',').map(|s| s.to_string()).collect::<Vec<String>>());

		// -- no_ext
		let no_ext = args.no_ext;

		// -- summary
		let summary = args.summary;

		Ok(Options {
			paths: final_paths,
			nums,
			glob,
			no_ext,
			summary,
		})
	}
}

pub fn run(options: Options) -> Result<()> {
	if options.summary {
		let mut dir_infos: Vec<DirInfo> = Vec::new();
		for path_str in &options.paths {
			// process_dir_info correctly handles options.summary
			// to avoid unnecessary computation for top_files and ext_stats.
			let dir_info = process_dir_info(path_str, &options)?;
			dir_infos.push(dir_info);
		}

		// Sort dir_infos by path_processed for consistent summary output
		dir_infos.sort_by(|a, b| a.path_processed.cmp(&b.path_processed));

		if !dir_infos.is_empty() {
			println!();
			// Determine the maximum path length for alignment.
			let max_path_len = dir_infos.iter().map(|di| di.path_processed.len()).max().unwrap_or(0);

			// Compute file num strs, and max len
			let files_num_strs: Vec<String> = dir_infos
				.iter()
				.map(|di| format!("{} files", format_num(di.total_numbers)))
				.collect();
			let max_files_str_len = files_num_strs.iter().map(|s| s.len()).max().unwrap_or(0);

			// Compute file saize strs, and max len
			let size_strs = dir_infos
				.iter()
				.map(|di| format_size(di.total_size, true))
				.collect::<Vec<String>>();
			let max_size_width = size_strs.iter().map(|s| s.len()).max().unwrap_or(0);

			// Print all summaries aligned.
			for ((dir_info, files_str), size_str) in dir_infos.iter().zip(files_num_strs).zip(size_strs) {
				println!(
					"{path:<path_width$}  - {files_str:>files_width$} | total size: {size:>max_size_width$}",
					path = dir_info.path_processed,
					files_str = files_str,
					size = size_str,
					max_size_width = max_size_width,
					path_width = max_path_len,
					files_width = max_files_str_len
				);
			}
		}
	} else {
		// Original behavior for detailed output if not in summary mode.
		for (i, path) in options.paths.iter().enumerate() {
			if i > 0 {
				println!("\n"); // Separator for multiple paths in detailed mode.
			}
			// exec_single_path will handle the detailed printing for each path.
			exec_single_path(path, &options)?;
		}
	}

	Ok(())
}

// endregion: --- Public Functions

// region:    --- Private Functions

fn exec_single_path(path_str: &str, options: &Options) -> Result<()> {
	// When this function is called, options.summary is false.
	// process_dir_info will populate top_files and ext_stats as needed for detailed view.
	let dir_info = process_dir_info(path_str, options)?;

	// Detailed printing logic.
	println!(
		"==== Directory info on '{}'\n\n{:>15}: {}\n{:>15}: {}",
		dir_info.path_processed,
		"Number of files",
		format_num(dir_info.total_numbers),
		"Total size",
		format_size(dir_info.total_size, true),
	);

	// ext_stats will be Some if options.no_ext is false (given options.summary is false).
	if let Some(ext_stats_data) = &dir_info.ext_stats {
		println!("\n== Top {} biggest size by extension", options.nums); // Title uses configured nums

		for ext_stat in ext_stats_data.top_by_ext.iter() {
			println!("{:<8} - {}", format_size(ext_stat.size, false), ext_stat.ext);
		}

		if ext_stats_data.others_size > 0 {
			println!("{:<8} - (others)", format_size(ext_stats_data.others_size, false));
		}
	}

	// top_files will be populated because options.summary is false.
	if !dir_info.top_files.is_empty() {
		println!("\n== Top {} biggest files", dir_info.top_files.len()); // Title uses actual count of files found
		for entry_info in dir_info.top_files.iter() {
			println!(
				"{:<8} - {}",
				format_size(entry_info.size, false),
				entry_info.path.as_str()
			);
		}
	}

	println!("\n=====");

	Ok(())
}

// endregion: --- Private Functions

