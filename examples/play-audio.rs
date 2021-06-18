use bevy::prelude::*;
use bevy_ambisonic::Velocity;

fn startup_system(
    mut commands: Commands,
    mut sources: ResMut<Assets<bevy_ambisonic::AmbisonicSource>>,
) {
    let handle = sources.add(bevy_ambisonic::AmbisonicSource::new_size(440, true));
    commands
        .spawn_bundle(bevy_ambisonic::AmbisonicBundle {
            transform: Transform::from_xyz(-50.0, 0.0, 0.0),
            controller: bevy_ambisonic::AmbisonicController::new(handle),
        })
        .insert(bevy_ambisonic::Velocity(Vec3::new(20.0, 0.0, 5.0)));
}

fn update_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, bevy_ambisonic::Velocity(velocity)) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
        transform.translation.z += velocity.z * time.delta_seconds();
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_ambisonic::AmbisonicPlugin)
        .add_startup_system(startup_system.system())
        .add_system(update_system.system())
        .run();
}
