use crate::blocks::Block;
use crate::{ema, utils};
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const PATH: &str = "/sys/class/power_supply/BAT0";
const ALPHA: f32 = 0.8; // Exponential moving average coefficient
const MINALPHA: f32 = 0.2; // Minimum exponential moving average coefficient
const PERIOD: u64 = 500; // Monitor interval in ms

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

fn str_to_status(s: &str) -> Message {
	match s.trim() {
		"Charging" => Message::Status(Status::Charging),
		"Discharging" => Message::Status(Status::Discharging),
		"Full" => Message::Status(Status::Full),
		"Unknown" => Message::Status(Status::Unknown),
		_ => panic!("Unknown status {}", s),
	}
}

fn str_to_charge(s: &str) -> Message {
	Message::Charge(s.trim().parse().unwrap())
}

// TODO: Update this to be compatible with `Monitor`.
fn looper<F: 'static>(tx: mpsc::Sender<Message>, mut f: utils::MonitorFile, content_fn: F)
where
	F: Fn(&str) -> Message + Send,
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
fn get_discharge_symbol(percent: f32) -> String {
	let n = ((1.0 - percent) / 0.2).floor() as u8;
	String::from_utf8(vec![239, 137, 128 + n]).unwrap()
}

fn get_symbol(status: Status, percent: f32) -> String {
	let s = match status {
		Status::Discharging => get_discharge_symbol(percent),
		_ => " ".to_string(),
	};
	wrap_in_color(&s, percent)
}

/// Convert a float of minutes into a string of hours and minutes.
fn minutes_to_string(remain: f32) -> String {
	let (hrs, mins) = (remain / 60.0, remain % 60.0);
	format!("{:.0}h{:02.0}m", hrs.floor(), mins)
}

// Calculate the time remaining to fill `gap`, given `rate`. We use a rate
// adjusted exponential moving average, based on the time since the last
// change in status.
// fn calc_remain(gap:f32, prev:f32, rate:f32, lastStatusChange: u64) ->f32 {
// 	_alpha := math.Max(math.Pow(ALPHA, Float64(lastStatusChange)), MINALPHA);
// 	return _alpha*gap/rate + (1-_alpha)*prev
// }

/// Calculate the time remaining to fill `gap`, given `rate`.
fn calc_remain(gap: f32, prev: f32, rate: f32) -> f32 {
	return ALPHA * gap / rate + (1.0 - ALPHA) * prev;
}

fn initialise(tx: mpsc::Sender<Message>) -> (f32, Status) {
	let mut charge_file = utils::MonitorFile::new(&format!("{}/{}", PATH, "charge_now"), PERIOD);
	let mut status_file = utils::MonitorFile::new(&format!("{}/{}", PATH, "status"), PERIOD);

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

pub fn add_sender(
	name: &'static str,
	s: crossbeam_channel::Sender<(&'static str, String)>,
) -> &'static str {
	let max = get_max_capacity();
	let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
	let mut sremain = "...".to_string();
	let mut last_status_change = 0;
	let mut remaining = ema::Ema::new(ALPHA);
	let (mut current_charge, mut current_status) = initialise(tx);
	let mut then = Instant::now();
	let mut percent = current_charge / max;
	let mut block = Block::new(name, true);

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
		let symbol = get_symbol(current_status, percent);

		block.full_text = Some(format!("{}{:.0}% ({})", symbol, percent * 100.0, sremain));
		s.send((name, block.to_string())).unwrap();
	});
	name
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
