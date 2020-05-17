#[macro_use]
extern crate lazy_static;

use regex::Regex;
use rs_blocks::{file, ema};

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

fn main() {
	let monitor = file::MonitorFile::new(MEMPATH, PERIOD);
	let mut mem = ema::Ema::new(ALPHA);

	for c in monitor {
		let perc = get_mem_percentage(match_mem_stats(&c));
		println!(" {:.1}%", mem.push(perc) * 100.0);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	const MEMFILE: &str = "MemTotal:  16134372 kB\nMemFree:  2757408 kB\n";

	// An old implementation for benchmarking that does not use lazy static
	fn match_mem_stats_old(s: &str) -> MemStats {
		let re = Regex::new(PATTERN).unwrap();
		let caps = re.captures(s).unwrap();
		MemStats {
			total: caps.get(1).unwrap().as_str().parse().unwrap(),
			free: caps.get(2).unwrap().as_str().parse().unwrap(),
		}
	}

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

// 	#[bench]
// 	fn bench_match_mem_stats(b: &mut Bencher) {
// 		b.iter(|| match_mem_stats(&MEMFILE));
// 	}
// 
// 	#[bench]
// 	fn bench_match_mem_stats_old(b: &mut Bencher) {
// 		b.iter(|| match_mem_stats_old(&MEMFILE));
// 	}
}
