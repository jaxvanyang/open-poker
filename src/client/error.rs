use std::fmt::Display;

use anyhow::anyhow;

pub fn anyhow_error(err: impl Display) -> anyhow::Error {
	anyhow!(err.to_string())
}
