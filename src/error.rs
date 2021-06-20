use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
	#[error("-n must be a number but was {0}")]
	InvalidNumberOfFiles(String),
}
