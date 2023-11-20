use toybox::prelude::*;

use crate::ext::*;
use crate::board::{Cell};
use crate::quad_builder::QuadBuilder;


#[derive(Copy, Clone, Debug)]
pub struct CellView {
	pub bounds: Aabb2,
	pub position: Vec2i,
	pub state: CellState,
	pub is_hovered: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CellState {
	Unopened,
	Flagged,
	Opened,
}


pub enum CellResponse {
	BombHit,
	FlagPlaced,
	OpenSpaceUncovered,
}


impl CellView {
	pub fn from(_cell: &Cell, bounds: Aabb2, position: Vec2i) -> CellView {
		CellView {
			bounds,
			position,
			state: CellState::Unopened,
			is_hovered: false,
		}
	}

	pub fn update(&mut self, ctx: &mut toybox::Context, cell: &Cell) -> Option<CellResponse> {
		let Some(mouse_pos_ndc) = ctx.input.pointer_position() else {
			self.is_hovered = false;
			return None
		};

		// TODO(pat.m): how to not need to do this
		let mouse_pos_view = match ctx.gfx.backbuffer_aspect() {
			aspect if aspect >= 1.0 => mouse_pos_ndc * Vec2::new(aspect, 1.0),
			aspect => mouse_pos_ndc * Vec2::new(1.0, 1.0/aspect),
		};

		self.is_hovered = self.bounds.contains_point(mouse_pos_view);

		if self.is_hovered && self.state != CellState::Opened {
			if ctx.input.button_just_down(input::MouseButton::Right) {
				match self.state {
					CellState::Flagged => {
						self.state = CellState::Unopened;
					}

					CellState::Unopened => {
						self.state = CellState::Flagged;
						return Some(CellResponse::FlagPlaced);
					}

					_ => unreachable!()
				};
			}

			if self.state == CellState::Unopened && ctx.input.button_just_down(input::MouseButton::Left) {
				self.state = CellState::Opened;

				return match *cell {
					Cell::Bomb => Some(CellResponse::BombHit),
					Cell::Empty => Some(CellResponse::OpenSpaceUncovered),
					_ => None
				};
			}
		}

		None
	}

	pub fn draw(&self, builder: &mut QuadBuilder, cell: &Cell) {
		match self.state {
			CellState::Unopened => self.draw_unopened(builder),
			CellState::Flagged => self.draw_flag(builder),
			CellState::Opened => self.draw_opened(builder, cell),
		}
	}

	fn draw_unopened(&self, builder: &mut QuadBuilder) {
		let bg_color = match self.is_hovered {
			false => Color::grey(0.3),
			true => Color::grey(0.7),
		};

		builder.add(self.bounds, bg_color);
	}

	fn draw_flag(&self, builder: &mut QuadBuilder) {
		self.draw_unopened(builder);
		builder.add(self.bounds.scale_about_center(Vec2::splat(0.5)), Color::white());
	}

	fn draw_opened(&self, builder: &mut QuadBuilder, cell: &Cell) {
		let cell_color = match cell {
			Cell::Empty => return,

			Cell::Bomb => Color::grey(0.01),
			Cell::BombAdjacent(_) => Color::from([1.0, 0.5, 1.0]),
		};

		builder.add(self.bounds, cell_color);

		if let Cell::BombAdjacent(count) = *cell {
			let cell_extent = self.bounds.size() / 2.0;

			let marker_bounds = self.bounds.shrink(Vec2::new(cell_extent.x*0.5, cell_extent.y * 0.6));

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
}