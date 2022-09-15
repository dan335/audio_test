use bevy::{
    prelude::{*, App, Assets, Commands, Handle, Res, ResMut, StartupStage},
    reflect::TypeUuid,
};
use bevy_oddio::{
    frames::Mono, oddio, output::AudioSink, Audio, AudioApp,
    ToSignal,
};

const VOICE_OUTPUT_SAMPLE_RATE: u32 = 11025;
const BYTES_PER_SAMPLE: u32 = 2;


pub struct VoicePlugin;

impl Plugin for VoicePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_audio_source::<1, _, NetworkAudio>()
            .add_startup_system(init_assets)
            .add_startup_system_to_stage(StartupStage::PostStartup, init_audio)
        ;
    }
}


// We create a marker struct to act as a dummy asset.
//
// This mirrors how bevy_audio works internally 
// by handling `Handle<AudioSource>` instead of AudioSource itself.
// The reason is for audio loader ergonomics.
// The consequence is that generative and programmatic usage of audio is harder. 
#[derive(TypeUuid)]
#[uuid = "ac7dad95-8e57-4180-a04e-71da19cbdc88"]
struct NetworkAudio;

// All audio assets in bevy_oddio requires `ToSignal`.
// This is the conversion from asset form to the actual `oddio::Signal`
impl ToSignal for NetworkAudio {
    // Settings needed for initializing `oddio::Stream`. See the documentation there.
    type Settings = (u32, usize);
    // The actual signal. It uses Mono instead of f32 because Asset must be implemented.
    type Signal = oddio::Stream<Mono>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::Stream::new(settings.0, settings.1)
    }
}

// This is the Handle of the NetworkAudio.
// We need this to play the audio.
struct NetworkAudioHandle(Handle<NetworkAudio>);
// This is the handle to the audio sink.
// We need this to control the audio.
struct NetworkAudioSink(Handle<AudioSink<NetworkAudio>>);

// We simply initialize the asset as a startup system.
fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<NetworkAudio>>) {
    let handle = assets.add(NetworkAudio);
    commands.insert_resource(NetworkAudioHandle(handle));
}

// We add the oddio::Stream to the audio mixer.
fn init_audio(
    mut commands: Commands,
    mut audio: ResMut<Audio<Mono, NetworkAudio>>,
    network_audio: Res<NetworkAudioHandle>,
) {
    let handle = audio.play(network_audio.0.clone(), (VOICE_OUTPUT_SAMPLE_RATE , 64));
    commands.insert_resource(NetworkAudioSink(handle));
}