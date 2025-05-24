use num_format::ToFormattedString;

pub fn format_num(num: impl ToFormattedString) -> String {
	num.to_formatted_string(&num_format::Locale::en)
}
