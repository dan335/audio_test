use bevy::prelude::*;
use bevy_steamworks::*;
use voice::VoicePlugin;
use bevy_oddio::{AudioPlugin};

mod voice;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SteamworksPlugin::new(AppId(849780)))
        .add_plugin(AudioPlugin)
        .add_plugin(VoicePlugin)
        .run();
}
