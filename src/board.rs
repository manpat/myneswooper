use toybox::prelude::*;
use crate::ext::*;



#[derive(Debug)]
pub struct Board {
	pub cells: Map<Cell>,
}

impl Board {
	pub fn empty(size: Vec2i) -> Self {
		Self {
			cells: Map::new(size, Cell::Empty),
		}
	}

	pub fn with_bombs(size: Vec2i, count: usize) -> Self {
		let mut board = Board::empty(size);
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let pos = Vec2i::new(
                rng.gen_range(0..size.x),
                rng.gen_range(0..size.y)
            );

            board.cells.set(pos, Cell::Bomb);
        }

        board.rebuild_adjacency();

		board
	}

	pub fn size(&self) -> Vec2i {
		self.cells.size()
	}

	pub fn rebuild_adjacency(&mut self) {
		for pos in vec2i_range(self.size()) {
			let Some(&cell) = self.cells.get(pos) else { continue };

			if cell == Cell::Bomb {
				continue
			}

			let neighbouring_bombs = self.cells.iter_neighbours(pos)
				.filter(|&&cell| cell == Cell::Bomb)
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

	pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> + '_ {
		self.data.iter_mut()
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

pub fn iter_neighbour_positions<const N: usize>(pos: Vec2i, size: Vec2i, deltas: [Vec2i; N]) -> impl Iterator<Item=Vec2i> {
	deltas.into_iter()
		.map(move |delta| pos + delta)
		.filter(move |&pos| Aabb2i::with_size(size).contains_point(pos))
}

pub fn iter_all_neighbour_positions(pos: Vec2i, size: Vec2i) -> impl Iterator<Item=Vec2i> {
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

	iter_neighbour_positions(pos, size, deltas)
}

pub fn iter_ortho_neighbour_positions(pos: Vec2i, size: Vec2i) -> impl Iterator<Item=Vec2i> {
	let deltas = [
		Vec2i::new( 0, -1),
		Vec2i::new( 0,  1),
		Vec2i::new(-1,  0),
		Vec2i::new( 1,  0),
	];

	iter_neighbour_positions(pos, size, deltas)
}

