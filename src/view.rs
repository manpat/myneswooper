use toybox::prelude::*;

use crate::board::*;
use crate::ext::*;
use crate::map::*;
use crate::quad_builder::QuadBuilder;

mod cell_view;
use cell_view::*;

pub use cell_view::CellResponse;


pub struct BoardView {
	main_vs: gfx::ShaderHandle,
	main_fs: gfx::ShaderHandle,
	atlas: gfx::ImageHandle,
	sampler: gfx::SamplerName,

	pub bounds: Aabb2,
	cells: Map<CellView>,
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
			cells: Self::make_cells(board_size, bounds),
		})
	}

	pub fn reset(&mut self, board_size: Vec2i) {
		self.bounds = Self::make_bounds(board_size);
		self.cells = Self::make_cells(board_size, self.bounds);
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, board: &mut Board, safe_zone: f32, cell_responses: &mut Vec<(Vec2i, CellResponse)>) {
		let types_and_states = std::iter::zip(board.types.iter(), board.states.iter_mut());

		for ((cell_position, cell_view), (&cell_type, cell_state)) in self.cells.iter_mut_with_positions().zip(types_and_states) {
			if let Some(response) = cell_view.update(ctx, cell_type, cell_state, safe_zone) {
				cell_responses.push((cell_position, response));
			}
		}
	}

	fn make_bounds(board_size: Vec2i) -> Aabb2 {
		let Vec2{x, y} = board_size.to_vec2();
		let aspect = x/y;

		let extent = Vec2::new(aspect, 1.0);

		Aabb2::new(-extent, extent)
			.shrink(Vec2::splat(0.05))
	}

	fn make_cells(board_size: Vec2i, bounds: Aabb2) -> Map<CellView> {
		Map::new_with(board_size, |pos| {
			let bounds = bounds.section(board_size, pos).scale_about_center(Vec2::splat(0.95));
			CellView::from(bounds)
		})
	}


	pub fn draw(&self, gfx: &mut gfx::System, board: &Board) {
		let mut builder = QuadBuilder::default();

		builder.add(self.bounds, Color::grey(0.2), 0);


		let types_and_states = std::iter::zip(board.types.iter(), board.states.iter());

		for (cell_view, (cell_type, cell_state)) in self.cells.iter().zip(types_and_states) {
			cell_view.draw(&mut builder, *cell_type, *cell_state);
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






