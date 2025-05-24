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
/// - Values are scaled to the largest appropriate unit (KB, MB, GB, TB, PB).
/// - For KB, MB, GB, and TB, the numerical part is formatted to ensure it's less than 1000.0
///   when displayed with one decimal place (e.g., "999.9 KB", not "1000.0 KB").
///   If a value would round to 1000.0 or more (e.g., 999.95 KB), it's scaled up (e.g., to "  1.0 MB").
///   This keeps the numerical part for these units within a 5-character width (e.g., "ddd.d").
/// - For values that scale to less than 1.0 KB (i.e., 0 to 999 bytes):
///   - Displayed with three decimal places, right-aligned to a 7-character width for the number part.
///   - Example: `512 bytes` -> `"  0.512 KB"` (total 10 characters including unit and space)
/// - For values that scale to 1.0 or more of a unit (KB, MB, GB, TB) but less than `UNIT_THRESHOLD_EXCLUSIVE` of that unit:
///   - Displayed with one decimal place, right-aligned to a 5-character width for the number part.
///   - Example: `1500 bytes` (1.5 KB) -> `"  1.5 KB"` (total 8 characters)
///   - Example: `1_200_000 bytes` (1.2 MB) -> `"  1.2 MB"` (total 8 characters)
/// - For the PB unit, the numerical part can exceed 5 characters if the value is large enough (e.g., "1234.5 PB").
/// - `0 bytes` is a special case, displayed as `"  0.000 KB"`.
///
/// Byte to Unit scaling summary:
/// - `0` to `999_949` bytes are typically scaled to KB.
///   - `0` to `999` bytes: formatted with 3 decimal places (e.g., `"  0.000 KB"` to `"  0.999 KB"`)
///   - `1_000` to `999_949` bytes: formatted with 1 decimal place (e.g., `"  1.0 KB"` to `"999.9 KB"`).
/// - Values from `999_950` bytes (0.99995 MB) upwards are scaled to MB, then GB, TB, PB accordingly.
///   If they are less than `UNIT_THRESHOLD_EXCLUSIVE` of that unit, they use one decimal place.
///   Otherwise, they scale to the next unit.
///   - Example: `999_950 bytes` (0.99995 MB) displays as `"  1.0 MB"`.
///   - Example: `999_940_000 bytes` (999.94 MB) displays as `"999.9 MB"`.
///   - Example: `999_950_000 bytes` (999.95 MB) displays as `"  1.0 GB"`.
pub fn format_size(bytes: u64) -> String {
	let size_f64 = bytes as f64;

	let (num_str, unit_str) = if bytes == 0 {
		(format!("{:>7.3}", 0.0), "KB".to_string()) // e.g., "  0.000 KB"
	} else {
		let val_kb = size_f64 / KB_SIZE_F64;
		if val_kb < 1.0 {
			// Values from 0.001 KB up to 0.999... KB
			(format!("{:>7.3}", val_kb), "KB".to_string()) // e.g., "  0.001 KB", "  0.999 KB"
		} else if val_kb < UNIT_THRESHOLD_EXCLUSIVE {
			// Values from 1.0 KB up to 999.94... KB
			(format!("{:>5.1}", val_kb), "KB".to_string()) // e.g., "  1.0 KB", "999.9 KB"
		} else {
			// Values >= 999.95 KB, switch to MB (effectively >= 1.0 MB)
			let val_mb = size_f64 / MB_SIZE_F64;
			if val_mb < UNIT_THRESHOLD_EXCLUSIVE {
				// Values from approx 1.0 MB up to 999.94... MB
				(format!("{:>5.1}", val_mb), "MB".to_string())
			} else {
				// Values >= 999.95 MB, switch to GB (effectively >= 1.0 GB)
				let val_gb = size_f64 / GB_SIZE_F64;
				if val_gb < UNIT_THRESHOLD_EXCLUSIVE {
					// Values from approx 1.0 GB up to 999.94... GB
					(format!("{:>5.1}", val_gb), "GB".to_string())
				} else {
					// Values >= 999.95 GB, switch to TB (effectively >= 1.0 TB)
					let val_tb = size_f64 / TB_SIZE_F64;
					if val_tb < UNIT_THRESHOLD_EXCLUSIVE {
						// Values from approx 1.0 TB up to 999.94... TB
						(format!("{:>5.1}", val_tb), "TB".to_string())
					} else {
						// Values >= 999.95 TB, switch to PB (effectively >= 1.0 PB)
						let val_pb = size_f64 / PB_SIZE_F64;
						(format!("{:>5.1}", val_pb), "PB".to_string()) // Always 1 decimal for PB, can exceed 5 chars
					}
				}
			}
		}
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
			// Byte range (0 to 999 bytes), displayed with 3 decimal places in KB
			(0, "  0.000 KB"),
			(1, "  0.001 KB"),   // 0.001 KB
			(49, "  0.049 KB"),  // 0.049 KB
			(50, "  0.050 KB"),  // 0.050 KB
			(500, "  0.500 KB"), // 0.500 KB
			(512, "  0.512 KB"), // 0.512 KB
			(949, "  0.949 KB"), // 0.949 KB
			(950, "  0.950 KB"), // 0.950 KB
			(999, "  0.999 KB"), // 0.999 KB
			// KB range (1.0 KB to 999.94... KB), displayed with 1 decimal place
			(1_000, "  1.0 KB"),   // Exactly 1.0 KB
			(1_500, "  1.5 KB"),   // 1.5 KB
			(512_000, "512.0 KB"), // 512.0 KB
			(999_900, "999.9 KB"), // 999.9 KB
			(999_940, "999.9 KB"), // 999.94 KB rounds to 999.9 KB
			// Values >= 999.95 KB transition to MB
			(999_949, "999.9 KB"), // 999.949 KB rounds to 999.9 KB
			(999_950, "  1.0 MB"), // 999.95 KB is displayed as 1.0 MB
			(999_999, "  1.0 MB"), // 999.999 KB is displayed as 1.0 MB
			// MB range and transitions
			(1_000_000, "  1.0 MB"),   // Exactly 1.0 MB
			(123_450_000, "123.5 MB"), // 123.45 MB rounds to 123.5 MB
			(129_000_000, "129.0 MB"),
			(999_940_000, "999.9 MB"), // 999.94 MB rounds to 999.9 MB
			(999_949_000, "999.9 MB"), // 999.949 MB rounds to 999.9 MB
			// Values >= 999.95 MB transition to GB
			(999_950_000, "  1.0 GB"), // 999.95 MB is displayed as 1.0 GB
			// GB range and transitions
			(1_000_000_000, "  1.0 GB"),
			(4_100_000_000, "  4.1 GB"),
			(999_940_000_000, "999.9 GB"), // 999.94 GB rounds to 999.9 GB
			// Values >= 999.95 GB transition to TB
			(999_950_000_000, "  1.0 TB"), // 999.95 GB is displayed as 1.0 TB
			// TB range and transitions
			(1_000_000_000_000, "  1.0 TB"),
			(999_940_000_000_000, "999.9 TB"), // 999.94 TB rounds to 999.9 TB
			// Values >= 999.95 TB transition to PB
			(999_950_000_000_000, "  1.0 PB"), // 999.95 TB is displayed as 1.0 PB
			// PB range
			(1_000_000_000_000_000, "  1.0 PB"),
			// Large PB values (number part might exceed 5 characters)
			(1_234_500_000_000_000_000, "1234.5 PB"),
			(u64::MAX, "18446.7 PB"), // u64::MAX is approx 18446.7 PB
		];

		// -- Exec & Check
		for (input_bytes, expected_str) in test_cases {
			assert_eq!(format_size(input_bytes), expected_str, "Input: {} bytes", input_bytes);
		}

		Ok(())
	}

	// region:    --- Support
	// (no support functions needed for this test)
	// endregion: --- Support
}

// endregion: --- Tests
