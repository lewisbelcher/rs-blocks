use rs_blocks::args;
use rs_blocks::blocks::{
	battery, brightness, cpu, memory, network, time, volume, Configure, Sender,
};
use std::collections::HashMap;
use std::fs;
use std::process;

const DEFAULT_CONFIG: &'static str = r#"
[time]
format = "%a %d %b <b>%H:%M:%S</b>"
period = 1
"#;

fn main() {
	let cmd_args = args::collect();
	let config = if let Some(path) = cmd_args.config {
		fs::read_to_string(path).unwrap()
	} else {
		DEFAULT_CONFIG.to_string()
	};

	let (s, r) = crossbeam_channel::unbounded();
	let mut order = Vec::new();

	for (block_type, config) in parse_config(&config) {
		let sender = create_sender(&block_type, config.to_string());
		order.push(sender.get_name());
		sender.add_sender(s.clone());
	}

	let mut blocks = HashMap::new();
	println!("{{\"version\":1,\"click_events\":true}}");
	println!("[");
	loop {
		let (name, block) = r.recv().unwrap();
		blocks.insert(name, block);
		print_blocks(&blocks, &order);
	}
}

fn create_sender(name: &str, config: String) -> Box<dyn Sender> {
	match name {
		"battery" => Box::new(battery::Battery::new(&config)),
		"brightness" => Box::new(brightness::Brightness::new(&config)),
		"cpu" => Box::new(cpu::Cpu::new(&config)),
		"memory" => Box::new(memory::Memory::new(&config)),
		"network" => Box::new(network::Network::new(&config)),
		"time" => Box::new(time::Time::new(&config)),
		"volume" => Box::new(volume::Volume::new(&config)),
		_ => {
			eprintln!("Unrecognised config element '{}'", name);
			process::exit(1);
		}
	}
}

/// Return a name->body mapping of a config file. Config file must be in toml
/// format with only top-level tables.
fn parse_config(cfg: &str) -> Vec<(String, String)> {
	let re = regex::Regex::new(r"\[(?P<name>\w+)\]\s+(?P<body>[^\[]*|\z)").unwrap();
	re.captures_iter(cfg)
		.map(|x| (x["name"].to_string(), x["body"].to_string()))
		.collect()
}

/// Print all blocks in a JSON array.
fn print_blocks(blocks: &HashMap<String, String>, order: &[String]) {
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
