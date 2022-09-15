use bevy::prelude::*;
use voice::VoicePlugin;
use bevy_oddio::{AudioPlugin};

mod voice;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(VoicePlugin)
        .run();
}
