use std::fs::File;
use std::process::{self, Command};
use std::io::{self, Read, Seek, SeekFrom};
use std::thread;
use std::time::Duration;

// Seek to the beginning of a file and read all its contents into a string.
fn read_to_string(f: &mut File, mut buf: &mut String) -> io::Result<()> {
	f.seek(SeekFrom::Start(0))?;
	f.read_to_string(&mut buf)?;
	Ok(())
}

pub fn str_to_f32(s: &str) -> f32 {
	s.trim().parse().unwrap()
}

/// `MonitorFile` is a struct which can be created to periodically read the
/// contents of a file when iterated.
pub struct MonitorFile {
	file: File,
	period: Duration,
	buf: String,
	first: bool,
}

impl MonitorFile {
	/// Create a new `MonitorFile` instance.
	pub fn new(path: &str, period: u64) -> MonitorFile {
		let file = File::open(path).unwrap();
		MonitorFile {
			file,
			period: Duration::from_millis(period),
			buf: String::new(),
			first: true,
		}
	}

	/// Read contents of file immediately.
	pub fn read(&mut self) -> String {
		self.buf.truncate(0);
		self._read();
		self.buf.clone()
	}

	fn _read(&mut self) {
		read_to_string(&mut self.file, &mut self.buf).unwrap();
	}
}

impl Iterator for MonitorFile {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		if self.first {
			self.first = false;
		} else {
			thread::sleep(self.period);
		}
		Some(self.read())
	}
}

// pub struct MonitorCommand {
// 	cmd: String,
// 	period: Duration,
// 	buf: String,
// 	first: bool,
// }
// 
// impl Reader for MonitorCommand {
// 	fn _read(&mut self) -> String {
// 		if let Ok(output) = Command::new(&self.cmd).output() {
// 			output.stdout
// 		} else {
// 			format!("Command failed: '{}'", &self.cmd)
// 		}
// 	}
// }
