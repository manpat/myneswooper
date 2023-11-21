#![feature(array_chunks)]

use toybox::prelude::*;


fn main() -> anyhow::Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	toybox::run("my-nesweeper", App::new)
}

mod ext;
use ext::*;

mod sound;
use sound::{SoundSystem, Sound};

mod quad_builder;
mod board;
mod view;

use board::Board;
use view::*;


struct App {
	board: Board,
	board_view: BoardView,

	sound: SoundSystem,

	board_size: Vec2i,
	num_bombs: usize,

	debug_board: bool,
}

impl App {
	fn new(ctx: &mut toybox::Context) -> anyhow::Result<App> {
		let board_size = Vec2i::new(8, 8);
		let num_bombs = 5;

		let board = Board::with_bombs(board_size, num_bombs);
		let board_view = BoardView::new(ctx, &board)?;

		Ok(App{
			board,
			board_view,

			sound: SoundSystem::start(&mut ctx.audio)?,

			board_size,
			num_bombs,

			debug_board: false,
		})
	}

	fn show_board_debug(&mut self, ctx: &mut toybox::Context) {
		if !self.debug_board {
			return;
		}

		let mut do_reset = false;

		egui::Window::new("Board")
			.open(&mut self.debug_board)
			.show(&ctx.egui, |ui| {
				ui.add(egui::DragValue::new(&mut self.board_size.x).clamp_range(2..=30));
				ui.add(egui::DragValue::new(&mut self.board_size.y).clamp_range(2..=30));
				ui.add(egui::DragValue::new(&mut self.num_bombs).clamp_range(1..=100));

				if ui.button("Reset").clicked() {
					do_reset = true;
				}
			});

		if do_reset {
			self.reset();
		}
	}

	fn reset(&mut self) {
		self.board = Board::with_bombs(self.board_size, self.num_bombs);
		self.board_view.reset(&self.board);
	}
}

impl toybox::App for App {
	fn present(&mut self, ctx: &mut toybox::Context) {
		self.show_board_debug(ctx);

		if ctx.input.button_just_down(input::Key::F5) {
			self.reset();
		}

		let _ = ctx.gfx.frame_encoder.command_group("main");

		ctx.gfx.frame_encoder.backbuffer_color([0.1; 3]);


		// TODO(pat.m): this is convoluted and confusing.
		// there should be utilies for constructing an ortho matrix that preserves a safe _bounding box_ instead of a 1x1 area.
		let aspect = ctx.gfx.backbuffer_aspect();
		let board_aspect = self.board_view.bounds.aspect();

		let safe_zone = if aspect < 1.0 {
			board_aspect
		} else {
			(board_aspect / aspect).max(1.0)
		};

		let global_uniforms = GlobalUniforms {
			projection: {
				Mat4::ortho_aspect(safe_zone, aspect, -1.0, 1.0)
			}
		};


		ctx.gfx.frame_encoder.bind_global_ubo(0, &[global_uniforms]);

		self.board_view.update(ctx, &self.sound, &mut self.board, safe_zone);
	}

	fn customise_debug_menu(&mut self, ui: &mut egui::Ui) {
		ui.menu_button("Debug", |_ui| {
			self.debug_board = true;
		});

		if ui.button("Reset").clicked() {
			self.reset();
		}

		if ui.button("Plik").clicked() {
			self.sound.play(Sound::Plik);
		}
	}
}



#[repr(C)]
#[derive(Copy, Clone)]
struct GlobalUniforms {
	projection: Mat4,
}