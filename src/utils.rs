use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::num;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Seek to the beginning of a file and read all its contents into a string.
fn read_to_string(f: &mut File, mut buf: &mut String) -> io::Result<()> {
	f.seek(SeekFrom::Start(0))?;
	f.read_to_string(&mut buf)?;
	Ok(())
}

/// Parse a string as a float32.
pub fn str_to_f32(s: &str) -> Result<f32, num::ParseFloatError> {
	s.trim().parse()
}

/// A monitoring abstraction which will periodically call `reader` when iterated.
pub struct Monitor<T>
where
	T: FnMut() -> String,
{
	reader: T,
	period: Duration,
	first: bool,
}

impl<T> Monitor<T>
where
	T: FnMut() -> String,
{
	fn new(reader: T, period: u64) -> Self {
		Monitor {
			reader,
			period: Duration::from_millis(period),
			first: true,
		}
	}

	pub fn read(&mut self) -> String {
		(self.reader)()
	}
}

impl<T> Iterator for Monitor<T>
where
	T: FnMut() -> String,
{
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		if self.first {
			self.first = false;
		} else {
			thread::sleep(self.period);
		}
		Some((self.reader)())
	}
}

/// Monitor a file at a given path. When iterated it's contents are periodically
/// read.
pub fn monitor_file(path: String, period: u64) -> Monitor<impl FnMut() -> String> {
	let mut file = File::open(&path).unwrap();
	let mut buf = String::new();
	Monitor::new(
		move || {
			buf.truncate(0);
			if let Ok(_) = read_to_string(&mut file, &mut buf) {
				buf.clone()
			} else {
				format!("Failed to read: {}", &path)
			}
		},
		period,
	)
}

/// Monitor a given command. When iterated it is periodically executed and its
/// stdout is returned.
pub fn monitor_command(
	cmd: &'static str,
	args: &'static [&'static str],
	period: u64,
) -> Monitor<impl FnMut() -> String> {
	Monitor::new(
		move || {
			if let Ok(output) = Command::new(cmd).args(args).output() {
				String::from_utf8(output.stdout).unwrap()
			} else {
				format!("Command failed: '{:?}'", &cmd)
			}
		},
		period,
	)
}

pub fn wait_for_signal(signal: i32, timeout: u64) -> crossbeam_channel::Receiver<()> {
	let (s, r) = crossbeam_channel::unbounded();
	let s2 = s.clone();
	unsafe {
		signal_hook::register(signal, move || s2.send(()).unwrap()).unwrap();
	}
	thread::spawn(move || {
		thread::sleep(Duration::from_millis(timeout));
		s.send(()).unwrap();
	});
	r
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
		read_to_string(&mut self.file, &mut self.buf).unwrap();
		self.buf.clone()
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
