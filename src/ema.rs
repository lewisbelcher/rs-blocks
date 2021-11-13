// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use std::ops;

pub struct Ema<T> {
	current: Option<T>,
	alpha: T,
}

impl<T> Ema<T>
where
	T: From<u8> + Copy + ops::Mul<Output = T> + ops::Add<Output = T> + ops::Sub<Output = T>,
{
	pub fn new(alpha: T) -> Ema<T> {
		Ema {
			current: None,
			alpha,
		}
	}

	pub fn push(&mut self, new: T) -> T {
		if let Some(mut current) = self.current {
			current = self.alpha * current + (T::from(1) - self.alpha) * new;
			self.current = Some(current);
			current
		} else {
			self.current = Some(new);
			new
		}
	}

	pub fn reset(&mut self) {
		self.current = None;
	}
}
