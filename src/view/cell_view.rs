use toybox::prelude::*;

use crate::ext::*;
use crate::board::{CellType, CellState};
use crate::quad_builder::QuadBuilder;


#[derive(Copy, Clone, Debug)]
pub struct CellView {
	pub bounds: Aabb2,
	pub is_hovered: bool,
}


pub enum CellResponse {
	BombHit,
	FlagPlaced,
	FlagRemoved,
	OpenSpaceUncovered,
	UnsafeSpaceUncovered,
}


impl CellView {
	pub fn from(bounds: Aabb2) -> CellView {
		CellView {
			bounds,
			is_hovered: false,
		}
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, cell_type: CellType, cell_state: &mut CellState, safe_zone: f32) -> Option<CellResponse> {
		let Some(mouse_pos_ndc) = ctx.input.pointer_position() else {
			self.is_hovered = false;
			return None
		};

		// TODO(pat.m): how to not need to do this
		let mouse_pos_view = match ctx.gfx.backbuffer_aspect() {
			aspect if aspect >= 1.0 => mouse_pos_ndc * Vec2::new(aspect, 1.0) * safe_zone,
			aspect => mouse_pos_ndc * Vec2::new(1.0, 1.0/aspect) * safe_zone,
		};

		self.is_hovered = self.bounds.contains_point(mouse_pos_view);

		if self.is_hovered && *cell_state != CellState::Opened {
			if ctx.input.button_just_down(input::MouseButton::Right) {
				return match *cell_state {
					CellState::Flagged => {
						*cell_state = CellState::Unopened;
						Some(CellResponse::FlagRemoved)
					}

					CellState::Unopened => {
						*cell_state = CellState::Flagged;
						Some(CellResponse::FlagPlaced)
					}

					_ => unreachable!()
				};
			}

			if *cell_state == CellState::Unopened && ctx.input.button_just_down(input::MouseButton::Left) {
				*cell_state = CellState::Opened;

				return match cell_type {
					CellType::Bomb => Some(CellResponse::BombHit),
					CellType::Empty => Some(CellResponse::OpenSpaceUncovered),
					CellType::BombAdjacent(_) => Some(CellResponse::UnsafeSpaceUncovered),
				};
			}
		}

		None
	}

	pub fn draw(&self, builder: &mut QuadBuilder, cell_type: CellType, cell_state: CellState) {
		match cell_state {
			CellState::Unopened => self.draw_unopened(builder),
			CellState::Flagged => self.draw_flag(builder),
			CellState::Opened => self.draw_opened(builder, cell_type),
		}
	}

	fn draw_unopened(&self, builder: &mut QuadBuilder) {
		let bg_color = match self.is_hovered {
			false => Color::grey(0.3),
			true => Color::grey(0.7),
		};

		builder.add(self.bounds, bg_color, 0);
	}

	fn draw_flag(&self, builder: &mut QuadBuilder) {
		self.draw_unopened(builder);
		builder.add(self.bounds, Color::white(), 9);
	}

	fn draw_opened(&self, builder: &mut QuadBuilder, cell_type: CellType) {
		match cell_type {
			CellType::Empty => {},

			CellType::Bomb => {
				builder.add(self.bounds, Color::white(), 10);
			}

			CellType::BombAdjacent(count) => {
				builder.add(self.bounds, Color::white(), count as u16);
				
				// self.draw_pips(builder, count);
			}
		}
	}

	#[allow(unused)]
	fn draw_pips(&self, builder: &mut QuadBuilder, count: usize) {
		builder.add(self.bounds, Color::from([1.0, 0.5, 1.0]), 0);

		let cell_extent = self.bounds.size() / 2.0;

		let marker_bounds = self.bounds.shrink(Vec2::new(cell_extent.x*0.5, cell_extent.y * 0.6));

		let pip_extent = cell_extent / 9.0;

		if count > 3 {
			let top_row = count / 2;
			let bottom_row = count - top_row;

			let (bottom_bounds, top_bounds) = marker_bounds.split_once_vertical(0.5);

			for pip_bounds in bottom_bounds.split(Vec2i::new(bottom_row as i32, 1)) {
				let pip = Aabb2::around_point(pip_bounds.center(), pip_extent);
				builder.add(pip, Color::grey(0.01), 0);
			}

			for pip_bounds in top_bounds.split(Vec2i::new(top_row as i32, 1)) {
				let pip = Aabb2::around_point(pip_bounds.center(), pip_extent);
				builder.add(pip, Color::grey(0.01), 0);
			}

		} else {
			for pip_bounds in marker_bounds.split(Vec2i::new(count as i32, 1)) {
				let pip = Aabb2::around_point(pip_bounds.center(), pip_extent);
				builder.add(pip, Color::grey(0.01), 0);
			}
		}
	}
}