use crate::blocks::Block;
use crate::{file, utils};
use std::thread;

const PERIOD: u64 = 1000; // Monitor interval in ms
const DEVICE: &str = "wlp2s0";

fn get_network_file(direction: &str) -> String {
	format!("/sys/class/net/{}/statistics/{}_bytes", DEVICE, direction)
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

pub fn add_sender(name: &'static str, s: crossbeam_channel::Sender<(&'static str, String)>) {
	let rx_file = file::MonitorFile::new(&get_network_file("rx"), PERIOD);
	let mut tx_file = file::MonitorFile::new(&get_network_file("tx"), PERIOD);

	let coef = 1.0 / (PERIOD as f32 * 1.024); // Report in kB
	let mut rx = Speed::new(coef);
	let mut tx = Speed::new(coef);
	let mut first = true;
	let mut block = Block::new(name, true);

	thread::spawn(move || {
		for rx_ in rx_file {
			rx.push(utils::str_to_f32(&rx_));
			tx.push(utils::str_to_f32(&tx_file.read()));

			if first {
				first = false;
			} else {
				block.full_text = Some(format!(
					"<span foreground='#ccffcc'>\u{f0ab} {:.1}</span> <span foreground='#ffcccc'>\u{f0aa} {:.1}</span>",
					rx.calc_speed(),
					tx.calc_speed()
				));
				s.send((name, block.to_string())).unwrap();
			}
		}
	});
}
