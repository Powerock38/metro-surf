use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig, TnuaPlatformerControls,
    TnuaPlatformerPlugin, TnuaRapier3dPlugin,
};
use noisy_bevy::simplex_noise_2d_seeded;

const SEED: f32 = 23.0;
const TIME_STEP: f32 = 1.0 / 60.0;

const LANES: i32 = 10;
const MAX_OBSTACLES_PER_LANE: i32 = 20;
const LANES_WIDTH: f32 = 10.0;
const OBSTACLE_LENGTH: f32 = 20.0;
const OBSTACLE_MAX_HEIGHT: f32 = 10.0;
const OBSTACLE_MIN_HEIGHT: f32 = 5.0;

const DEFAULT_POSITION: Vec3 = Vec3::ZERO;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(TnuaRapier3dPlugin)
        .add_plugin(TnuaPlatformerPlugin)
        .add_startup_system(setup_terrain)
        .add_startup_system(setup_player)
        .add_system(player_movement)
        .add_system(check_wall_crashes)
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .run();
}

#[derive(Component)]
struct Player {
    target_lane: Option<i32>,
    move_cooldown: Timer,
}

fn map_range(from_range: (f32, f32), to_range: (f32, f32), s: f32) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain_width = LANES_WIDTH * LANES as f32;
    let terrain_length = OBSTACLE_LENGTH * MAX_OBSTACLES_PER_LANE as f32;

    // terrain
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -terrain_width / 2.0,
                max_x: terrain_width / 2.0,
                min_y: 0.0,
                max_y: 0.0,
                min_z: -terrain_length,
                max_z: 0.0,
            })),
            material: materials.add(Color::WHITE.into()),
            ..default()
        })
        .insert(Collider::halfspace(Vec3::Y).unwrap())
        .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_1));

    // obstacles
    for i in 0..LANES {
        for j in 0..MAX_OBSTACLES_PER_LANE {
            let noise = simplex_noise_2d_seeded(Vec2::new(i as f32, j as f32), SEED);

            let height = map_range((-1.0, 1.0), (0.0, OBSTACLE_MAX_HEIGHT), noise);

            if height >= OBSTACLE_MIN_HEIGHT {
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(
                            LANES_WIDTH,
                            height,
                            OBSTACLE_LENGTH,
                        ))),
                        material: materials.add(
                            Color::hsl(map_range((-1.0, 1.0), (0.0, 360.0), noise), 1.0, 0.1)
                                .into(),
                        ),
                        transform: Transform::from_xyz(
                            (i as f32 * LANES_WIDTH) - (terrain_width / 2.0),
                            height * 0.5,
                            -j as f32 * OBSTACLE_LENGTH - (OBSTACLE_LENGTH / 2.0),
                        ),
                        ..default()
                    })
                    .insert(Collider::cuboid(
                        0.5 * LANES_WIDTH,
                        0.5 * height,
                        0.5 * OBSTACLE_LENGTH,
                    ))
                    .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_1));
            }
        }
    }

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_scene = asset_server.load("alien.glb#Scene0");

    // player
    commands
        .spawn((
            SceneBundle {
                scene: player_scene,
                transform: Transform::from_translation(DEFAULT_POSITION)
                    .with_scale(Vec3::splat(10.0)),
                ..default()
            },
            Player {
                target_lane: None,
                move_cooldown: Timer::from_seconds(0.2, TimerMode::Once),
            },
        ))
        .insert(RigidBody::Dynamic)
        .with_children(|c| {
            c.spawn(Collider::capsule_y(0.3, 0.2))
                .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2))
                .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
                .insert(TransformBundle {
                    local: Transform::from_xyz(0.0, 4.0, 0.0),
                    ..default()
                });
        })
        .insert(TnuaPlatformerBundle::new_with_config(
            TnuaPlatformerConfig {
                up: Vec3::Y,
                forward: -Vec3::Z,
                full_speed: 50.0,
                full_jump_height: 50.0,
                float_height: 1.0,
                cling_distance: 0.5,
                spring_strengh: 400.0,
                spring_dampening: 1.2,
                acceleration: 100.0,
                air_acceleration: 60.0,
                coyote_time: 0.15,
                jump_start_extra_gravity: 10.0,
                jump_fall_extra_gravity: 30.0,
                jump_shorten_extra_gravity: 40.0,
                free_fall_behavior: TnuaFreeFallBehavior::LikeJumpShorten,
                tilt_offset_angvel: 0.0,
                tilt_offset_angacl: 0.0,
                turning_angvel: 10.0,
            },
        ))
        .insert(Velocity::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        // .insert(TnuaRapier3dSensorShape(Collider::capsule_y(0.2, 0.1)))
        // .insert(TnuaPlatformerAnimatingOutput::default())
        .with_children(|c| {
            c.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 3.0, 6.0).looking_at(Vec3::Y, Vec3::Y),
                // transform: Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        });
}

fn player_movement(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut TnuaPlatformerControls, &mut Player, &mut Transform)>,
) {
    let (mut controls, mut player, mut player_transform) = query.single_mut();

    let mut direction = -Vec3::Z;

    let mut moved = false;

    let mut jumped = false;

    if player.move_cooldown.tick(time.delta()).finished() {
        if keyboard.pressed(KeyCode::Up) {
            direction -= Vec3::Z;
        }
        if keyboard.pressed(KeyCode::Down) {
            direction += Vec3::Z;
        }

        let current_lane_f32 = player_transform.translation.x / LANES_WIDTH;
        let current_lane = current_lane_f32.round() as i32;

        let relative_pos_in_lane =
            current_lane_f32 - (player_transform.translation.x / LANES_WIDTH).round();

        if keyboard.pressed(KeyCode::Left) {
            if relative_pos_in_lane > 0.0 && controls.desired_velocity.x > 0.0 {
                player.target_lane = Some(current_lane);
            } else {
                player.target_lane = Some(current_lane - 1);
            }
            moved = true;
        }
        if keyboard.pressed(KeyCode::Right) {
            if relative_pos_in_lane < 0.0 && controls.desired_velocity.x < 0.0 {
                player.target_lane = Some(current_lane);
            } else {
                player.target_lane = Some(current_lane + 1);
            }
            moved = true;
        }

        jumped = keyboard.pressed(KeyCode::Space);
        moved = moved || jumped;
    }

    if let Some(target_lane) = player.target_lane {
        let target_x = target_lane as f32 * LANES_WIDTH;

        if (target_x - player_transform.translation.x).abs() < 0.1 {
            player.target_lane = None;
            player_transform.translation.x = target_lane as f32 * LANES_WIDTH;
        } else {
            direction += Vec3::X * ((target_x - player_transform.translation.x) / LANES_WIDTH);
        }
    }

    *controls = TnuaPlatformerControls {
        desired_velocity: direction,
        desired_forward: -Vec3::Z,
        jump: jumped.then(|| 1.0),
    };

    if moved {
        player.move_cooldown.reset();
    }
}

fn check_wall_crashes(
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for contact_force_event in contact_force_events.iter() {
        // purposely no checks on -Z
        if contact_force_event.max_force_direction.x.abs() > 0.5
            || contact_force_event.max_force_direction.z > 0.5
        {
            println!("GAME OVER");
            let (mut player, mut player_transform) = query.single_mut();

            player.target_lane = None;
            player_transform.translation = DEFAULT_POSITION;

            break;
        }
    }
}
