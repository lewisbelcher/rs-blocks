use std::collections::HashMap;

use rs_blocks::blocks::{cpu, memory, network, time, battery};

const BATTERY: &str = "battery";
const CPU: &str = "cpu";
const MEMORY: &str = "memory";
const NETWORK: &str = "network";
const TIME: &str = "time";

fn main() {
	let (s, r) = crossbeam_channel::unbounded();
	let mut blocks = HashMap::new();

	battery::add_sender(BATTERY, s.clone());
	cpu::add_sender(CPU, s.clone());
	memory::add_sender(MEMORY, s.clone());
	network::add_sender(NETWORK, s.clone());
	time::add_sender(TIME, s.clone());
	let order = [NETWORK, MEMORY, CPU, BATTERY, TIME];

	println!("{{\"version\":1,\"click_events\":true}}");
	println!("[");
	loop {
		let (name, block) = r.recv().unwrap();
		blocks.insert(name, block);

		print_blocks(&blocks, &order);
	}
}

/// Print all blocks in a JSON array.
fn print_blocks(blocks: &HashMap<&str, String>, order: &[&str; 5]) {
	let mut first = true;
	print!("[");
	for name in order.iter() {
		if let Some(block) = blocks.get(name) {
			if !first {
				print!(",");
			}
			print!("{}", block);
			first = false;
		}
	}
	println!("],");
}
