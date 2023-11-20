use toybox::prelude::*;




pub trait Aabb2Ext {
	fn expand(&self, amount: Vec2) -> Aabb2;
	fn shrink(&self, amount: Vec2) -> Aabb2 {
		self.expand(-amount)
	}

	fn scale_about_center(&self, amount: Vec2) -> Aabb2;
	fn aspect(&self) -> f32 {
		let size = self.size();
		size.x / size.y
	}

	fn size(&self) -> Vec2;
	fn center(&self) -> Vec2;

	fn split_once_vertical(&self, percent: f32) -> (Aabb2, Aabb2);
	fn split(&self, times: Vec2i) -> impl Iterator<Item=Aabb2>;
	fn section(&self, subdivisions: Vec2i, which: Vec2i) -> Aabb2;
}

impl Aabb2Ext for Aabb2 {
	fn expand(&self, amount: Vec2) -> Self {
		Aabb2 {
			min: self.min - amount,
			max: self.max + amount,
		}
	}

	fn scale_about_center(&self, amount: Vec2) -> Self {
		let extents = self.size() * amount / 2.0;
		let center = self.center();
		Aabb2::around_point(center, extents)
	}

	fn size(&self) -> Vec2 {
		if self.is_empty() {
			Vec2::zero()
		} else {
			self.max - self.min
		}
	}

	fn center(&self) -> Vec2 {
		(self.min + self.max) / 2.0
	}

	fn split_once_vertical(&self, percent: f32) -> (Aabb2, Aabb2) {
		let split_y = self.min.y + percent * (self.max.y - self.min.y);

		let Aabb2{min, max} = *self;

		let min_half = Aabb2::new(min, Vec2{y: split_y, ..max});
		let max_half = Aabb2::new(Vec2{y: split_y, ..min}, max);

		(min_half, max_half)
	}

	fn split(&self, times: Vec2i) -> impl Iterator<Item=Aabb2> {
		assert!(!self.is_empty());
		assert!(times.x > 0);
		assert!(times.y > 0);

		let size = self.size();
		let cell_size = size / times.to_vec2();
		let start = self.min;

		vec2i_range(times)
			.map(move |pos| {
				let pos = start + pos.to_vec2() * cell_size;
				Aabb2::new(pos, pos + cell_size)
			})
	}

	fn section(&self, subdivisions: Vec2i, which: Vec2i) -> Aabb2 {
		let cell_size = self.size() / subdivisions.to_vec2();
		let start = self.min + cell_size * which.to_vec2();
		Aabb2::new(start, start + cell_size)
	}
}


pub fn vec2i_range(Vec2i{x, y}: Vec2i) -> impl Iterator<Item=Vec2i> {
	(0..y)
		.flat_map(move |j| {
			(0..x).map(move |i| Vec2i::new(i, j))
		})
}





pub struct Aabb2i {
	pub min: Vec2i, // inclusive
	pub max: Vec2i, // exclusive
}

impl Aabb2i {
	pub fn with_size(size: Vec2i) -> Aabb2i {
		Aabb2i {
			min: Vec2i::zero(),
			max: size,
		}
	}

	pub fn contains_point(&self, point: Vec2i) -> bool {
		point.x >= self.min.x && point.y >= self.min.y
		&& point.x < self.max.x && point.y < self.max.y
	}
}