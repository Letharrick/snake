pub mod game;
mod snake;
mod object;
mod direction;

bracket_terminal::add_wasm_support!();

#[cfg(target_arch = "wasm32")]
use bracket_terminal::prelude::BError;
#[cfg(target_arch = "wasm32")]
use game::Game;

#[cfg(target_arch = "wasm32")]
fn main() -> BError {
    Game::new().run()
}