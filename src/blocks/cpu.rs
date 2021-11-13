// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! # CPU block
//!
//! Use this block to get CPU monitoring in the status bar.
//!
//! This block reads from `/proc/stat` to calculate CPU usage.
//!
//! Typical configuration:
//!
//! ```toml
//! [cpu]
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

const PATTERN: &str = r"cpu\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)";
const PATH: &str = "/proc/stat";

#[derive(Configure, Deserialize)]
pub struct Cpu {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_alpha")]
	alpha: f32,
}

fn default_name() -> String {
	"cpu".to_string()
}

fn default_period() -> f32 {
	1.0
}

fn default_alpha() -> f32 {
	0.7
}

impl Sender for Cpu {
	fn add_sender(&self, channel: crossbeam_channel::Sender<Message>) -> anyhow::Result<()> {
		let name = self.get_name();
		let monitor = utils::monitor_file(PATH.to_string(), self.period);
		let mut perc = ema::Ema::new(self.alpha);
		let mut cpu = Usage {
			idle: 0.0,
			total: 0.0,
		};
		let mut block = Block::new(name.clone(), true);

		thread::spawn(move || {
			for c in monitor {
				let current_cpu = calc_cpu(match_proc(&c));
				block.full_text = Some(format!(
					" {:.1}%",
					perc.push(calc_dcpu(&current_cpu, &cpu))
				));
				channel.send((name.clone(), block.to_string())).unwrap();
				cpu = current_cpu;
			}
		});

		Ok(())
	}
}

struct Usage {
	idle: f32,
	total: f32,
}

fn calc_cpu(stat: regex::Captures) -> Usage {
	// (user, nice, system, idle, iowait, irq, softirq)
	let stats: Vec<f32> = stat
		.iter()
		.skip(1)
		.map(|x| x.unwrap().as_str().parse().unwrap())
		.collect();

	Usage {
		idle: (stats[3] + stats[4]),
		total: stats.iter().sum(),
	}
}

fn calc_dcpu(cpu: &Usage, prevcpu: &Usage) -> f32 {
	(1.0 - (cpu.idle - prevcpu.idle) / (cpu.total - prevcpu.total)) * 100.0
}

fn match_proc(s: &str) -> regex::Captures {
	lazy_static! {
		static ref RE: Regex = Regex::new(PATTERN).unwrap();
	}
	RE.captures(s)
		.expect(&format!("Failed to match /proc/stat contents '{}'", s))
}

#[cfg(test)]
mod tests {
	use super::*;
	const STATFILE: &str = "cpu  237476 0 85111 17267319 2310 34402 4846 0 0 0\n";

	#[test]
	fn regex_matches() {
		match_proc(&STATFILE);
	}
}
