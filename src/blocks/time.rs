use crate::blocks::{Block, Configure, Message, Sender};
use chrono::prelude::*;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
pub struct Time {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_format")]
	format: String,
	#[serde(default = "default_period")]
	period: u64,
}

fn default_name() -> String {
	"time".to_string()
}

fn default_format() -> String {
	"%a %d %b <b>%H:%M:%S</b>".to_string()
}

fn default_period() -> u64 {
	1000
}

impl Configure for Time {
	fn new(config: String) -> Time {
		toml::from_str(&config).expect("Invalid config for block 'time'")
	}
}

impl Sender for Time {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn add_sender(&self, s: crossbeam_channel::Sender<Message>) {
		let name = self.name.clone();
		let format = self.format.clone();
		let period = self.period;
		let mut block = Block::new(name.clone(), true);

		thread::spawn(move || loop {
			block.full_text = Some(Local::now().format(&format).to_string());
			s.send((name.clone(), block.to_string())).unwrap();
			thread::sleep(Duration::from_millis(period));
		});
	}
}
