mod argc;
mod error;

pub use self::error::{Error, Result};

use argc::argc_app;
use clap::ArgMatches;
use clap::{crate_version, Arg, Command};
use file_size::fit_4;
use globset::{Glob, GlobSetBuilder};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

const DIR: &str = "./";
const TOP_NUMS: usize = 5;

fn main() {
	let argc = argc_app().get_matches();

	let options = match Options::from_argc(argc) {
		Ok(options) => options,
		Err(ex) => {
			println!("ERROR parsing input {}", ex);
			return;
		}
	};

	match exec(options) {
		Ok(_) => (),
		Err(ex) => {
			println!("ERROR - {}", ex);
		}
	}
}

struct Entry {
	path: PathBuf,
	size: u64,
}

struct Options {
	nums: usize,
	glob: Option<Vec<String>>,
	no_ext: bool,
}

impl Options {
	fn from_argc(argc: ArgMatches) -> Result<Options> {
		// -- nums
		let nums: usize = match argc.get_one::<String>("nums") {
			None => TOP_NUMS,
			Some(nums) => nums
				.parse::<usize>()
				.map_err(|_| Error::InvalidNumberOfFiles(nums.to_string()))?,
		};

		// -- glob
		let glob = argc
			.get_one::<String>("glob")
			.map(|glob| glob.split(',').map(|s| s.to_string()).collect::<Vec<String>>());

		// -- by_ext
		let no_ext = argc.get_flag("no-ext");

		Ok(Options { nums, glob, no_ext })
	}
}

fn exec(options: Options) -> Result<()> {
	let mut total_size: u64 = 0;
	let mut total_numbers: u32 = 0;
	let mut tops: Vec<Entry> = Vec::with_capacity(options.nums + 1);
	let mut min_of_tops = 0;

	let glob = options
		.glob
		.map(|vs| {
			let mut builder = GlobSetBuilder::new();
			for v in vs {
				builder.add(Glob::new(&v)?);
			}
			builder.build()
		})
		.transpose()?;

	let mut by_ext: Option<HashMap<String, u64>> = if !options.no_ext { Some(HashMap::new()) } else { None };

	// get entry iterator.
	let entries = WalkDir::new(DIR)
		.into_iter()
		.filter_map(|e| e.ok())
		// match the ventual glob
		.filter(|e| {
			if let Some(glob) = &glob {
				glob.is_match(e.path())
			} else {
				true
			}
		});

	for entry in entries {
		let path = entry.path();
		if path.is_file() && !entry.path_is_symlink() {
			total_numbers += 1;
			let size = entry.metadata()?.len();
			total_size += size;

			if let Some(by_ext) = &mut by_ext {
				if let Some(ext) = path.extension() {
					let ext = ext.to_string_lossy().to_string();
					*by_ext.entry(ext).or_insert(0) += size;
				}
			}

			if min_of_tops < size {
				tops.push(Entry {
					path: entry.path().to_path_buf(),
					size,
				});
				tops.sort_by(|a, b| b.size.cmp(&a.size));
				if tops.len() > options.nums {
					tops.pop();
				}
				min_of_tops = tops.last().map(|e| e.size).unwrap_or(0);
			}
		}
		// println!("{}", entry.path().display());
	}

	println!(
		"== Summary\nNumber of files {}\nTotal size: {}",
		total_numbers,
		fit_4(total_size)
	);

	let mut others_size = 0;
	if let Some(mut by_ext) = by_ext {
		println!("\n== Top {} biggest size by extension", options.nums);
		let mut by_ext: Vec<(String, u64)> = by_ext.drain().collect();
		by_ext.sort_by(|a, b| b.1.cmp(&a.1));

		for (i, (ext, size)) in by_ext.iter().enumerate() {
			if i < options.nums {
				println!("{:<4} - {}", fit_4(*size), ext);
			} else {
				others_size += size;
			}
		}
	}

	if others_size > 0 {
		println!("{:<4} - (others)", fit_4(others_size));
	}

	println!("\n== Top {} biggest files", tops.len());
	for Entry { size, path } in tops.iter() {
		println!("{:<4} - {}", fit_4(*size), path.to_string_lossy());
	}

	Ok(())
}
