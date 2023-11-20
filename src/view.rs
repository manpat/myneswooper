use toybox::prelude::*;

use crate::ext::*;
use crate::board::*;
use crate::quad_builder::QuadBuilder;

mod cell_view;
use cell_view::*;


pub struct BoardView {
	main_vs: gfx::ShaderHandle,
	main_fs: gfx::ShaderHandle,

	bounds: Aabb2,
	cells: Map<CellView>,
}


impl BoardView {
	pub fn new(ctx: &mut toybox::Context, board: &Board) -> anyhow::Result<BoardView> {
		let rm = &mut ctx.gfx.resource_manager;

		let bounds = Aabb2::new(Vec2::splat(-1.0), Vec2::splat(1.0)) .expand(Vec2::splat(-0.05));

		let cells = Map::new_with(|pos| {
			let cell = board.cells.get(pos).unwrap();
			let bounds = bounds.section(board.size(), pos);
			CellView::from(&cell, bounds, pos)
		});

		Ok(BoardView {
			main_vs: rm.request(gfx::LoadShaderRequest::from("shaders/main.vs.glsl")?),
			main_fs: rm.request(gfx::LoadShaderRequest::from("shaders/main.fs.glsl")?),
			bounds,
			cells,
		})
	}

	pub fn reset(&mut self, board: &Board) {
		self.cells = Map::new_with(|pos| {
			let cell = board.cells.get(pos).unwrap();
			let bounds = self.bounds.section(board.size(), pos);
			CellView::from(&cell, bounds, pos)
		});
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, board: &mut Board) {
		let mut any_response = None;

		for (cell_view, cell) in self.cells.iter_mut().zip(board.cells.iter()) {
			if let Some(response) = cell_view.update(ctx, cell) {
				any_response = Some((response, cell_view.position));
			}
		}

		if let Some((response, position)) = any_response {
			self.handle_response(response, position, board);
		}

		self.draw(&mut ctx.gfx, board);
	}


	fn handle_response(&mut self, response: CellResponse, cell_position: Vec2i, board: &mut Board) {
		match response {
			CellResponse::BombHit => {
				let is_first_cell = self.cells.iter()
					.filter(|view| view.state == CellState::Opened)
					.count() == 1;

				// First click is always safe
				if is_first_cell {
					self.move_bomb(cell_position, board);
					return;
				}

				self.uncover_all();
				println!("LOSE!")
			}

			CellResponse::FlagPlaced => {
				if self.are_all_bombs_marked(board) {
					self.uncover_all();
					println!("WIN!");
				}
			}

			CellResponse::OpenSpaceUncovered => {
				self.flood_uncover_empty(cell_position, board);
			}
		}
	}

	fn uncover_all(&mut self) {
		for cell_view in self.cells.iter_mut() {
			if cell_view.state != CellState::Flagged {
				cell_view.state = CellState::Opened;
			}
		}
	}

	fn are_all_bombs_marked(&self, board: &Board) -> bool {
		self.cells.iter().map(|v| v.state)
			.zip(board.cells.iter().cloned())
			.all(|state_cell| match state_cell {
				(CellState::Flagged, cell) => cell == Cell::Bomb,
				(state, Cell::Bomb) => state == CellState::Flagged,
				_ => true,
			})
	}

	fn flood_uncover_empty(&mut self, start: Vec2i, board: &Board) {
		let start_cell = *board.cells.get(start).unwrap();
		let starting_from_blank = start_cell == Cell::Empty;

		let mut blank_queue = Vec::new();
		let mut adjacent_queue = Vec::new();

		if starting_from_blank {
			blank_queue.push(start);
		} else {
			adjacent_queue.push(start);
		}

		while let Some(position) = blank_queue.pop() {
			println!("dequeued blank {position:?}");

			for neighbour_position in iter_ortho_neighbour_positions(position, board.size()) {
				let cell = *board.cells.get(neighbour_position).unwrap();
				if cell == Cell::Bomb {
					continue
				}

				let state = &mut self.cells.get_mut(neighbour_position).unwrap().state;
				if *state != CellState::Unopened {
					continue;
				}

				*state = CellState::Opened;

				if cell == Cell::Empty {
					blank_queue.push(neighbour_position);
				}
			}
		}

		while let Some(position) = adjacent_queue.pop() {
			println!("dequeued adjacent {position:?}");

			for neighbour_position in iter_ortho_neighbour_positions(position, board.size()) {
				let cell = *board.cells.get(neighbour_position).unwrap();
				if cell == Cell::Bomb {
					continue
				}

				let state = &mut self.cells.get_mut(neighbour_position).unwrap().state;
				*state = CellState::Opened;
			}
		}
	}

	fn move_bomb(&mut self, position: Vec2i, board: &mut Board) {
		println!("Moving bomb from {position:?}");

		board.cells.set(position, Cell::Empty);

        let mut rng = rand::thread_rng();

        for _ in 0..8 {
            let new_position = Vec2i::new(
                rng.gen_range(0..board.size().x),
                rng.gen_range(0..board.size().y)
            );

            if new_position == position {
            	continue
            }

            let cell = board.cells.get_mut(new_position).unwrap();
            if *cell == Cell::Empty {
            	*cell = Cell::Bomb;
            	break
            }
        }

		board.rebuild_adjacency();

		// If the newly empty cell has no adjacent bombs, flood fill as normal
		let new_cell = board.cells.get(position).unwrap();
		if new_cell == &Cell::Empty {
			self.flood_uncover_empty(position, board);
		}
	}

	fn draw(&self, gfx: &mut gfx::System, board: &Board) {
		let mut builder = QuadBuilder::default();

		builder.add(self.bounds, Color::grey(0.2));

		for (cell_view, cell) in self.cells.iter().zip(board.cells.iter()) {
			cell_view.draw(&mut builder, cell);
		}

		builder.finish();

		let mut group = gfx.frame_encoder.command_group("main");

		group.draw(self.main_vs, self.main_fs)
			.elements(builder.indices.len() as u32)
			.indexed(&builder.indices)
			.ssbo(0, &builder.vertices)
			.depth_test(false);
	}
}






