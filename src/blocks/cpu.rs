#[macro_use]
extern crate lazy_static;

use regex::Regex;
use rs_blocks::{self, ema, file};

const ALPHA: f32 = 0.7;
const PERIOD: u64 = 1000; // Monitor interval in ms
const PATTERN: &str = r"cpu\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)";
const PATH: &str = "/proc/stat";

fn calc_cpu(stat: regex::Captures) -> Cpu {
	// (user, nice, system, idle, iowait, irq, softirq)
	let stats: Vec<f32> = stat
		.iter()
		.skip(1)
		.map(|x| x.unwrap().as_str().parse().unwrap())
		.collect();

	Cpu {
		idle: (stats[3] + stats[4]),
		total: stats.iter().sum(),
	}
}

fn calc_dcpu(cpu: &Cpu, prevcpu: &Cpu) -> f32 {
	(1.0 - (cpu.idle - prevcpu.idle) / (cpu.total - prevcpu.total)) * 100.0
}

struct Cpu {
	idle: f32,
	total: f32,
}

fn match_proc(s: &str) -> regex::Captures {
	lazy_static! {
		static ref RE: Regex = Regex::new(PATTERN).unwrap();
	}
	RE.captures(s).unwrap()
}

fn main() {
	let monitor = file::MonitorFile::new(PATH, PERIOD);
	let mut perc = ema::Ema::new(ALPHA);
	let mut cpu = Cpu {
		idle: 0.0,
		total: 0.0,
	};

	for c in monitor {
		let current_cpu = calc_cpu(match_proc(&c));
		println!(" {:.1}%", perc.push(calc_dcpu(&current_cpu, &cpu)));
		cpu = current_cpu;
	}
}
