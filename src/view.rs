use toybox::prelude::*;

use crate::board::*;
use crate::ext::*;
use crate::map::*;
use crate::quad_builder::QuadBuilder;



pub enum CellResponse {
	BombHit,
	FlagPlaced,
	FlagRemoved,
	OpenSpaceUncovered,
	UnsafeSpaceUncovered,
}


pub struct BoardView {
	main_vs: gfx::ShaderHandle,
	main_fs: gfx::ShaderHandle,
	atlas: gfx::ImageHandle,
	sampler: gfx::SamplerName,

	pub bounds: Aabb2,
	pub hovered_cell: Option<Vec2i>,

	cell_bounds: Map<Aabb2>,
}


impl BoardView {
	pub fn new(ctx: &mut toybox::Context, board_size: Vec2i) -> anyhow::Result<BoardView> {
		let core = &mut ctx.gfx.core;
		let rm = &mut ctx.gfx.resource_manager;

		let bounds = Self::make_bounds(board_size);

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
			cell_bounds: Self::make_cells(board_size, bounds),
			hovered_cell: None,
		})
	}

	pub fn reset(&mut self, board_size: Vec2i) {
		self.bounds = Self::make_bounds(board_size);
		self.cell_bounds = Self::make_cells(board_size, self.bounds);
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, board: &mut Board, mouse_pos: Option<Vec2>) -> Option<(Vec2i, CellResponse)> {
		// Find the first cell underneath the mouse, if any.
		self.hovered_cell = mouse_pos.and_then(|mouse_pos|
			self.cell_bounds.iter_with_positions()
				.filter(|(_, bounds)| bounds.contains_point(mouse_pos))
				.map(|(position, _)| position)
				.next()
		);

		let Some(cell_position) = self.hovered_cell else { return None };

		let cell_state = board.states.get_mut(cell_position).unwrap();
		if *cell_state == CellState::Opened {
			return None;
		}

		if ctx.input.button_just_down(input::MouseButton::Right) {
			let response = match *cell_state {
				CellState::Flagged => {
					*cell_state = CellState::Unopened;
					CellResponse::FlagRemoved
				}

				CellState::Unopened => {
					*cell_state = CellState::Flagged;
					CellResponse::FlagPlaced
				}

				_ => unreachable!()
			};

			return Some((cell_position, response))
		}

		if *cell_state == CellState::Unopened && ctx.input.button_just_down(input::MouseButton::Left) {
			*cell_state = CellState::Opened;

			let cell_type = *board.types.get(cell_position).unwrap();
			let response = match cell_type {
				CellType::Bomb => CellResponse::BombHit,
				CellType::Empty => CellResponse::OpenSpaceUncovered,
				CellType::BombAdjacent(_) => CellResponse::UnsafeSpaceUncovered,
			};

			return Some((cell_position, response))
		}

		None
	}

	fn make_bounds(board_size: Vec2i) -> Aabb2 {
		let Vec2{x, y} = board_size.to_vec2();
		let aspect = x/y;

		let extent = Vec2::new(aspect, 1.0);

		Aabb2::new(-extent, extent)
			.shrink(Vec2::splat(0.05))
	}

	fn make_cells(board_size: Vec2i, bounds: Aabb2) -> Map<Aabb2> {
		Map::new_with(board_size, |pos| {
			bounds.section(board_size, pos).scale_about_center(Vec2::splat(0.95))
		})
	}


	pub fn draw(&self, gfx: &mut gfx::System, board: &Board) {
		let mut builder = QuadBuilder::default();

		builder.add(self.bounds, Color::grey(0.2), 0);


		let types_and_states = std::iter::zip(board.types.iter(), board.states.iter());

		for ((position, cell_bounds), (cell_type, cell_state)) in self.cell_bounds.iter_with_positions().zip(types_and_states) {
			let is_hovered = self.hovered_cell == Some(position);
			draw_cell(&mut builder, *cell_bounds, *cell_type, *cell_state, is_hovered);
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






fn draw_cell(builder: &mut QuadBuilder, bounds: Aabb2, cell_type: CellType, cell_state: CellState, is_hovered: bool) {
	match cell_state {
		CellState::Unopened => draw_cell_unopened(builder, bounds, is_hovered),
		CellState::Flagged => draw_cell_flag(builder, bounds, is_hovered),
		CellState::Opened => draw_cell_opened(builder, bounds, cell_type),
	}
}

fn draw_cell_unopened(builder: &mut QuadBuilder, bounds: Aabb2, is_hovered: bool) {
	let bg_color = match is_hovered {
		false => Color::grey(0.3),
		true => Color::grey(0.7),
	};

	builder.add(bounds, bg_color, 0);
}

fn draw_cell_flag(builder: &mut QuadBuilder, bounds: Aabb2, is_hovered: bool) {
	draw_cell_unopened(builder, bounds, is_hovered);
	builder.add(bounds, Color::white(), 9);
}

fn draw_cell_opened(builder: &mut QuadBuilder, bounds: Aabb2, cell_type: CellType) {
	match cell_type {
		CellType::Empty => {},

		CellType::Bomb => {
			builder.add(bounds, Color::white(), 10);
		}

		CellType::BombAdjacent(count) => {
			builder.add(bounds, Color::white(), count as u16);
		}
	}
}