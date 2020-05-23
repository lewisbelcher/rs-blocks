use crate::blocks::{Block, Configure, Message, Sender};
use crate::utils;
use std::thread;

#[derive(Deserialize)]
pub struct Volume {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_name() -> String {
	"volume".to_string()
}

fn default_period() -> u64 {
	1000
}

fn default_period() -> i32 {
	signal_hook::SIGUSR2
}

impl Configure for Volume {
	fn new(config: String) -> Volume {
		toml::from_str(&config).expect("Invalid config for block 'time'")
	}
}

impl Sender for Volume {
	pub fn add_sender(&self, s: crossbeam_channel::Sender<Message>) {
		let re = regex::Regex::new(r"\[(?P<percent>\d+%)\] \[(?P<status>on|off)\]").unwrap();
		let mut block = Block::new(self.name.clone(), true);
		let mut monitor = utils::monitor_command("amixer", &[], self.period);
		let recv = utils::wait_for_signal(self.update_signal, self.period);

		thread::spawn(move || loop {
			let output = monitor.read();
			block.full_text = Some(if let Some(captures) = re.captures(&output) {
				if captures.name("status").unwrap().as_str() == "on" {
					format!(" {}", captures.name("percent").unwrap().as_str())
				} else {
					"".to_string()
				}
			} else {
				output
			});
			s.send((name, block.to_string())).unwrap();
			recv.recv().unwrap();
		});
	}
}
