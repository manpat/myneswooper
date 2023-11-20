use toybox::prelude::*;


fn main() -> anyhow::Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	toybox::run("my-nesweeper", App::new)
}

mod ext;
use ext::*;


mod quad_builder;
mod board;
mod view;

use board::Board;
use view::*;


struct App {
	board: Board,
	board_view: BoardView,

	debug_board: bool,
}

impl App {
	fn new(ctx: &mut toybox::Context) -> anyhow::Result<App> {
		let board = Board::with_bombs(5);
		let board_view = BoardView::new(ctx, &board)?;

		Ok(App{
			board,
			board_view,
			debug_board: false,
		})
	}

	fn show_board_debug(&mut self, ctx: &mut toybox::Context) {
		if !self.debug_board {
			return;
		}

		egui::Window::new("Board")
			.open(&mut self.debug_board)
			.show(&ctx.egui, |ui| {
				ui.label(format!("{:#?}", self.board));
			});
	}

	fn reset(&mut self) {
		self.board = Board::with_bombs(5);
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

		let aspect = ctx.gfx.backbuffer_aspect();
		let global_uniforms = GlobalUniforms {
			projection: Mat4::ortho_aspect(1.0, aspect, -1.0, 1.0),
		};

		ctx.gfx.frame_encoder.bind_global_ubo(0, &[global_uniforms]);

		self.board_view.update(ctx, &mut self.board);
	}

	fn customise_debug_menu(&mut self, ui: &mut egui::Ui) {
		ui.menu_button("Debug", |_ui| {
			self.debug_board = true;
		});

		if ui.button("Reset").clicked() {
			self.reset();
		}
	}
}



#[repr(C)]
#[derive(Copy, Clone)]
struct GlobalUniforms {
	projection: Mat4,
}