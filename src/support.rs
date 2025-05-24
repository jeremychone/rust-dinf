use num_format::ToFormattedString;

pub fn format_num(num: impl ToFormattedString) -> String {
	num.to_formatted_string(&num_format::Locale::en)
}

// region:    --- Size Formatting

const KB_SIZE_F64: f64 = 1000.0;
const MB_SIZE_F64: f64 = KB_SIZE_F64 * 1000.0;
const GB_SIZE_F64: f64 = MB_SIZE_F64 * 1000.0;
const TB_SIZE_F64: f64 = GB_SIZE_F64 * 1000.0;
const PB_SIZE_F64: f64 = TB_SIZE_F64 * 1000.0;

/// Threshold for unit scaling. If a value in a given unit (KB, MB, GB, TB)
/// is `UNIT_THRESHOLD_EXCLUSIVE` or greater (e.g., 999.95), it would round
/// to 1000.0 or more when formatted to one decimal place. In such cases,
/// the value is scaled to the next larger unit.
const UNIT_THRESHOLD_EXCLUSIVE: f64 = 999.95;

/// Formats a byte size into a human-readable string using decimal prefixes (KB, MB, etc.).
///
/// The `left_align` parameter controls the alignment of the numerical part of the string.
///
pub fn format_size(bytes: u64, left_align: bool) -> String {
	let size_f64 = bytes as f64;

	let (value_to_format, unit_str, use_three_decimals) = if bytes == 0 {
		(0.0, "KB", true)
	} else {
		let val_kb = size_f64 / KB_SIZE_F64;
		if val_kb < 1.0 {
			(val_kb, "KB", true)
		} else if val_kb < UNIT_THRESHOLD_EXCLUSIVE {
			(val_kb, "KB", false)
		} else {
			let val_mb = size_f64 / MB_SIZE_F64;
			if val_mb < UNIT_THRESHOLD_EXCLUSIVE {
				(val_mb, "MB", false)
			} else {
				let val_gb = size_f64 / GB_SIZE_F64;
				if val_gb < UNIT_THRESHOLD_EXCLUSIVE {
					(val_gb, "GB", false)
				} else {
					let val_tb = size_f64 / TB_SIZE_F64;
					if val_tb < UNIT_THRESHOLD_EXCLUSIVE {
						(val_tb, "TB", false)
					} else {
						let val_pb = size_f64 / PB_SIZE_F64;
						(val_pb, "PB", false)
					}
				}
			}
		}
	};

	let num_str = if left_align {
		if use_three_decimals {
			format!("{:.3}", value_to_format)
		} else {
			format!("{:.1}", value_to_format)
		}
	} else if use_three_decimals {
		format!("{:>7.3}", value_to_format)
	} else {
		format!("{:>5.1}", value_to_format)
	};

	format!("{} {}", num_str, unit_str)
}

// endregion: --- Size Formatting

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*; // For format_size

	#[test]
	fn test_support_format_size_various() -> Result<()> {
		// -- Setup & Fixtures
		let test_cases = vec![
			// input_bytes, expected_padded (left_align: false), expected_left_aligned (left_align: true)
			// Byte range (0 to 999 bytes), 3 decimal places
			(0, "  0.000 KB", "0.000 KB"),
			(1, "  0.001 KB", "0.001 KB"),
			(49, "  0.049 KB", "0.049 KB"),
			(50, "  0.050 KB", "0.050 KB"),
			(500, "  0.500 KB", "0.500 KB"),
			(512, "  0.512 KB", "0.512 KB"),
			(949, "  0.949 KB", "0.949 KB"),
			(950, "  0.950 KB", "0.950 KB"),
			(999, "  0.999 KB", "0.999 KB"),
			// KB range (1.0 KB to 999.94... KB), 1 decimal place
			(1_000, "  1.0 KB", "1.0 KB"),
			(1_500, "  1.5 KB", "1.5 KB"),
			(512_000, "512.0 KB", "512.0 KB"), // "512.0" is 5 chars, no padding with `{:>5.1}`
			(999_900, "999.9 KB", "999.9 KB"), // "999.9" is 5 chars, no padding with `{:>5.1}`
			(999_940, "999.9 KB", "999.9 KB"), // 999.94 rounds to 999.9
			// Values >= 999.95 KB transition to MB
			(999_949, "999.9 KB", "999.9 KB"), // 999.949 KB rounds to 999.9 KB
			(999_950, "  1.0 MB", "1.0 MB"),   // 999.95 KB (0.99995 MB) displays as 1.0 MB
			(999_999, "  1.0 MB", "1.0 MB"),   // 999.999 KB (0.999999 MB) displays as 1.0 MB
			// MB range and transitions
			(1_000_000, "  1.0 MB", "1.0 MB"),
			(123_450_000, "123.5 MB", "123.5 MB"), // "123.5" is 5 chars
			(129_000_000, "129.0 MB", "129.0 MB"), // "129.0" is 5 chars
			(999_940_000, "999.9 MB", "999.9 MB"),
			(999_949_000, "999.9 MB", "999.9 MB"),
			// Values >= 999.95 MB transition to GB
			(999_950_000, "  1.0 GB", "1.0 GB"),
			// GB range and transitions
			(1_000_000_000, "  1.0 GB", "1.0 GB"),
			(4_100_000_000, "  4.1 GB", "4.1 GB"),
			(999_940_000_000, "999.9 GB", "999.9 GB"),
			// Values >= 999.95 GB transition to TB
			(999_950_000_000, "  1.0 TB", "1.0 TB"),
			// TB range and transitions
			(1_000_000_000_000, "  1.0 TB", "1.0 TB"),
			(999_940_000_000_000, "999.9 TB", "999.9 TB"),
			// Values >= 999.95 TB transition to PB
			(999_950_000_000_000, "  1.0 PB", "1.0 PB"),
			// PB range
			(1_000_000_000_000_000, "  1.0 PB", "1.0 PB"),
			// Large PB values (number part might exceed 5 characters)
			(1_234_500_000_000_000_000, "1234.5 PB", "1234.5 PB"), // "1234.5" > 5 chars
			(u64::MAX, "18446.7 PB", "18446.7 PB"),                // u64::MAX is approx 18446.7 PB, > 5 chars
		];

		// -- Exec & Check
		for (input_bytes, expected_padded, expected_left_aligned) in test_cases {
			assert_eq!(
				format_size(input_bytes, false),
				expected_padded,
				"Input: {} bytes, left_align: false (padded)",
				input_bytes
			);
			assert_eq!(
				format_size(input_bytes, true),
				expected_left_aligned,
				"Input: {} bytes, left_align: true (unpadded)",
				input_bytes
			);
		}

		Ok(())
	}

	// region:    --- Support
	// (no support functions needed for this test)
	// endregion: --- Support
}

// endregion: --- Tests
