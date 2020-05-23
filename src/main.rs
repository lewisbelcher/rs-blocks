// use rs_blocks::blocks::{battery, brightness, cpu, memory, network, time, volume};
use rs_blocks::blocks::{time, Configure, Sender};
use std::collections::HashMap;
use std::process;

fn main() {
	let (s, r) = crossbeam_channel::unbounded();
	let ss = r#"
	[time]
	format = "%a %d %b <b>%H:%M:%S</b>"
	"#;
	let mut order = Vec::new();

	for (block_type, config) in parse_config(ss) {
		let sender = get_struct(&block_type, config.to_string());
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

fn get_struct(name: &str, config: String) -> Box<dyn Sender> {
	Box::new(match name {
		"time" => time::Time::new(config),
		_ => {
			eprintln!("Unrecognised config element '{}'", name);
			process::exit(1);
		}
	})
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
