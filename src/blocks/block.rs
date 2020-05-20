use serde::Serialize;

#[derive(Serialize)]
pub struct Block {
	pub name: &'static str,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub background: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub color: Option<String>,
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
	pub fn new(name: &'static str, pango: bool) -> Block {
		Block {
			name,
			background: None,
			color: None,
			full_text: None,
			markup: if pango {
				Some("pango".to_string())
			} else {
				None
			},
			separator: None,
			separator_block_width: Some(3),
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

#[cfg(test)]
mod test {
	use super::*;
	use serde_json;

	#[test]
	fn create_json() {
		serde_json::to_string(&Block::new()).unwrap();
	}
}
