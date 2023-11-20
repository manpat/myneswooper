use toybox::prelude::*;

use crate::ext::*;
use crate::board::{Board, Cell};


pub struct BoardView {
	main_vs: gfx::ShaderHandle,
	main_fs: gfx::ShaderHandle,
}


impl BoardView {
	pub fn new(ctx: &mut toybox::Context, _board: &Board) -> anyhow::Result<BoardView> {
		let rm = &mut ctx.gfx.resource_manager;

		Ok(BoardView {
			main_vs: rm.request(gfx::LoadShaderRequest::from("shaders/main.vs.glsl")?),
			main_fs: rm.request(gfx::LoadShaderRequest::from("shaders/main.fs.glsl")?),
		})
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, board: &mut Board) {
		let mut builder = QuadBuilder::default();

		let board_bounds = Aabb2::new(Vec2::splat(-1.0), Vec2::splat(1.0)) .expand(Vec2::splat(-0.05));

		builder.add(board_bounds, Color::grey(0.2));

		for (cell_bound, &cell) in board_bounds.split(board.size()).zip(&board.cells.data) {
			let cell_color = match cell {
				Cell::Bomb => Color::grey(0.01),
				Cell::Empty => Color::grey(0.4),
				Cell::BombAdjacent(_) => Color::from([1.0, 0.5, 1.0]),
			};

			builder.add(cell_bound.expand(Vec2::splat(-0.05)), cell_color);

			if let Cell::BombAdjacent(count) = cell {
				let cell_extent = cell_bound.size() / 2.0;

				let marker_bounds = cell_bound.expand(Vec2::new(-cell_extent.x*0.5, -cell_extent.y * 0.6));

				let pip_extent = cell_extent / 9.0;

				if count > 3 {
					let top_row = count / 2;
					let bottom_row = count - top_row;

					let (bottom_bounds, top_bounds) = marker_bounds.split_once_vertical(0.5);

					for pip_bounds in bottom_bounds.split(Vec2i::new(bottom_row as i32, 1)) {
						let pip = Aabb2::around_point(pip_bounds.center(), pip_extent);
						builder.add(pip, Color::grey(0.01));
					}

					for pip_bounds in top_bounds.split(Vec2i::new(top_row as i32, 1)) {
						let pip = Aabb2::around_point(pip_bounds.center(), pip_extent);
						builder.add(pip, Color::grey(0.01));
					}

				} else {
					for pip_bounds in marker_bounds.split(Vec2i::new(count as i32, 1)) {
						let pip = Aabb2::around_point(pip_bounds.center(), pip_extent);
						builder.add(pip, Color::grey(0.01));
					}
				}
			}
		}

		builder.finish();

		let mut group = ctx.gfx.frame_encoder.command_group("main");

		group.draw(self.main_vs, self.main_fs)
			.elements(builder.indices.len() as u32)
			.indexed(&builder.indices)
			.ssbo(0, &builder.vertices)
			.depth_test(false);
	}
}








#[repr(C)]
#[derive(Copy, Clone)]
struct QuadVert {
	pos: Vec2,
	color: [u8; 4],
	_pad: u32,
}

#[derive(Default)]
struct QuadBuilder {
	vertices: Vec<QuadVert>,
	indices: Vec<u32>,
}

impl QuadBuilder {
	fn add(&mut self, Aabb2{min, max}: Aabb2, color: impl Into<Color>) {
		let color = color.into().to_byte_array();

		self.vertices.push(QuadVert{ pos: Vec2::new(min.x, min.y), color, _pad: 0 });
		self.vertices.push(QuadVert{ pos: Vec2::new(min.x, max.y), color, _pad: 0 });
		self.vertices.push(QuadVert{ pos: Vec2::new(max.x, max.y), color, _pad: 0 });
		self.vertices.push(QuadVert{ pos: Vec2::new(max.x, min.y), color, _pad: 0 });
	}

	fn finish(&mut self) {
		let num_quads = self.vertices.len() / 4;

		let indices = (0..num_quads as u32)
			.flat_map(|idx| [0u32, 1, 2, 0, 2, 3].into_iter().map(move |base| base + idx*4));

		self.indices.extend(indices)
	}
}


