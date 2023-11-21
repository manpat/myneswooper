use toybox::prelude::*;

use crate::ext::*;
use crate::board::*;
use crate::sound::*;
use crate::quad_builder::QuadBuilder;

mod cell_view;
use cell_view::*;


pub struct BoardView {
	main_vs: gfx::ShaderHandle,
	main_fs: gfx::ShaderHandle,
	atlas: gfx::ImageHandle,
	sampler: gfx::SamplerName,

	pub bounds: Aabb2,
	cells: Map<CellView>,
}


impl BoardView {
	pub fn new(ctx: &mut toybox::Context, board: &Board) -> anyhow::Result<BoardView> {
		let core = &mut ctx.gfx.core;
		let rm = &mut ctx.gfx.resource_manager;

		let bounds = Self::make_bounds(board);

		Ok(BoardView {
			main_vs: rm.request(gfx::LoadShaderRequest::from("shaders/main.vs.glsl")?),
			main_fs: rm.request(gfx::LoadShaderRequest::from("shaders/main.fs.glsl")?),
			atlas: rm.request(gfx::LoadImageRequest::from("images/atlas.png")),

			sampler: {
				let sampler = core.create_sampler();
				core.set_sampler_addressing_mode(sampler, gfx::AddressingMode::Clamp);
				core.set_sampler_minify_filter(sampler, gfx::FilterMode::Nearest, None);
				core.set_sampler_magnify_filter(sampler, gfx::FilterMode::Nearest);
				sampler
			},

			bounds,
			cells: Self::make_cells(board, bounds),
		})
	}

	pub fn reset(&mut self, board: &Board) {
		self.bounds = Self::make_bounds(board);
		self.cells = Self::make_cells(board, self.bounds);
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, sound: &SoundSystem, board: &mut Board, safe_zone: f32) {
		let mut any_response = None;

		for (cell_view, cell) in self.cells.iter_mut().zip(board.cells.iter()) {
			if let Some(response) = cell_view.update(ctx, cell, safe_zone) {
				any_response = Some((response, cell_view.position));
			}
		}

		if let Some((response, position)) = any_response {
			self.handle_response(response, position, board, sound);
		}

		self.draw(&mut ctx.gfx, board);
	}

	fn make_bounds(board: &Board) -> Aabb2 {
		let Vec2{x, y} = board.size().to_vec2();
		let aspect = x/y;

		let extent = Vec2::new(aspect, 1.0);

		Aabb2::new(-extent, extent)
			.shrink(Vec2::splat(0.05))
	}

	fn make_cells(board: &Board, bounds: Aabb2) -> Map<CellView> {
		Map::new_with(board.size(), |pos| {
			let cell = board.cells.get(pos).unwrap();
			let bounds = bounds.section(board.size(), pos).scale_about_center(Vec2::splat(0.95));
			CellView::from(&cell, bounds, pos)
		})
	}


	fn handle_response(&mut self, response: CellResponse, cell_position: Vec2i, board: &mut Board, sound: &SoundSystem) {
		match response {
			CellResponse::BombHit => {
				let is_first_cell = self.cells.iter()
					.filter(|view| view.state == CellState::Opened)
					.count() == 1;

				// First click is always safe
				if is_first_cell {
					sound.play(Sound::Plik);
					self.move_bomb(cell_position, board);
					return;
				}

				self.uncover_all();
				sound.play(Sound::Bong);
				println!("LOSE!")
			}

			CellResponse::FlagPlaced => {
				if self.are_all_bombs_marked(board) {
					self.uncover_all();
					sound.play(Sound::Tada);
					println!("WIN!");
				} else {
					sound.play(Sound::Thup);
				}
			}

			CellResponse::FlagRemoved => {
				sound.play(Sound::Unthup);
			}

			CellResponse::OpenSpaceUncovered => {
				sound.play(Sound::Plik);
				self.flood_uncover_empty(cell_position, board);
			}

			CellResponse::UnsafeSpaceUncovered => {
				sound.play(Sound::Plik);
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

		let mut visit_queue = vec![start];

		while let Some(position) = visit_queue.pop() {
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

				if starting_from_blank && cell == Cell::Empty {
					visit_queue.push(neighbour_position);
				}
			}
		}
	}

	fn move_bomb(&mut self, position: Vec2i, board: &mut Board) {
		println!("Moving bomb from {position:?}");

		board.cells.set(position, Cell::Empty);

        let mut rng = rand::thread_rng();

        for _ in 0..16 {
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

		builder.add(self.bounds, Color::grey(0.2), 0);

		for (cell_view, cell) in self.cells.iter().zip(board.cells.iter()) {
			cell_view.draw(&mut builder, cell);
		}

		builder.finish();

		let mut group = gfx.frame_encoder.command_group("main");

		group.draw(self.main_vs, self.main_fs)
			.elements(builder.indices.len() as u32)
			.indexed(&builder.indices)
			.ssbo(0, &builder.vertices)
			.sampled_image(0, self.atlas, self.sampler)
			.depth_test(false)
			.blend_mode(gfx::BlendMode::ALPHA);
	}
}






