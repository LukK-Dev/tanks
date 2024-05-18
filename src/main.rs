mod game;
mod player;
mod turret;

use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::default().add_plugins(GamePlugin).run();
}
