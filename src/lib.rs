//! # Ambisonic plugin for the bevy game engine
//!
//! This library adds surround sound by using the ambisonic crate.
//! It comes with an own asset loader to load AmbisonicSample's from
//! WAV, MP3, OGG and TRAC.  These assets can be assigned to a an
//! entity using the AmbisonicSource component and you hear the sound
//! from the entities GlobalTransform position.  Adjust the origin of
//! the component by assigning the AmbisonicCenter component to an
//! entity which also has a GlobalPosition.
//!
//! ## Example
//!
//! ``` no_run
//! use bevy::prelude::*;
//!
//! fn startup_system(
//!     mut commands: Commands,
//!     mut sources: ResMut<Assets<bevy_ambisonic::AmbisonicSample>>,
//!     // If audio files should be loaded
//!     // asset_server: Res<AssetServer>,
//! ) {
//!     // Generate an infinite sine sample of 440Hz.
//!     let handle = sources.add(bevy_ambisonic::AmbisonicSample::new_sine(440, true));
//!
//!     // You could also load an audio file
//!     // let handle = asset_server.load("my-audio-file.ogg");
//!     commands
//!         .spawn_bundle(bevy_ambisonic::AmbisonicBundle {
//!             transform: Transform::from_xyz(-50.0, 0.0, 0.0),
//!             controller: bevy_ambisonic::AmbisonicSource::new(handle),
//!         })
//!         .insert(bevy_ambisonic::Velocity(Vec3::new(20.0, 0.0, 5.0)))
//!         .insert(GlobalTransform::default());
//! }
//! fn main() {
//!     App::build()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(bevy_ambisonic::AmbisonicPlugin)
//!         .add_startup_system(startup_system.system())
//!         .run();
//! }
//! ```

use std::io::Cursor;
use std::sync::Arc;

use ambisonic::rodio::source;
use bevy::asset::{AssetLoader, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

#[derive(Clone, TypeUuid)]
#[uuid = "7cb323df-ef1e-47ef-b64a-d0996c631894"]
pub struct AmbisonicSample {
    data: Arc<[f32]>,
    current_frame_len: Option<usize>,
    sample_rate: u32,
    total_duration: Option<std::time::Duration>,
    repeat: bool,
    current_index: usize,
}
impl ambisonic::rodio::Source for AmbisonicSample {
    fn current_frame_len(&self) -> Option<usize> {
        self.current_frame_len
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.total_duration
    }
}
impl Iterator for AmbisonicSample {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.repeat && self.current_index >= self.data.len() {
            self.current_index = 0;
        }
        if self.current_index >= self.data.len() {
            None
        } else {
            let result = self.data[self.current_index];
            self.current_index += 1;
            Some(result)
        }
    }
}
impl AmbisonicSample {
    pub fn from_source_f32<T: source::Source<Item = f32>>(source: T, repeat: bool) -> Self {
        let current_frame_len = source.current_frame_len();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();
        let current_index = 0;
        let data = (0u16..u16::MAX).zip(source).map(|(_, x)| x).collect();
        AmbisonicSample {
            data,
            current_frame_len,
            sample_rate,
            total_duration,
            repeat,
            current_index,
        }
    }
    pub fn from_source_i16<T: source::Source<Item = i16>>(source: T, repeat: bool) -> Self {
        let current_frame_len = source.current_frame_len();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();
        let current_index = 0;
        let data = (0u16..u16::MAX).zip(source)
            .map(|(_, x)| (x as f32) / (i16::MAX as f32))
            .collect();
        AmbisonicSample {
            data,
            current_frame_len,
            sample_rate,
            total_duration,
            repeat,
            current_index,
        }
    }

    pub fn new_sine(freq: u32, repeat: bool) -> Self {
        let sample = source::SineWave::new(freq);
        AmbisonicSample::from_source_f32(sample, repeat)
    }
}
pub struct AmbisonicSource {
    controller: Option<ambisonic::SoundController>,
    handle: Handle<AmbisonicSample>,
}
impl AmbisonicSource {
    pub fn new(handle: Handle<AmbisonicSample>) -> Self {
        AmbisonicSource {
            controller: None,
            handle,
        }
    }
    pub fn restart(&mut self) {
        if let Some(ref controller) = self.controller {
            controller.stop();
        }
        self.controller = None;
    }
}

#[derive(Clone)]
pub struct Velocity(pub Vec3);

#[derive(Default)]
pub struct AmbisonicResource {
    audio_scene: Option<ambisonic::Ambisonic>,
}

pub fn ambisonic_startup_system(mut ambisonic_res: NonSendMut<AmbisonicResource>) {
    ambisonic_res.audio_scene = Some(ambisonic::AmbisonicBuilder::default().build());
}

pub fn amibsonic_shutdown_system(mut ambisonic_res: NonSendMut<AmbisonicResource>) {
    ambisonic_res.audio_scene = None;
}

pub fn ambisonic_update_system(
    sources: Res<Assets<AmbisonicSample>>,
    scene: NonSend<AmbisonicResource>,
    mut query: QuerySet<(
        Query<(&GlobalTransform, &mut AmbisonicSource, Option<&Velocity>)>,
        Query<&GlobalTransform, With<AmbisonicCenter>>
    )>,
) {
    let center = match query.q1().single() {
        Ok(center) => *center,
        _ => GlobalTransform::from_xyz(0.0, 0.0, 0.0),
    };
    for (transform, mut controller, velocity) in query.q0_mut().iter_mut() {
        let matrix = transform.compute_matrix();
        let final_matrix = center.compute_matrix().inverse() * matrix;
        let transform = GlobalTransform::from_matrix(final_matrix);
        if let Some(ref mut controller) = controller.controller {
            controller.adjust_position([
                transform.translation[0],
                transform.translation[1],
                transform.translation[2],
            ]);
            if let Some(Velocity(velocity)) = velocity {
                controller.set_velocity([velocity[0], velocity[1], velocity[2]]);
            }
        } else if let (Some(source), Some(ref scene)) =
            (sources.get(controller.handle.clone()), &scene.audio_scene)
        {
            let mut sound_controller = scene.play_at(
                source.clone(),
                [
                    transform.translation[0],
                    transform.translation[1],
                    transform.translation[2],
                ],
            );
            if let Some(Velocity(velocity)) = velocity {
                sound_controller.set_velocity([velocity[0], velocity[1], velocity[2]]);
            }
            controller.controller = Some(sound_controller);
        }
    }
}

#[derive(Bundle)]
pub struct AmbisonicBundle {
    pub transform: Transform,
    pub controller: AmbisonicSource,
}

pub struct AmbisonicCenter;

pub struct AmbisonicPlugin;
impl Plugin for AmbisonicPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<AmbisonicSample>()
            .init_asset_loader::<AmbisonicLoader>()
            .insert_non_send_resource(AmbisonicResource::default())
            .add_startup_system(ambisonic_startup_system.system())
            .add_system(ambisonic_update_system.system());
    }
}

#[derive(Default)]
pub struct AmbisonicLoader;
impl AssetLoader for AmbisonicLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        let reader = Cursor::new(bytes);
        Box::pin(async move {
            let sample = ambisonic::rodio::Decoder::new(reader)?;
            load_context.set_default_asset(LoadedAsset::new(AmbisonicSample::from_source_i16(sample, false)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3", "wav", "ogg", "flac"]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
