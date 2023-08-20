use bevy::prelude::*;
use bevy::utils::FloatOrd;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use rand::prelude::*;
use std::f32::consts::PI;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;
const PIXEL_TO_WORLD: f32 = 30. / 720.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Reblhell".into(),
                        resolution: (WIDTH, HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_state::<AppState>()
        // TODO: fix performance with lots of enemies
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), (cleanup_menu, spawn_player))
        .add_systems(OnEnter(AppState::InGame), spawn_enemies)
        // .add_systems(Startup, spawn_player)
        // .add_systems(PostStartup, spawn_enemies)
        .add_systems(
            Update,
            (
                player_movement,
                camera_track_player,
                enemy_movement,
                enemy_damage_player,
                check_player_health,
                close_shot_attack,
                close_shot_bullet,
                enemy_death_check,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // .add_systems(Update, player_movement)
        // .add_systems(Update, camera_track_player)
        // .add_systems(Update, enemy_movement)
        // .add_systems(Update, enemy_damage_player)
        // .add_systems(Update, check_player_health)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
    GameOver,
}

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_menu(mut commands: Commands) {
    let button_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
    commands.insert_resource(ClearColor(Color::rgb(0.16, 0.17, 0.27)));
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

// TODO: HP / Damage
// add movement speed to the struct so it can be altered later
#[derive(Component)]
pub struct Player {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub sprite_size: f32,
}

fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let player_size = 64.0;
    let player_hp = 10.0;

    let close_shot = spawn_close_shot(&mut commands);

    commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
                texture: asset_server.load("sprites/ball_blue_large.png"),
                ..default()
            },
            Player {
                health: player_hp,
                max_health: player_hp,
                speed: 500.0,
                sprite_size: player_size,
            },
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED_Z,
            Damping {
                linear_damping: 100.0,
                angular_damping: 1.0,
            },
            Collider::ball(player_size / 2.0), // for some reason the collider lags just a bit behind the sprite
        ))
        .add_child(close_shot);
}

// TODO: Fix the collider lagging behind when moving
fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    player: Query<&Player>,
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

        transform.translation += direction * player.single().speed * time.delta_seconds();
    }
}

fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ..default()
    });
}

fn camera_track_player(
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    player_transform: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera_trans = camera_transform.single_mut();
    let playertrans = player_transform.single().translation.truncate();
    let camtrans = camera_trans.translation.truncate();
    camera_trans.translation = camtrans.lerp(playertrans, 0.1).extend(999.0);
}

// TODO: Move these constants to the struct for Enemy
// Calculation for the circumference of enemy spawn circle
// C = 2 * PI * R -> R = C / (2 * PI)
// C = NUMBER_OF_ENEMIES * ENEMY_SIZE
pub const NUMBER_OF_ENEMIES: usize = 10;
pub const ENEMY_SPEED: f32 = 200.0;
pub const ENEMY_SIZE: f32 = 64.0;
pub const PLAYER_SAFE_AREA: f32 = (NUMBER_OF_ENEMIES as f32 * (ENEMY_SIZE * 0.8)) / (2.0 * PI);

// TODO: HP / Damage
// add movement speed to the struct so it can be altered later
#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2,
    pub health: f32,
    pub damage_per_second: f32,
    // pub speed: f32,
}

fn spawn_enemies(
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
                health: 2.0,
                damage_per_second: 1.0,
            },
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED_Z,
            Damping {
                linear_damping: 100.0,
                angular_damping: 1.0,
            },
            Collider::ball(ENEMY_SIZE / 2.0),
        ));
    }
}

fn despawn_enemy(
    mut commands: Commands,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemy: Query<(Entity, &Transform), With<Enemy>>,
) {
    let player = player.single();

    for (enemy, transform) in &enemy {
        if Vec2::distance(
            player.translation.truncate(),
            transform.translation.truncate(),
        ) > 30.0
        {
            commands.entity(enemy).despawn_recursive();
        }
    }
}

