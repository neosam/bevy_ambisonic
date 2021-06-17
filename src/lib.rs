use ambisonic::rodio::source;
use bevy::prelude::*;

pub struct AmbisonicController {
    controller: ambisonic::SoundController,
}

#[derive(Clone)]
pub struct AmbisonicSource {
    data: Vec<f32>,
    current_frame_len: Option<usize>,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<std::time::Duration>,
    current_index: usize,
    repeat: bool,
}
impl ambisonic::rodio::Source for AmbisonicSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.current_frame_len
    }

    fn channels(&self) -> u16 {
        self.channels
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
    pub fn from_source<T: source::Source<Item=f32>>(source: T, repeat: bool) -> Self {
        let current_frame_len = source.current_frame_len();
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();
        let current_index = 0;
        let data = (0u16..u16::MAX).zip(source)
            .map(|(_, x)| x)
            .collect();
        AmbisonicSource {
            data, current_frame_len, channels, sample_rate, total_duration, current_index, repeat
        }
    }

    pub fn new_size(freq: u32, repeat: bool) -> Self {
        let sample = source::SineWave::new(freq);
        AmbisonicSource::from_source(sample, repeat)
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

pub fn ambisonic_sync_system(
    mut query: Query<(Entity, &AmbisonicSource, &Transform, &Velocity)>,
    scene: NonSend<AmbisonicResource>,
    mut commands: Commands,
) {
    for (entity, source, transform, velocity) in query.iter_mut() {
        dbg!("Sync");
        commands.entity(entity).remove::<AmbisonicSource>();
        let source: AmbisonicSource = source.clone();
        if let Some(ref scene) = &scene.audio_scene {
            let mut sound = AmbisonicController {
                controller: scene.play_at(source, [transform.translation[0], transform.translation[1], transform.translation[2]]),
            };
            //if let Some(Velocity(velocity)) = velocity {
                sound.controller.set_velocity([velocity.0[0], velocity.0[1], velocity.0[2]]);
            //}
            commands.entity(entity).insert(sound);
        } else {
            dbg!("Ambisonic not initialized");
        }
    }
}

pub fn ambisonic_update_system(
    mut query: Query<(&Transform, &mut AmbisonicController, Option<&Velocity>)>,
) {
    for (transform, mut source, velocity) in query.iter_mut() {
        source.controller.adjust_position([transform.translation[0], transform.translation[1], transform.translation[2]]);
        if let Some(Velocity(velocity)) = velocity {
            source.controller.set_velocity([velocity[0], velocity[1], velocity[2]]);
        }
    }
}

#[derive(Bundle)]
pub struct AmbisonicBundle {
    pub transform: Transform,
    pub source: AmbisonicSource,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
