// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! # Network block
//!
//! Use this block to get network monitoring in the status bar.
//!
//! Typical configuration:
//!
//! ```toml
//! [network]
//! path_to_rx = "/sys/class/net/wlan0/statistics/rx_bytes"
//! path_to_tx = "/sys/class/net/wlan0/statistics/tx_bytes"
//! ```
//!
//! ## Configuration options
//!
//! - `name`: Name of the block (must be unique)
//! - `period`: Default update period in seconds (extra updates may occur on
//!    event changes etc)
//! - `alpha`: Weight for the exponential moving average of value updates
//! - `path_to_rx`: Path to the file to monitor for network receiving traffic
//!    (usually something like `/sys/class/net/<DEVICE>/statistics/rx_bytes`
//!    where `<DEVICE>` is the network device to monitor)
//! - `path_to_tx`: Path to the file to monitor for network transmission traffic
//!    (usually something like `/sys/class/net/<DEVICE>/statistics/tx_bytes`
//!    where `<DEVICE>` is the network device to monitor)

use crate::blocks::{Block, Configure, Message, Sender, ValidatedPath};
use crate::utils;
use serde::Deserialize;
use std::thread;

#[derive(Configure, Deserialize)]
pub struct Network {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	path_to_rx: ValidatedPath,
	path_to_tx: ValidatedPath,
}

fn default_name() -> String {
	"network".to_string()
}

fn default_period() -> f32 {
	1.0
}

impl Sender for Network {
	fn add_sender(&self, channel: crossbeam_channel::Sender<Message>) -> anyhow::Result<()> {
		let name = self.get_name();
		let rx_file = utils::monitor_file(self.path_to_rx.0.clone(), self.period);
		let mut tx_file = utils::monitor_file(self.path_to_tx.0.clone(), self.period);
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
					channel.send((name.clone(), block.to_string())).unwrap();
				}
			}
		});

		Ok(())
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