fn enemy_movement(
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

fn enemy_damage_player(
    enemies: Query<(&Collider, &GlobalTransform, &Enemy)>,
    mut player: Query<&mut Player>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
) {
    for (collider, transform, enemy) in &enemies {
        rapier_context.intersections_with_shape(
            transform.translation().truncate(),
            0.0,
            collider,
            QueryFilter::new(),
            |entity| {
                if let Ok(mut player) = player.get_mut(entity) {
                    player.health -= enemy.damage_per_second * time.delta_seconds();
                    println!("Player's health is: {}", player.health);
                }
                true
            },
        );
    }
}

fn check_player_health(player: Query<&Player>) {
    let player = player.single();

    if player.health <= 0.0 {
        println!("The Player is DEAD!!!");
    }
}

fn damage_enemy(enemy: &mut Enemy, damage: f32) {
    enemy.health -= damage;
}

fn enemy_death_check(mut commands: Commands, mut enemies: Query<(Entity, &Transform, &Enemy)>) {
    //TODO dying animation
    for (entity, _transform, enemy) in &mut enemies {
        if enemy.health <= 0.0 {
            //TODO fire event for sounds
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
pub struct CloseShot {
    pub timer: Timer,
}

#[derive(Component)]
pub struct CloseShotBullet {
    pub lifetime: Timer,
    pub speed: f32,
    pub damage: f32,
    pub direction: Vec2,
}

pub fn spawn_close_shot(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            SpatialBundle::default(),
            Name::new("Close Shot"),
            CloseShot {
                timer: Timer::from_seconds(1.2, TimerMode::Repeating),
            },
        ))
        .id()
}

pub fn spawn_close_shot_bullet(
    commands: &mut Commands,
    assets: &AssetServer,
    spawn_pos: Vec2,
    direction: Vec2,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y, 1.0),
                texture: assets.load("./sprites/ball_blue_small.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(PIXEL_TO_WORLD * 1000.0)),
                    ..default()
                },
                ..default()
            },
            Name::new("Close Shot Bullet"),
            CloseShotBullet {
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                damage: 2.0,
                speed: 4.5,
                direction,
            },
            Sensor,
            Collider::cuboid(0.2, 0.2),
        ))
        .id()
}

fn close_shot_bullet(
    mut commands: Commands,
    //Gross but makes font loading easier
    mut bullets: Query<(Entity, &mut Transform, &Collider, &mut CloseShotBullet), Without<Enemy>>,
    rapier_context: Res<RapierContext>,
    mut enemy: Query<(&mut Enemy, &Transform)>,
    time: Res<Time>,
) {
    for (bullet_entity, mut transform, collider, mut bullet) in &mut bullets {
        bullet.lifetime.tick(time.delta());
        if bullet.lifetime.just_finished() {
            commands.entity(bullet_entity).despawn_recursive();
        }

        transform.translation += bullet.direction.extend(0.0) * time.delta_seconds() * bullet.speed;

        rapier_context.intersections_with_shape(
            transform.translation.truncate(),
            0.0,
            collider,
            QueryFilter::new(),
            |entity| {
                if let Ok((mut enemy, _transform)) = enemy.get_mut(entity) {
                    damage_enemy(&mut enemy, bullet.damage);
                    println!("Dealt {} damage to the enemy!", bullet.damage);
                    commands.entity(bullet_entity).despawn_recursive();
                }
                true
            },
        );
    }
}

fn close_shot_attack(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut close_shots: Query<(&GlobalTransform, &mut CloseShot)>,
    enemy: Query<&Transform, With<Enemy>>,
    time: Res<Time>,
) {
    for (transform, mut close_shot) in &mut close_shots {
        close_shot.timer.tick(time.delta());
        if close_shot.timer.just_finished() {
            if let Some(closest_enemy) = enemy.iter().min_by_key(|enemy_transform| {
                FloatOrd(Vec2::length(
                    transform.translation().truncate() - enemy_transform.translation.truncate(),
                ))
            }) {
                let direction = (closest_enemy.translation.truncate()
                    - transform.translation().truncate())
                .normalize();

                spawn_close_shot_bullet(
                    &mut commands,
                    &assets,
                    transform.translation().truncate(),
                    direction,
                );
            }
        }
    }
}
