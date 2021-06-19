# Ambisonic plugin for the bevy game engine

This library adds surround sound by using the ambisonic crate.
It comes with an own asset loader to load AmbisonicSample's from
WAV, MP3, OGG and TRAC.  These assets can be assigned to a an
entity using the AmbisonicSource component and you hear the sound
from the entities GlobalTransform position.  Adjust the origin of
the component by assigning the AmbisonicCenter component to an
entity which also has a GlobalPosition.

## Example

``` no_run
use bevy::prelude::*;

fn startup_system(
    mut commands: Commands,
    mut sources: ResMut<Assets<bevy_ambisonic::AmbisonicSample>>,
    // If audio files should be loaded
    // asset_server: Res<AssetServer>,
) {
    // Generate an infinite sine sample of 440Hz.
    let handle = sources.add(bevy_ambisonic::AmbisonicSample::new_sine(440, true));

    // You could also load an audio file
    // let handle = asset_server.load("my-audio-file.ogg");
    commands
        .spawn_bundle(bevy_ambisonic::AmbisonicBundle {
            transform: Transform::from_xyz(-50.0, 0.0, 0.0),
            controller: bevy_ambisonic::AmbisonicSource::new(handle),
        })
        .insert(bevy_ambisonic::Velocity(Vec3::new(20.0, 0.0, 5.0)))
        .insert(GlobalTransform::default());
}
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_ambisonic::AmbisonicPlugin)
        .add_startup_system(startup_system.system())
        .run();
}
```