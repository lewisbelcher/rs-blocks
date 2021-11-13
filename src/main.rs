// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use anyhow::Context;
use rs_blocks::args;
use rs_blocks::blocks::{
	battery, brightness, cpu, memory, network, time, volume, Configure, Sender,
};
use std::collections::HashMap;
use std::fs;

const DEFAULT_CONFIG: &'static str = r#"
[time]
format = "%a %d %b <b>%H:%M:%S</b>"
period = 1
"#;

fn main() -> anyhow::Result<()> {
	env_logger::init();
	let cmd_args = args::collect();
	let config = if let Some(path) = cmd_args.config {
		fs::read_to_string(&path)
			.context(format!("Failed to read config file '{}'", path.display()))?
	} else {
		DEFAULT_CONFIG.to_string()
	};

	let (s, r) = crossbeam_channel::unbounded();
	let mut order = Vec::new();

	for (block_type, config) in parse_config(&config) {
		let sender = create_sender(&block_type, config.to_string())?;
		order.push(sender.get_name());
		sender.add_sender(s.clone())?;
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

/// Create a sender object for a given config.
fn create_sender(name: &str, config: String) -> anyhow::Result<Box<dyn Sender>> {
	match name {
		"battery" => Ok(Box::new(battery::Battery::new(&config)?)),
		"brightness" => Ok(Box::new(brightness::Brightness::new(&config)?)),
		"cpu" => Ok(Box::new(cpu::Cpu::new(&config)?)),
		"memory" => Ok(Box::new(memory::Memory::new(&config)?)),
		"network" => Ok(Box::new(network::Network::new(&config)?)),
		"time" => Ok(Box::new(time::Time::new(&config)?)),
		"volume" => Ok(Box::new(volume::Volume::new(&config)?)),
		_ => {
			anyhow::bail!("Unrecognised config element '{}'", name)
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
