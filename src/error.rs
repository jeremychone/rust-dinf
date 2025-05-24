use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
	#[display("-n must be a number but was {}", _0)]
	InvalidNumberOfFiles(String),

	PathNotUtf8(String),

	// -- Externals
	#[from]
	Io(std::io::Error),
	#[from]
	Clap(clap::Error),
	#[from]
	Glob(globset::Error),
	#[from]
	Walkdir(walkdir::Error),
}

impl std::error::Error for Error {}

