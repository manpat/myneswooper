use toybox::prelude::*;
use crate::ext::*;


pub const BOARD_SIZE: Vec2i = Vec2i::splat(10); 
pub const BOARD_WIDTH: i32 = BOARD_SIZE.x; 


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

	pub fn with_bombs(count: usize) -> Self {
		let mut board = Board::empty();
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let pos = Vec2i::new(
                rng.gen_range(0..board.size().x),
                rng.gen_range(0..board.size().y)
            );

            board.cells.set(pos, Cell::Bomb);
        }

        board.rebuild_adjacency();

		board
	}

	pub fn size(&self) -> Vec2i {
		BOARD_SIZE
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
	pub data: [T; (BOARD_SIZE.x*BOARD_SIZE.y) as usize],
}

impl<T> Map<T> {
	pub fn new_with<F>(value_fn: F) -> Self
		where F: Fn(Vec2i) -> T
	{
		Self {
			data: std::array::from_fn(move |idx| {
				let idx = idx as i32;
				let pos = Vec2i::new(idx % BOARD_WIDTH, idx / BOARD_WIDTH);
				value_fn(pos)
			})
		}
	}

	#[allow(unused)]
	pub fn size(&self) -> Vec2i {
		BOARD_SIZE
	}

	pub fn in_bounds(&self, pos: Vec2i) -> bool {
		Aabb2i::with_size(BOARD_SIZE).contains_point(pos)
	}

	pub fn get_mut(&mut self, pos: Vec2i) -> Option<&mut T> {
		if !self.in_bounds(pos) {
			return None
		}

		let index = pos.x + pos.y * BOARD_WIDTH;
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

		let index = pos.x + pos.y * BOARD_WIDTH;
		Some(&self.data[index as usize])
	}


	pub fn iter_neighbours(&self, pos: Vec2i) -> impl Iterator<Item=&T> + '_ {
		iter_all_neighbour_positions(pos, self.size())
			.filter_map(|pos| self.get(pos))
	}
}

impl<T: Copy> Map<T> {
	pub fn new(value: T) -> Self {
		Self {
			data: [value; (BOARD_SIZE.x*BOARD_SIZE.y) as usize]
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

