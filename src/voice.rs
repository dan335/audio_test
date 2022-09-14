use bevy::{
    prelude::{*, App, Assets, Commands, Handle, Res, ResMut, StartupStage},
    reflect::TypeUuid,
};
use bevy_steamworks::Client;
use bevy_oddio::{
    frames::Mono, oddio, output::AudioSink, Audio, AudioApp,
    ToSignal,
};

const VOICE_OUTPUT_SAMPLE_RATE: u32 = 11025;
const BYTES_PER_SAMPLE: u32 = 2;

pub struct VoiceInput {
    pub is_recording: bool,
}

impl Default for VoiceInput {
    fn default() -> VoiceInput {
        VoiceInput {
            is_recording: false,
        }
    }
}

pub struct VoicePlugin;

impl Plugin for VoicePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(VoiceInput::default())
            .add_audio_source::<1, _, NetworkAudio>()
            .add_startup_system(init_assets)
            .add_startup_system_to_stage(StartupStage::PostStartup, init_audio)
            .add_system(keyboard_input)
            .add_system(record_play_audio)
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



fn record_play_audio (
    voice_data: Res<VoiceInput>,
    steam_client: Res<Client>,
    sink_handle: Res<NetworkAudioSink>,
    mut sinks: ResMut<Assets<AudioSink<NetworkAudio>>>,
) {
    if voice_data.is_recording {
        let mut pcb_compressed: u32 = 0;
        let res = steam_client.user().get_available_voice(&mut pcb_compressed);

        match res {
            Ok(_) => {
                
                if pcb_compressed > 0 {

                    let mut voice_capture_buffer: Vec<u8> = vec![0; 1024];
                    let voice_capture_buffer_size: u32 = 1024;
                    let mut voice_capture_bytes_written: u32 = 0;

                    let voice_result = steam_client.user().get_voice(&mut voice_capture_buffer, voice_capture_buffer_size, &mut voice_capture_bytes_written);

                    match voice_result {
                        Ok(_) => {
                            if voice_capture_bytes_written > 0 {

                                let mut compressed_audio = voice_capture_buffer.clone();
                                compressed_audio.truncate(voice_capture_bytes_written.try_into().unwrap());

                                let mut uncompressed_audio: Vec<u8> = vec![0; (VOICE_OUTPUT_SAMPLE_RATE * BYTES_PER_SAMPLE).try_into().unwrap()];
				                let uncompressed_audio_size: u32 = VOICE_OUTPUT_SAMPLE_RATE * BYTES_PER_SAMPLE;
				                let mut uncompressed_audio_bytes_written: u32 = 0;

				                let res = steam_client.user().decompress_voice(
				                    &compressed_audio,
				                    &mut uncompressed_audio,
				                    uncompressed_audio_size,
				                    &mut uncompressed_audio_bytes_written,
				                    VOICE_OUTPUT_SAMPLE_RATE,
				                );

				                match res {
				                    Ok(_) => {
				                        if uncompressed_audio_bytes_written > 0 {
				                            let sink = match sinks.get_mut(&sink_handle.0) {
				                                Some(sink) => sink,
				                                None => return,
				                            };

				                            let audio_bytes: Vec<Mono> = uncompressed_audio
				                                .into_iter()
				                                // We need to convert u8 to f32. [-1, 1]
				                                .map(|x| (x as f32) / (u8::MAX as f32))
				                                .map(|x| (x * 2.0) - 1.0)
				                                // We need to convert the frame to its asset form.
				                                .map(|x| Mono::from([x,]))
				                                .collect();

				                            sink.control::<oddio::Stream<_>, _>().write(&audio_bytes);
				                        }
				                    },
				                    Err(err) => {
				                        info!("error receiving voice: {:?}", err);
				                    },
				                }
                                
                            }
                        },
                        Err(err) => {
                            match err {
                                bevy_steamworks::VoiceResult::NotInitialized => (),
                                bevy_steamworks::VoiceResult::NotRecording => (),
                                bevy_steamworks::VoiceResult::NoData => (),
                                bevy_steamworks::VoiceResult::BufferTooSmall => (),
                                bevy_steamworks::VoiceResult::DataCorrupted => (),
                                bevy_steamworks::VoiceResult::Restricted => (),
                            }
                        }
                    }
                }

            },
            Err(err) => {
                match err {
                    bevy_steamworks::VoiceResult::NotInitialized => (),
                    bevy_steamworks::VoiceResult::NotRecording => (),
                    bevy_steamworks::VoiceResult::NoData => (),
                    bevy_steamworks::VoiceResult::BufferTooSmall => (),
                    bevy_steamworks::VoiceResult::DataCorrupted => (),
                    bevy_steamworks::VoiceResult::Restricted => (),
                }
            },
        }
    }
}


fn keyboard_input (
    keys: Res<Input<KeyCode>>,
    steam_client: Res<Client>,
    mut voice_input: ResMut<VoiceInput>,
) {
    // up
    if keys.just_pressed(KeyCode::Space) {
        steam_client.user().start_voice_recording();
        voice_input.is_recording = true;
    }
    if keys.just_released(KeyCode::Space) {
        steam_client.user().stop_voice_recording();
        voice_input.is_recording = false;
    }
}