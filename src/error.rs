use derive_more::{Display, From};

#[derive(Debug, From, Display)]
pub enum AppError {
	#[display("-n must be a number but was {}", _0)]
	InvalidNumberOfFiles(String),

	// -- Externals
	#[from]
	Clap(clap::Error),
}
