use toybox::prelude::*;
use crate::ext::*;
use crate::map::*;


#[derive(Debug)]
pub struct Board {
	pub types: Map<CellType>,
	pub states: Map<CellState>,
}

impl Board {
	pub fn empty(size: Vec2i) -> Self {
		Self {
			types: Map::new(size, CellType::Empty),
			states: Map::new(size, CellState::Unopened),
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

            board.types.set(pos, CellType::Bomb);
        }

        board.rebuild_adjacency();

		board
	}

	pub fn size(&self) -> Vec2i {
		self.types.size()
	}

	fn rebuild_adjacency(&mut self) {
		for pos in vec2i_range(self.size()) {
			let Some(&cell) = self.types.get(pos) else { continue };

			if cell == CellType::Bomb {
				continue
			}

			let neighbouring_bombs = self.types.iter_neighbours(pos)
				.filter(|&&cell| cell == CellType::Bomb)
				.count();

			let new_cell = match neighbouring_bombs {
				0 => CellType::Empty,
				n => CellType::BombAdjacent(n),
			};

			self.types.set(pos, new_cell);
		}
	}
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CellType {
	Empty,
	Bomb,
	BombAdjacent(usize),
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CellState {
	Unopened,
	Flagged,
	Opened,
}




impl Board {
	pub fn are_all_bombs_flagged(&self) -> bool {
		self.states.iter().cloned()
			.zip(self.types.iter().cloned())
			.all(|state_cell| match state_cell {
				(CellState::Flagged, cell) => cell == CellType::Bomb,
				(state, CellType::Bomb) => state == CellState::Flagged,
				_ => true,
			})
	}

	pub fn uncover_all(&mut self) {
		for state in self.states.iter_mut() {
			if *state != CellState::Flagged {
				*state = CellState::Opened;
			}
		}
	}

	pub fn flood_uncover_empty(&mut self, start: Vec2i) {
		let start_cell = *self.types.get(start).unwrap();
		let starting_from_blank = start_cell == CellType::Empty;

		let mut visit_queue = vec![start];

		while let Some(position) = visit_queue.pop() {
			for neighbour_position in iter_ortho_neighbour_positions(position, self.size()) {
				let cell = *self.types.get(neighbour_position).unwrap();
				if cell == CellType::Bomb {
					continue
				}

				let state = self.states.get_mut(neighbour_position).unwrap();
				if *state != CellState::Unopened {
					continue;
				}

				*state = CellState::Opened;

				if starting_from_blank && cell == CellType::Empty {
					visit_queue.push(neighbour_position);
				}
			}
		}
	}

	pub fn move_bomb(&mut self, position: Vec2i) {
		println!("Moving bomb from {position:?}");

		self.types.set(position, CellType::Empty);

        let mut rng = rand::thread_rng();
        let Vec2i{x: width, y: height} = self.size();

        // Retry naively until it works
        for _ in 0..32 {
            let new_position = Vec2i::new(
                rng.gen_range(0..width),
                rng.gen_range(0..height)
            );

            if new_position == position {
            	continue
            }

            let cell = self.types.get_mut(new_position).unwrap();
            if *cell == CellType::Empty {
            	*cell = CellType::Bomb;
            	break
            }
        }

		self.rebuild_adjacency();

		// If the newly empty cell has no adjacent bombs, flood fill as normal
		let new_cell = self.types.get(position).unwrap();
		if new_cell == &CellType::Empty {
			self.flood_uncover_empty(position);
		}
	}
}

