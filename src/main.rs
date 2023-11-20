#![feature(return_position_impl_trait_in_trait)]

use toybox::prelude::*;


fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    toybox::run("my-nesweeper", App::new)
}

mod ext;
use ext::*;


mod board;
mod view;

use view::*;


struct App {
    board: board::Board,

    board_view: BoardView,
}

impl App {
    fn new(ctx: &mut toybox::Context) -> anyhow::Result<App> {
        use board::*;

        let mut board = Board::empty();

        board.cells.set(Vec2i::new(0, 0), Cell::Bomb);
        // board.cells.set(Vec2i::new(0, 1), Cell::Bomb);
        // board.cells.set(Vec2i::new(0, 2), Cell::Bomb);

        // board.cells.set(Vec2i::new(1, 0), Cell::Bomb);
        board.cells.set(Vec2i::new(1, 2), Cell::Bomb);

        board.cells.set(Vec2i::new(2, 0), Cell::Bomb);
        board.cells.set(Vec2i::new(2, 1), Cell::Bomb);
        board.cells.set(Vec2i::new(2, 2), Cell::Bomb);

        board.rebuild_adjacency();

        let board_view = BoardView::new(ctx, &board)?;
        Ok(App{ board, board_view })
    }
}

impl toybox::App for App {
    fn present(&mut self, ctx: &mut toybox::Context) {
        let _ = ctx.gfx.frame_encoder.command_group("main");

        ctx.gfx.frame_encoder.backbuffer_color([0.1; 3]);

        let aspect = ctx.gfx.backbuffer_aspect();
        let global_uniforms = GlobalUniforms {
            projection: Mat4::ortho_aspect(1.0, aspect, -1.0, 1.0),
        };

        ctx.gfx.frame_encoder.bind_global_ubo(0, &[global_uniforms]);

        self.board_view.update(ctx, &mut self.board);
    }
}



#[repr(C)]
#[derive(Copy, Clone)]
struct GlobalUniforms {
    projection: Mat4,
}