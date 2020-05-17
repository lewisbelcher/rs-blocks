use rs_blocks::{self, file};

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

fn main() {
	let rx_file = file::MonitorFile::new(&get_network_file("rx"), PERIOD);
	let mut tx_file = file::MonitorFile::new(&get_network_file("tx"), PERIOD);

	let coef = 1.0 / (PERIOD as f32 * 1.024); // Report in kB
	let mut rx = Speed::new(coef);
	let mut tx = Speed::new(coef);
	let mut first = true;

	for rx_ in rx_file {
		rx.push(rs_blocks::str_to_f32(&rx_));
		tx.push(rs_blocks::str_to_f32(&tx_file.read()));

		if first {
			first = false;
		} else {
			print!(
				"<span foreground='#ccffcc'>\u{f0ab} {:.1}</span> ",
				rx.calc_speed()
			);
			println!(
				"<span foreground='#ffcccc'>\u{f0aa} {:.1}</span>",
				tx.calc_speed()
			);
		}
	}
}
