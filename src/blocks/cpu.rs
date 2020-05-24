use crate::blocks::Block;
use crate::{ema, utils};
use regex::Regex;
use std::thread;

const ALPHA: f32 = 0.7;
const PERIOD: f32 = 1.0;
const PATTERN: &str = r"cpu\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)";
const PATH: &str = "/proc/stat";

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
	RE.captures(s).unwrap()
}

pub fn add_sender(
	name: &'static str,
	s: crossbeam_channel::Sender<(&'static str, String)>,
) -> &'static str {
	let monitor = utils::monitor_file(PATH.to_string(), PERIOD);
	let mut perc = ema::Ema::new(ALPHA);
	let mut cpu = Usage {
		idle: 0.0,
		total: 0.0,
	};
	let mut block = Block::new(name, true);

	thread::spawn(move || {
		for c in monitor {
			let current_cpu = calc_cpu(match_proc(&c));
			block.full_text = Some(format!(
				" {:.1}%",
				perc.push(calc_dcpu(&current_cpu, &cpu))
			));
			s.send((name, block.to_string())).unwrap();
			cpu = current_cpu;
		}
	});
	name
}
