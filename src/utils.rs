// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use std::fs::{self, File};
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

/// Parse a string as a float32.
pub fn file_to_f32(path: &str) -> Result<f32, num::ParseFloatError> {
	fs::read_to_string(path).unwrap().trim().parse()
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
	fn new(reader: T, period: f32) -> Self {
		Monitor {
			reader,
			period: Duration::from_secs_f32(period),
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
		Some(self.read())
	}
}

/// Monitor a file at a given path. When iterated it's contents are periodically
/// read. NB the file should already have been verified to exist at this point,
/// so we should safe to expect that it still does.
pub fn monitor_file(path: String, period: f32) -> Monitor<impl FnMut() -> String> {
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
	period: f32,
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

/// Wait for a signal to occur with a given timeout. Sends a message through
/// the returned receiver when the signal/timeout occurs.
pub fn wait_for_signal(signal: i32, timeout: f32) -> crossbeam_channel::Receiver<()> {
	let (s, r) = crossbeam_channel::unbounded();
	let s2 = s.clone();
	unsafe {
		signal_hook::register(signal, move || s2.send(()).unwrap()).unwrap();
	}
	thread::spawn(move || loop {
		thread::sleep(Duration::from_secs_f32(timeout));
		s.send(()).unwrap();
	});
	r
}
