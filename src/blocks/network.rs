// Copyright ⓒ 2019-2020 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use crate::blocks::{Block, Configure, Message, Sender};
use crate::utils;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
pub struct Network {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	device: String,
	rx_file: Option<String>,
	tx_file: Option<String>,
}

fn default_name() -> String {
	"network".to_string()
}

fn default_period() -> f32 {
	1.0
}

fn get_network_file(device: &str, direction: &str) -> String {
	format!("/sys/class/net/{}/statistics/{}_bytes", device, direction)
}

impl Configure for Network {
	fn post_deserialise(mut instance: Self) -> Self
	where
		Self: Sized,
	{
		instance.rx_file = Some(get_network_file(&instance.device, "rx"));
		instance.tx_file = Some(get_network_file(&instance.device, "tx"));
		instance
	}

	fn get_name(&self) -> String {
		self.name.clone()
	}
}

impl Sender for Network {
	fn add_sender(&self, s: crossbeam_channel::Sender<Message>) {
		let name = self.get_name();
		let rx_file = utils::monitor_file(self.rx_file.as_ref().unwrap().clone(), self.period);
		let mut tx_file = utils::monitor_file(self.tx_file.as_ref().unwrap().clone(), self.period);
		let coef = 1.0 / (self.period * 1024.0); // Report in kB
		let mut rx = Speed::new(coef);
		let mut tx = Speed::new(coef);
		let mut first = true;
		let mut block = Block::new(name.clone(), true);

		thread::spawn(move || {
			for rx_ in rx_file {
				rx.push(utils::str_to_f32(&rx_).unwrap());
				tx.push(utils::str_to_f32(&tx_file.read()).unwrap());

				if first {
					first = false;
				} else {
					block.full_text = Some(format!(
						"<span foreground='#ccffcc'>\u{f0ab} {:.1}</span> <span foreground='#ffcccc'>\u{f0aa} {:.1}</span>",
						rx.calc_speed(),
						tx.calc_speed()
					));
					s.send((name.clone(), block.to_string())).unwrap();
				}
			}
		});
	}
}

struct Speed {
	curr: f32,
	prev: f32,
	coef: f32,
}

impl Speed {
	fn new(coef: f32) -> Speed {
		Speed {
			curr: 0.0,
			prev: 0.0,
			coef,
		}
	}

	fn push(&mut self, new: f32) {
		self.prev = self.curr;
		self.curr = new;
	}

	fn calc_speed(&self) -> f32 {
		(self.curr - self.prev) * self.coef
	}
}
