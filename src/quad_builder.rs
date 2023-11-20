use toybox::prelude::*;



#[derive(Default)]
pub struct QuadBuilder {
	pub vertices: Vec<QuadVert>,
	pub indices: Vec<u32>,
}

impl QuadBuilder {
	pub fn add(&mut self, Aabb2{min, max}: Aabb2, color: impl Into<Color>) {
		let color = color.into().to_byte_array();

		self.vertices.push(QuadVert{ pos: Vec2::new(min.x, min.y), color, _pad: 0 });
		self.vertices.push(QuadVert{ pos: Vec2::new(min.x, max.y), color, _pad: 0 });
		self.vertices.push(QuadVert{ pos: Vec2::new(max.x, max.y), color, _pad: 0 });
		self.vertices.push(QuadVert{ pos: Vec2::new(max.x, min.y), color, _pad: 0 });
	}

	pub fn finish(&mut self) {
		let num_quads = self.vertices.len() / 4;

		let indices = (0..num_quads as u32)
			.flat_map(|idx| [0u32, 1, 2, 0, 2, 3].into_iter().map(move |base| base + idx*4));

		self.indices.extend(indices)
	}
}




#[repr(C)]
#[derive(Copy, Clone)]
pub struct QuadVert {
	pos: Vec2,
	color: [u8; 4],
	_pad: u32,
}