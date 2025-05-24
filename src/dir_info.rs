use crate::Result; // crate::error::Result
use crate::exec::Options;
use globset::{Glob, GlobSetBuilder};
use simple_fs::SPath;
use std::collections::HashMap;
use walkdir::WalkDir;

// region:    --- Types

#[derive(Debug)]
pub struct DirEntryInfo {
	pub path: SPath,
	pub size: u64,
}

#[derive(Debug)]
pub struct ExtStat {
	pub ext: String,
	pub size: u64,
}

#[derive(Debug, Default)]
pub struct ExtStats {
	pub top_by_ext: Vec<ExtStat>,
	pub others_size: u64,
}

#[derive(Debug)]
pub struct DirInfo {
	pub path_processed: String,
	pub total_numbers: u32,
	pub total_size: u64,
	pub top_files: Vec<DirEntryInfo>, // Populated if not options.summary
	pub ext_stats: Option<ExtStats>,  // Populated if not options.summary and not options.no_ext
}

// endregion: --- Types

// region:    --- Public Functions

pub fn process_dir_info(path_str: &str, options: &Options) -> Result<DirInfo> {
	let mut total_size: u64 = 0;
	let mut total_numbers: u32 = 0;
	let mut top_files_collector: Vec<DirEntryInfo> = Vec::new(); // Initialize empty
	let mut min_of_tops = 0;

	if !options.summary {
		top_files_collector = Vec::with_capacity(options.nums + 1);
	}

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

	let mut by_ext_map_collector: Option<HashMap<String, u64>> = if !options.no_ext && !options.summary {
		Some(HashMap::new())
	} else {
		None
	};

	let entries = WalkDir::new(path_str).into_iter().filter_map(|e| e.ok()).filter(|e| {
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

			if let Some(current_by_ext_map) = &mut by_ext_map_collector {
				if let Some(ext) = entry_path.extension() {
					let ext_str = ext.to_string_lossy().to_string();
					*current_by_ext_map.entry(ext_str).or_insert(0) += size;
				}
			}

			if !options.summary && min_of_tops < size {
				match SPath::from_std_path(entry.path()) {
					Ok(spath) => {
						top_files_collector.push(DirEntryInfo { path: spath, size });
						top_files_collector.sort_by(|a, b| b.size.cmp(&a.size));
						if top_files_collector.len() > options.nums {
							top_files_collector.pop();
						}
						min_of_tops = top_files_collector.last().map(|e| e.size).unwrap_or(0);
					}
					Err(_) => {
						// Silently skip if path conversion fails, maintaining original behavior.
					}
				}
			}
		}
	}

	let final_ext_stats = if let Some(mut map) = by_ext_map_collector {
		let mut ext_stats_list: Vec<ExtStat> = map.drain().map(|(ext, size)| ExtStat { ext, size }).collect();
		ext_stats_list.sort_by(|a, b| b.size.cmp(&a.size));

		let mut top_by_ext_result = Vec::new();
		let mut others_size_agg = 0;

		for (i, ext_stat) in ext_stats_list.into_iter().enumerate() {
			if i < options.nums {
				top_by_ext_result.push(ext_stat);
			} else {
				others_size_agg += ext_stat.size;
			}
		}
		Some(ExtStats {
			top_by_ext: top_by_ext_result,
			others_size: others_size_agg,
		})
	} else {
		None
	};

	Ok(DirInfo {
		path_processed: path_str.to_string(),
		total_numbers,
		total_size,
		top_files: top_files_collector,
		ext_stats: final_ext_stats,
	})
}

// endregion: --- Public Functions
