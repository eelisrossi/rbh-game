use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;
use std::f32::consts::PI;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SIZE: f32 = 64.0; // This is the player sprite size.

// TODO: whenever the number of enemies is increased
// also increase the safe area around the player so the enemies
// don't collide on spawn too much
pub const NUMBER_OF_ENEMIES: usize = 100;
pub const ENEMY_SPEED: f32 = 100.0;
pub const ENEMY_SIZE: f32 = 64.0;
pub const PLAYER_SAFE_AREA: f32 = 700.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(PostStartup, spawn_enemies)
        .add_systems(Update, player_movement)
        .add_systems(Update, camera_track_player)
        .add_systems(Update, enemy_movement)
        .run();
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2,
}

pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("sprites/ball_blue_large.png"),
            ..default()
        },
        Player {},
    ));
}

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ..default()
    });
}

pub fn camera_track_player(
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    player_transform: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera_trans = camera_transform.single_mut();
    let playertrans = player_transform.single().translation.truncate();
    let camtrans = camera_trans.translation.truncate();
    camera_trans.translation = camtrans.lerp(playertrans, 0.1).extend(999.0);
}

pub fn spawn_enemies(
    mut commands: Commands,
    player_transform: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    let playertrans = player_transform.single().translation.truncate();

    for _ in 0..NUMBER_OF_ENEMIES {
        // spawns enemies on the circumference of a circle
        // where r = PLAYER_SAFE_AREA + (rand * int)
        // and angle = rand * 2 * PI
        let radius = PLAYER_SAFE_AREA + (random::<f32>() * 100.0);
        let theta = random::<f32>() * 2.0 * PI;
        let random_x = playertrans.x + radius * theta.cos();
        let random_y = playertrans.y + radius * theta.sin();

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 0.0),
                texture: asset_server.load("sprites/ball_red_large.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(random::<f32>(), random::<f32>()).normalize(),
            },
        ));
    }
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

pub fn enemy_movement(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_transform: Query<(&mut Transform, &Enemy)>,
    time: Res<Time>,
) {
    let player_transform = player_query.single();

    for (mut transform, _enemy) in &mut enemy_transform {
        let direction = (transform.translation.truncate()
            - player_transform.translation.truncate())
        .normalize();
        transform.translation -= (direction * time.delta_seconds() * ENEMY_SPEED).extend(0.);
    }
}
