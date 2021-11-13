// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! Base implementation and traits for all blocks.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::path::Path;

/// The type sent by a block to the main thread.
pub type Message = (String, String);

#[derive(Serialize)]
pub struct Block {
	pub name: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub background: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub colour: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub full_text: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub markup: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub separator: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub separator_block_width: Option<usize>,
}

impl Block {
	pub fn new(name: String, pango: bool) -> Block {
		Block {
			name,
			background: None,
			colour: None,
			full_text: None,
			markup: if pango {
				Some("pango".to_string())
			} else {
				None
			},
			separator: None,
			separator_block_width: Some(18),
		}
	}

	pub fn to_string(&self) -> String {
		if let Ok(s) = serde_json::to_string(self) {
			s
		} else {
			format!("Error in '{}'", self.name)
		}
	}
}

/// Configure trait for blocks.
///
/// Configuration is in toml format sent as a string so that each block
/// sender can deserialise it in its own way.
pub trait Configure {
	fn new<'a>(config: &'a str) -> anyhow::Result<Self>
	where
		Self: Sized + Deserialize<'a>,
	{
		toml::from_str(config).context(format!("Invalid config block '{}'", config))
	}

	fn get_name(&self) -> String;
}

/// Sender trait for blocks.
///
/// A block must implement creating a closure which sends messages over a
/// channel when new updates for publishing are ready.
pub trait Sender: Configure {
	fn add_sender(&self, channel: crossbeam_channel::Sender<Message>) -> anyhow::Result<()>;
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct ValidatedPath(pub String);

impl TryFrom<String> for ValidatedPath {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		if Path::new(&value).exists() {
			Ok(ValidatedPath(value))
		} else {
			Err(format!("Path '{}' does not exist", value))
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn create_json() {
		Block::new("hi".to_string(), true).to_string();
	}
}
