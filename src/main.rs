mod game;
mod object;
mod snake;
mod direction;

use game::Game;

fn main() {
    let game = Game::new();

    game.run();
}
