mod game;
use crate::game::Game;

fn main() {
    let game = Game::new();
    game.run();
}
