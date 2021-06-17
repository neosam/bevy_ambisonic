use bevy::prelude::*;
use bevy_ambisonic::Velocity;

fn startup_system(mut commands: Commands) {
    commands.spawn_bundle(bevy_ambisonic::AmbisonicBundle {
        transform: Transform::from_xyz(-50.0, 0.0, 0.0),
        source: bevy_ambisonic::AmbisonicSource::new_size(440, true),
    }).insert(bevy_ambisonic::Velocity(Vec3::new(20.0, 0.0, 5.0)));
}

fn update_system(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, bevy_ambisonic::Velocity(velocity)) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
        transform.translation.z += velocity.z * time.delta_seconds();
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_non_send_resource(bevy_ambisonic::AmbisonicResource::default())
        .add_startup_system(bevy_ambisonic::ambisonic_startup_system.system())
        .add_startup_system(startup_system.system())
        .add_system(bevy_ambisonic::ambisonic_sync_system.system())
        .add_system(bevy_ambisonic::ambisonic_update_system.system())
        .add_system(update_system.system())
        .run();
}