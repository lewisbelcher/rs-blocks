// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rs_blocks::utils;

fn read_file(path: &str) {
	std::fs::read_to_string(path).unwrap();
}

fn bench_file_readers(c: &mut Criterion) {
	let path = "/sys/class/net/wlan0/statistics/rx_bytes";

	c.bench_function("reopen file", |b| b.iter(|| read_file(black_box(path))));

	let mut f = utils::monitor_file(path.to_string(), 4.0);
	c.bench_function("buffered file", |b| b.iter(|| f.read()));
}

criterion_group!(benches, bench_file_readers);
criterion_main!(benches);
