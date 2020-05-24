use crate::blocks::{Block, Configure, Message, Sender};
use crate::utils;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
pub struct Brightness {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_name() -> String {
	"brightness".to_string()
}

fn default_period() -> f32 {
	1.0
}

fn default_update_signal() -> i32 {
	signal_hook::SIGUSR1
}

impl Configure for Brightness {}

impl Sender for Brightness {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn add_sender(&self, s: crossbeam_channel::Sender<Message>) {
		let name = self.name.clone();
		let mut block = Block::new(name.clone(), true);
		let mut monitor = utils::monitor_command("brightnessctl", &["g"], self.period);
		let recv = utils::wait_for_signal(self.update_signal, self.period);

		thread::spawn(move || loop {
			let output = monitor.read();
			block.full_text = Some(if let Ok(num) = utils::str_to_f32(&output) {
				format!(" {:.0}%", num / 75.0)
			} else {
				output
			});
			s.send((name.clone(), block.to_string())).unwrap();
			recv.recv().unwrap();
		});
	}
}
