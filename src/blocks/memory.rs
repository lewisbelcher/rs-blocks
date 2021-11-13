// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! # Memory block
//!
//! Use this block to get memory monitoring in the status bar. Reads from
//! `/proc/meminfo` to calculate memory usage.
//!
//! Typical configuration:
//!
//! ```toml
//! [memory]
//! ```
//!
//! ## Configuration options
//!
//! - `name`: Name of the block (must be unique)
//! - `period`: Default update period in seconds (extra updates may occur on
//!    event changes etc)
//! - `alpha`: Weight for the exponential moving average of value updates

use crate::blocks::{Block, Configure, Message, Sender};
use crate::{ema, utils};
use regex::Regex;
use serde::Deserialize;
use std::thread;

const MEMPATH: &str = "/proc/meminfo";
const PATTERN: &str = r"(?s)MemTotal:\s+(\d+).+MemFree:\s+(\d+)";

#[derive(Configure, Deserialize)]
pub struct Memory {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_alpha")]
	alpha: f32,
}

fn default_name() -> String {
	"memory".to_string()
}

fn default_period() -> f32 {
	1.0
}

fn default_alpha() -> f32 {
	0.5
}

impl Sender for Memory {
	fn add_sender(&self, channel: crossbeam_channel::Sender<Message>) -> anyhow::Result<()> {
		let name = self.get_name();
		let monitor = utils::monitor_file(MEMPATH.to_string(), self.period);
		let mut mem = ema::Ema::new(self.alpha);
		let mut block = Block::new(name.clone(), true);

		thread::spawn(move || {
			for text in monitor {
				let perc = get_mem_percentage(match_mem_stats(&text));
				block.full_text = Some(format!(" {:.1}%", mem.push(perc) * 100.0));
				channel.send((name.clone(), block.to_string())).unwrap();
			}
		});

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
struct MemStats {
	total: f64,
	free: f64,
}

fn match_mem_stats(s: &str) -> MemStats {
	lazy_static! {
		static ref RE: Regex = Regex::new(PATTERN).unwrap();
	}
	let caps = RE.captures(s).unwrap();
	MemStats {
		total: caps.get(1).unwrap().as_str().parse().unwrap(),
		free: caps.get(2).unwrap().as_str().parse().unwrap(),
	}
}

fn get_mem_percentage(mem: MemStats) -> f32 {
	1.0 - mem.free as f32 / mem.total as f32
}

#[cfg(test)]
mod tests {
	use super::*;
	const MEMFILE: &str = "MemTotal:  16134372 kB\nMemFree:  2757408 kB\n";

	#[test]
	fn regex_matches() {
		let mem = match_mem_stats(&MEMFILE);
		assert_eq!(
			mem,
			MemStats {
				total: 16134372.0,
				free: 2757408.0
			}
		);
	}
}
