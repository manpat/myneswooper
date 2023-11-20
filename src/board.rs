use toybox::prelude::*;
use crate::ext::*;


pub const BOARD_SIZE: usize = 3; 

#[derive(Debug)]
pub struct Board {
	pub cells: Map<Cell>,
}

impl Board {
	pub fn empty() -> Self {
		Self {
			cells: Map::new(Cell::Empty),
		}
	}

	pub const fn size(&self) -> Vec2i {
		Vec2i::splat(BOARD_SIZE as i32)
	}

	pub fn rebuild_adjacency(&mut self) {
		for pos in vec2i_range(self.size()) {
			let Some(cell) = self.cells.get(pos) else { continue };

			if cell == Cell::Bomb {
				continue
			}


			let neighbouring_bombs = self.cells.iter_neighbours(pos)
				.filter(|&cell| cell == Cell::Bomb)
				.count();

			let new_cell = match neighbouring_bombs {
				0 => Cell::Empty,
				n => Cell::BombAdjacent(n),
			};

			self.cells.set(pos, new_cell);
		}
	}
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Cell {
	Empty,
	Bomb,
	BombAdjacent(usize),
}





#[derive(Debug)]
pub struct Map<T: Copy> {
	pub data: [T; BOARD_SIZE*BOARD_SIZE],
}

impl<T: Copy> Map<T> {
	pub fn new(value: T) -> Self {
		Self {
			data: [value; BOARD_SIZE*BOARD_SIZE]
		}
	}

	pub const fn size(&self) -> Vec2i {
		Vec2i::splat(BOARD_SIZE as i32)
	}

	pub fn in_bounds(&self, pos: Vec2i) -> bool {
		pos.x >= 0
		&& pos.y >= 0
		&& pos.x < BOARD_SIZE as i32
		&& pos.y < BOARD_SIZE as i32
	}

	pub fn get(&self, pos: Vec2i) -> Option<T> {
		if !self.in_bounds(pos) {
			return None
		}

		let index = pos.x + pos.y * BOARD_SIZE as i32;
		Some(self.data[index as usize])
	}

	pub fn get_mut(&mut self, pos: Vec2i) -> Option<&mut T> {
		if !self.in_bounds(pos) {
			return None
		}

		let index = pos.x + pos.y * BOARD_SIZE as i32;
		Some(&mut self.data[index as usize])
	}

	pub fn set(&mut self, pos: Vec2i, value: T) {
		if let Some(cell) = self.get_mut(pos) {
			*cell = value;
		}
	}

	pub fn iter_neighbours(&self, pos: Vec2i) -> impl Iterator<Item=T> + '_ {
		iter_neighbour_positions(pos)
			.filter_map(|pos| self.get(pos))
	}
}

pub fn iter_neighbour_positions(pos: Vec2i) -> impl Iterator<Item=Vec2i> {
	let deltas = [
		Vec2i::new(-1, -1),
		Vec2i::new( 0, -1),
		Vec2i::new( 1, -1),

		Vec2i::new(-1,  0),
		Vec2i::new( 1,  0),

		Vec2i::new(-1,  1),
		Vec2i::new( 0,  1),
		Vec2i::new( 1,  1),
	];

	deltas.into_iter().map(move |delta| pos + delta)
}