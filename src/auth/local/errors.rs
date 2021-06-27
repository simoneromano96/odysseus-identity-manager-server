use handlebars::RenderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LocalErrors {
	#[error("{0}")]
	HandlebarsError(#[from] RenderError),
}
