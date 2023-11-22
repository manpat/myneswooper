use toybox::prelude::*;
use crate::ext::*;


#[derive(Debug)]
pub struct Map<T> {
	pub data: Vec<T>,
	size: Vec2i,
}

impl<T> Map<T> {
	pub fn new_with<F>(size: Vec2i, value_fn: F) -> Self
		where F: Fn(Vec2i) -> T
	{
		Self {
			data: (0..size.x*size.y).map(move |idx| {
				let idx = idx as i32;
				let pos = Vec2i::new(idx % size.x, idx / size.x);
				value_fn(pos)
			})
			.collect(),

			size
		}
	}

	pub fn size(&self) -> Vec2i {
		self.size
	}

	pub fn in_bounds(&self, pos: Vec2i) -> bool {
		Aabb2i::with_size(self.size).contains_point(pos)
	}

	pub fn get_mut(&mut self, pos: Vec2i) -> Option<&mut T> {
		if !self.in_bounds(pos) {
			return None
		}

		let index = pos.x + pos.y * self.size.x;
		Some(&mut self.data[index as usize])
	}

	pub fn set(&mut self, pos: Vec2i, value: T) {
		if let Some(cell) = self.get_mut(pos) {
			*cell = value;
		}
	}

	pub fn iter(&self) -> impl Iterator<Item=&T> + '_ {
		self.data.iter()
	}

	pub fn iter_with_positions(&self) -> impl Iterator<Item=(Vec2i, &T)> + '_ {
		std::iter::zip(vec2i_range(self.size), &self.data)
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> + '_ {
		self.data.iter_mut()
	}

	pub fn iter_mut_with_positions(&mut self) -> impl Iterator<Item=(Vec2i, &mut T)> + '_ {
		std::iter::zip(vec2i_range(self.size), &mut self.data)
	}

	pub fn get(&self, pos: Vec2i) -> Option<&T> {
		if !self.in_bounds(pos) {
			return None
		}

		let index = pos.x + pos.y * self.size.x;
		Some(&self.data[index as usize])
	}


	pub fn iter_neighbours(&self, pos: Vec2i) -> impl Iterator<Item=&T> + '_ {
		iter_all_neighbour_positions(pos, self.size)
			.filter_map(|pos| self.get(pos))
	}
}

impl<T: Copy> Map<T> {
	pub fn new(size: Vec2i, value: T) -> Self {
		Self {
			data: vec![value; (size.x*size.y) as usize],
			size,
		}
	}
}