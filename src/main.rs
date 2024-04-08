use bevy::prelude::*;
use rand::{prelude::SliceRandom, thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, movement_system.after(seek_system))
        .add_systems(Update, z_index_system)
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, seek_system)
        .add_systems(Update, camera_move)
        .run();
}

#[derive(Component, Default)]
struct Moveable {
    walk_speed: f32,
}

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component)]
struct Seek {
    target: Entity,
}

fn seek_system(
    mut e: Query<(&Transform, &Seek, &Moveable, &mut Velocity)>,
    seekables: Query<&Transform>,
) {
    for (t, s, m, mut v) in e.iter_mut() {
        if let Ok(t_other) = seekables.get(s.target) {
            let desired = (t_other.translation - t.translation).normalize_or_zero();
            let desired = desired * m.walk_speed;
            v.0 = desired;
        }
    }
}

fn keyboard_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut e: Query<(&mut Velocity, &Moveable)>,
) {
    for (mut v, m) in e.iter_mut() {
        v.0 = Vec3::default();

        if keyboard.pressed(KeyCode::ArrowRight) {
            v.0.x += m.walk_speed;
        }

        if keyboard.pressed(KeyCode::ArrowLeft) {
            v.0.x -= m.walk_speed;
        }

        if keyboard.pressed(KeyCode::ArrowUp) {
            v.0.y += m.walk_speed;
        }

        if keyboard.pressed(KeyCode::ArrowDown) {
            v.0.y -= m.walk_speed;
        }
    }
}

fn z_index_system(mut e: Query<(&mut Transform), Without<Camera>>) {
    for mut t in e.iter_mut() {
        t.translation.z = -1.0 * t.translation.y;
    }
}

fn movement_system(time: Res<Time>, mut e: Query<(&Velocity, &mut Transform)>) {
    for (v, mut t) in e.iter_mut() {
        t.translation += v.0 * time.delta_seconds();

        if v.0.x > 0.0 {
            t.scale = Vec3::new(1.0, 1.0, 1.0);
        } else if v.0.x < 0.0 {
            t.scale = Vec3::new(-1.0, 1.0, 1.0);
        }

        if v.0.length() > 0.0 {
            let phase = (time.elapsed_seconds() * v.0.length() * 0.5).sin();
            t.rotation.z = phase.remap(-1.0, 1.0, -0.04, 0.04);
            t.scale.y = phase.remap(-1.0, 1.0, 0.8, 1.1);
        } else {
            t.scale.y = 1.0;
            t.rotation.z = 0.0;
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25)),
            ..default()
        })
        .insert(Moveable { walk_speed: -2.0 });

    let mut player = commands.spawn(SpriteBundle {
        texture: asset_server.load("cleric.png"),
        sprite: Sprite {
            anchor: bevy::sprite::Anchor::BottomCenter,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    player
        .insert(Moveable { walk_speed: 40.0 })
        .insert(CameraTarget)
        .insert(Velocity::default());
        
    let player_id = player.id();

    let mut rng = thread_rng();

    for _ in 0..200 {
        commands
            .spawn(SpriteBundle {
                texture: asset_server
                    .load(*["warrior.png", "archer.png"].choose(&mut rng).unwrap()),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    ..default()
                },
                transform: Transform::from_xyz(
                    rng.gen_range(-400.0..400.0) * 1.5,
                    rng.gen_range(-400.0..400.0) * 1.5,
                    0.0,
                ),
                ..default()
            })
            .insert(Moveable { walk_speed: rng.gen_range(10.0..30.0) })
            .insert(Seek { target: player_id })
            .insert(Velocity::default());
    }
}

#[derive(Component)]
struct CameraTarget;

fn camera_move(
    mut camera: Query<&mut Transform, With<Camera>>,
    targets: Query<&Transform, (With<CameraTarget>, Without<Camera>)>,
) {
    if let Ok(mut cam) = camera.get_single_mut() {
        if let Ok(target) = targets.get_single() {
            cam.translation = cam.translation.lerp(target.translation, 0.01);
        }
    }
}
