// Copyright ⓒ 2019-2020 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use crate::blocks::{Block, Configure, Message, Sender};
use crate::utils;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
pub struct Volume {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_name() -> String {
	"volume".to_string()
}

fn default_period() -> f32 {
	10.0
}

fn default_update_signal() -> i32 {
	signal_hook::SIGUSR2
}

impl Configure for Volume {}

impl Sender for Volume {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn add_sender(&self, s: crossbeam_channel::Sender<Message>) {
		let re = regex::Regex::new(r"\[(?P<percent>\d+%)\] \[(?P<status>on|off)\]").unwrap();
		let mut block = Block::new(self.name.clone(), true);
		let mut monitor = utils::monitor_command("amixer", &[], self.period);
		let recv = utils::wait_for_signal(self.update_signal, self.period);
		let name = self.get_name();

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
			s.send((name.clone(), block.to_string())).unwrap();
			recv.recv().unwrap();
		});
	}
}
