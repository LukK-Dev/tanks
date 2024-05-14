mod game;
mod player;

use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::default().add_plugins(GamePlugin).run();
}
