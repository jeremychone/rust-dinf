use crate::Result;
use crate::argc::Args;

use globset::{Glob, GlobSetBuilder};
use simple_fs::SPath;
use size::Size;
use std::collections::HashMap;
use walkdir::WalkDir;

// region:    --- Constants

const DEFAULT_DIR: &str = "./";
const TOP_NUMS: usize = 5;

// endregion: --- Constants

// region:    --- Types

struct Entry {
	path: SPath,
	size: u64,
}

pub struct Options {
	paths: Vec<String>,
	nums: usize,
	glob: Option<Vec<String>>,
	no_ext: bool,
	summary: bool,
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
	let mut total_size: u64 = 0;
	let mut total_numbers: u32 = 0;
	let mut tops: Vec<Entry> = Vec::with_capacity(options.nums + 1);
	let mut min_of_tops = 0;

	let glob_set = options
		.glob
		.as_ref()
		.map(|vs| {
			let mut builder = GlobSetBuilder::new();
			for v in vs {
				builder.add(Glob::new(v)?);
			}
			builder.build()
		})
		.transpose()?;

	let mut by_ext_map: Option<HashMap<String, u64>> = if !options.no_ext && !options.summary {
		Some(HashMap::new())
	} else {
		None
	};

	// get entry iterator.
	let entries = WalkDir::new(path_str)
		.into_iter()
		.filter_map(|e| e.ok())
		// match the eventual glob
		.filter(|e| {
			if let Some(gs) = &glob_set {
				gs.is_match(e.path())
			} else {
				true
			}
		});

	for entry in entries {
		let entry_path = entry.path();
		if entry_path.is_file() && !entry.path_is_symlink() {
			total_numbers += 1;
			let size = entry.metadata()?.len();
			total_size += size;

			if let Some(current_by_ext_map) = &mut by_ext_map {
				if let Some(ext) = entry_path.extension() {
					let ext_str = ext.to_string_lossy().to_string();
					*current_by_ext_map.entry(ext_str).or_insert(0) += size;
				}
			}

			if !options.summary && min_of_tops < size {
				let Ok(spath) = SPath::from_std_path(entry.path()) else {
					// Silently skip if path conversion fails, maintaining original behavior
					continue;
				};
				tops.push(Entry { path: spath, size });
				tops.sort_by(|a, b| b.size.cmp(&a.size));
				if tops.len() > options.nums {
					tops.pop();
				}
				min_of_tops = tops.last().map(|e| e.size).unwrap_or(0);
			}
		}
	}

	if options.summary {
		println!(
			"{:<15}: {} files, {}",
			path_str,
			total_numbers,
			Size::from_bytes(total_size),
		);
		return Ok(());
	}

	println!(
		"==== Directory info on '{}'\n\n{:>15}: {}\n{:>15}: {}",
		path_str,
		"Number of files",
		total_numbers,
		"Total size",
		Size::from_bytes(total_size),
	);

	let mut others_size = 0;
	if let Some(mut current_by_ext_map) = by_ext_map {
		println!("\n== Top {} biggest size by extension", options.nums);
		let mut by_ext_vec: Vec<(String, u64)> = current_by_ext_map.drain().collect();
		by_ext_vec.sort_by(|a, b| b.1.cmp(&a.1));

		for (i, (ext, current_size)) in by_ext_vec.iter().enumerate() {
			if i < options.nums {
				println!("{:<10} - {}", Size::from_bytes(*current_size).to_string(), ext);
			} else {
				others_size += current_size;
			}
		}
	}

	if others_size > 0 {
		println!("{:<10} - (others)", Size::from_bytes(others_size).to_string());
	}

	println!("\n== Top {} biggest files", tops.len());
	for Entry {
		size,
		path: entry_spath,
	} in tops.iter()
	{
		println!("{:<10} - {}", Size::from_bytes(*size).to_string(), entry_spath.as_str());
	}

	println!("\n=====");

	Ok(())
}

// endregion: --- Private Functions
