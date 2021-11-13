// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rs_blocks::file::MonitorFile;

fn read_file(path: &Path) -> Result<String, ()> {
	let mut f = OpenOptions::new().read(true).open(path).unwrap();
	let mut content = String::new();
	f.read_to_string(&mut content).unwrap();
	content.pop();
	Ok(content)
}

fn bench_file_readers(c: &mut Criterion) {
	let path = "/sys/class/net/wlp2s0/statistics/rx_bytes";

	let device_path = Path::new(path);
	c.bench_function("reopen file", |b| b.iter(|| read_file(black_box(device_path))));

	let mut f = MonitorFile::new(path, 4);
	c.bench_function("buffered file", |b| b.iter(|| f.read()));
}

criterion_group!(benches, bench_file_readers);
criterion_main!(benches);
