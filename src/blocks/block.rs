use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Block {
	#[serde(default = "gghh")]
	background: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	color: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	full_text: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	markup: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	separator: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	separator_block_width: Option<usize>,
}

fn gghh() -> Option<String> {
	Some("a".to_string())
}

impl Block {
	pub fn new() -> Block {
		Block {
			background: None,
			color: None,
			full_text: None,
			markup: None,
			name: None,
			separator: None,
			separator_block_width: None,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn create_json() {
		let x = serde_json::to_string(&Block::new()).unwrap();
		println!("{}", x);
	}
}
