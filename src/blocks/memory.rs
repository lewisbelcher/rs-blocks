use crate::blocks::Block;
use crate::{ema, file};
use regex::Regex;
use std::thread;

const ALPHA: f32 = 0.5;
const PERIOD: u64 = 1000; // Monitor interval in ms
const PATTERN: &str = r"(?s)MemTotal:\s+(\d+).+MemFree:\s+(\d+)";
const MEMPATH: &str = "/proc/meminfo";

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

pub fn add_sender(name: &'static str, s: crossbeam_channel::Sender<(&'static str, String)>) {
	let monitor = file::MonitorFile::new(MEMPATH, PERIOD);
	let mut mem = ema::Ema::new(ALPHA);
	let mut block = Block::new(name, true);

	thread::spawn(move || {
		for c in monitor {
			let perc = get_mem_percentage(match_mem_stats(&c));
			block.full_text = Some(format!(" {:.1}%", mem.push(perc) * 100.0));
			s.send((name, block.to_string())).unwrap();
		}
	});
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
