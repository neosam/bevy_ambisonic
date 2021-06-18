use std::sync::Arc;

use ambisonic::rodio::source;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

#[derive(Clone, TypeUuid)]
#[uuid = "7cb323df-ef1e-47ef-b64a-d0996c631894"]
pub struct AmbisonicSource {
    data: Arc<[f32]>,
    current_frame_len: Option<usize>,
    sample_rate: u32,
    total_duration: Option<std::time::Duration>,
    repeat: bool,
    current_index: usize,
}
impl ambisonic::rodio::Source for AmbisonicSource {
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
impl Iterator for AmbisonicSource {
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
impl AmbisonicSource {
    pub fn from_source<T: source::Source<Item = f32>>(source: T, repeat: bool) -> Self {
        let current_frame_len = source.current_frame_len();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();
        let current_index = 0;
        let data = (0u16..u16::MAX).zip(source).map(|(_, x)| x).collect();
        AmbisonicSource {
            data,
            current_frame_len,
            sample_rate,
            total_duration,
            repeat,
            current_index,
        }
    }

    pub fn new_size(freq: u32, repeat: bool) -> Self {
        let sample = source::SineWave::new(freq);
        AmbisonicSource::from_source(sample, repeat)
    }
}
pub struct AmbisonicController {
    controller: Option<ambisonic::SoundController>,
    handle: Handle<AmbisonicSource>,
}
impl AmbisonicController {
    pub fn new(handle: Handle<AmbisonicSource>) -> Self {
        AmbisonicController {
            controller: None,
            handle,
        }
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
    sources: Res<Assets<AmbisonicSource>>,
    scene: NonSend<AmbisonicResource>,
    mut query: QuerySet<(
        Query<(&GlobalTransform, &mut AmbisonicController, Option<&Velocity>)>,
        Query<&GlobalTransform, With<AmbisonicCenter>>
    )>,
) {
    let center = match query.q1().single() {
        Ok(center) => center.clone(),
        _ => GlobalTransform::from_xyz(0.0, 0.0, 0.0),
    };
    for (transform, mut controller, velocity) in query.q0_mut().iter_mut() {
        let matrix = transform.compute_matrix();
        let final_matrix = matrix * center.compute_matrix();
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
    pub controller: AmbisonicController,
}

pub struct AmbisonicCenter;

pub struct AmbisonicPlugin;
impl Plugin for AmbisonicPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<AmbisonicSource>()
            .insert_non_send_resource(AmbisonicResource::default())
            .add_startup_system(ambisonic_startup_system.system())
            .add_system(ambisonic_update_system.system());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
