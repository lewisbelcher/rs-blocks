use crate::blocks::{Block, Configure, Message as Msg, Sender};
use crate::{ema, utils};
use serde::Deserialize;
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const PATH: &str = "/sys/class/power_supply/BAT0";

#[derive(Deserialize)]
pub struct Battery {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_alpha")]
	alpha: f32,
}

fn default_name() -> String {
	"battery".to_string()
}

fn default_period() -> f32 {
	0.6
}

fn default_alpha() -> f32 {
	0.8
}

impl Configure for Battery {}

impl Sender for Battery {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn add_sender(&self, s: crossbeam_channel::Sender<Msg>) {
		let name = self.name.clone();
		let max = get_max_capacity();
		let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
		let mut sremain = "...".to_string();
		let mut last_status_change = 0;
		let mut remaining = ema::Ema::new(self.alpha);
		let (mut current_charge, mut current_status) = initialise(self.period, tx);
		let mut then = Instant::now();
		let mut percent = current_charge / max;
		let mut block = Block::new(name.clone(), true);
		let mut symbol = get_symbol(current_status, percent);

		if current_status == Status::Full {
			block.full_text = Some(create_full_text(&symbol, percent, "Full"));
			s.send((name.clone(), block.to_string())).unwrap();
		}

		thread::spawn(move || loop {
			let message = rx.recv().unwrap();
			let now = Instant::now();

			match message {
				Message::Charge(charge) => {
					sremain = if last_status_change == 0 {
						remaining.reset();
						"...".to_string()
					} else {
						let elapsed = now.duration_since(then).as_secs() as f32 / 60.0;
						let gap = match current_status {
							Status::Charging => max - charge,
							Status::Discharging => charge,
							Status::Full => 0.0,
							Status::Unknown => charge,
						};
						let rate = (charge - current_charge).abs() / elapsed;
						minutes_to_string(remaining.push(gap / rate))
					};

					then = now;
					current_charge = charge;
					last_status_change += 1;
					percent = current_charge / max;
				}
				Message::Status(status) => {
					if status != current_status {
						last_status_change = 0;
						current_status = status;
						sremain = "...".to_string();
					}
				}
			}
			if current_status == Status::Full {
				sremain = "Full".to_string();
			}
			symbol = get_symbol(current_status, percent);

			block.full_text = Some(create_full_text(&symbol, percent, &sremain));
			s.send((name.clone(), block.to_string())).unwrap();
		});
	}
}

fn get_max_capacity() -> f32 {
	let path = format!("{}/{}", PATH, "charge_full");
	utils::str_to_f32(&fs::read_to_string(&path).unwrap()).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
	Charging,
	Discharging,
	Full,
	Unknown,
}

#[derive(Debug, Clone, Copy)]
enum Message {
	Charge(f32),
	Status(Status),
}

/// Convert a string to a status.
fn str_to_status(s: &str) -> Message {
	match s.trim() {
		"Charging" => Message::Status(Status::Charging),
		"Discharging" => Message::Status(Status::Discharging),
		"Full" => Message::Status(Status::Full),
		"Unknown" => Message::Status(Status::Unknown),
		_ => panic!("Unknown status {}", s),
	}
}

/// Convert a string to a charge enum.
fn str_to_charge(s: &str) -> Message {
	Message::Charge(s.trim().parse().unwrap())
}

/// Continuously monitor `f` for changes, when a change occurs pipe its contents
/// through `content_fn` and send the results over the sender `tx`.
fn looper<F, T>(tx: mpsc::Sender<Message>, mut f: utils::Monitor<T>, content_fn: F)
where
	F: 'static + Fn(&str) -> Message + Send,
	T: 'static + FnMut() -> String + Send,
{
	thread::spawn(move || {
		let mut prev = f.read();
		for contents in f {
			if contents != prev {
				tx.send(content_fn(&contents)).unwrap();
				prev = contents;
			}
		}
	});
}

/// Given a percentage of charge, wrap the string `s` in an appropriate color.
fn wrap_in_color(s: &str, percent: f32) -> String {
	let color = if percent > 0.5 {
		format!("{:0>2x}ff00", 255 - (510.0 * (percent - 0.5)) as i32)
	} else {
		format!("ff{:2x}00", (510.0 * percent) as i32)
	};
	format!("<span foreground='#{}'>{}</span>", color, s)
}

/// Given a percentage of charge, return an appropriate battery symbol.
fn get_discharge_symbol(percent: f32) -> &'static str {
	if percent > 0.90 {
		" "
	} else if percent > 0.60 {
		" "
	} else if percent > 0.40 {
		" "
	} else if percent > 0.10 {
		" "
	} else {
		" "
	}
}

fn get_symbol(status: Status, percent: f32) -> String {
	let s = match status {
		Status::Discharging => get_discharge_symbol(percent),
		_ => " ",
	};
	wrap_in_color(s, percent)
}

/// Convert a float of minutes into a string of hours and minutes.
fn minutes_to_string(remain: f32) -> String {
	let (hrs, mins) = (remain / 60.0, remain % 60.0);
	format!("{:.0}h{:02.0}m", hrs.floor(), mins)
}

/// Start watching the appropriate files for changes and return their current
/// contents.
fn initialise(period: f32, tx: mpsc::Sender<Message>) -> (f32, Status) {
	let mut charge_file = utils::monitor_file(format!("{}/{}", PATH, "charge_now"), period);
	let mut status_file = utils::monitor_file(format!("{}/{}", PATH, "status"), period);

	let current_charge = match str_to_charge(&charge_file.read()) {
		Message::Charge(charge) => charge,
		_ => panic!("Unexpected contents of charge"),
	};

	let current_status = match str_to_status(&status_file.read()) {
		Message::Status(status) => status,
		_ => panic!("Unexpected contents of status"),
	};

	looper(tx.clone(), charge_file, str_to_charge);
	looper(tx.clone(), status_file, str_to_status);

	(current_charge, current_status)
}

fn create_full_text(symbol: &str, percent: f32, remaining: &str) -> String {
	format!("{}{:.0}% ({})", symbol, percent * 100.0, remaining)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn minutes_to_string_works() {
		assert_eq!(minutes_to_string(302.2), "5h02m");
		assert_eq!(minutes_to_string(302.7), "5h03m");
	}
}
