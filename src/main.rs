use bevy::prelude::*;
use tanks::game::GamePlugin;

fn main() {
    App::default().add_plugins(GamePlugin).run();
}
