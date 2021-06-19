use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::{Camera, ScalingMode}};

struct Marker;

fn startup_system(
    mut commands: Commands,
    texture_asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let handle = texture_asset_server.load("test.wav");

    let microphone = commands.spawn()
        .insert(Transform::from_xyz(0.0, 0.0, -1001.0))
        .insert(GlobalTransform::default())
        .insert(bevy_ambisonic::AmbisonicCenter)
        .id();

    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scaling_mode = ScalingMode::FixedHorizontal;
    camera_bundle.orthographic_projection.scale = 2.0;
    let _camera = commands.spawn_bundle(camera_bundle)
        .push_children(&[microphone])
        .id();

    let sound = commands
        .spawn_bundle(bevy_ambisonic::AmbisonicBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            controller: bevy_ambisonic::AmbisonicSource::new(handle),
        })
        .insert(GlobalTransform::default())
        .id();

    let texture = texture_asset_server.load("icon.png");
    let sprite = commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(0.3, 0.3)),
        material: materials.add(texture.into()),
        transform: Transform::from_xyz(-1.0, 0.0, 0.0),
        ..SpriteBundle::default()
    })
        .push_children(&[sound])
        .id();
    let _center = commands.spawn()
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(Marker)
        .push_children(&[sprite]);
}

fn update_sprite(mut query: Query<&mut Transform, With<Marker>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_z(PI / 8.0 * time.delta_seconds()));
    }
}
fn update_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    if let Ok(mut transform) = query.single_mut() {
        transform.translation.x = (time.seconds_since_startup().sin() * 1.0) as f32;
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_ambisonic::AmbisonicPlugin)
        .add_startup_system(startup_system.system())
        .add_system(update_sprite.system())
        .add_system(update_camera.system())
        .run();
}
