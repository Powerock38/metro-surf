use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig, TnuaPlatformerControls,
    TnuaPlatformerPlugin, TnuaRapier3dPlugin,
};
use noisy_bevy::simplex_noise_2d_seeded;

const TIME_STEP: f32 = 1.0 / 60.0;
// const LANES: i32 = 3;

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
        .add_system(player_input)
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .run();
}

#[derive(Component)]
struct Player {}

fn map_range(from_range: (f32, f32), to_range: (f32, f32), s: f32) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain_width = 50.0;
    let terrain_length = 200.0;

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
        .insert(Collider::halfspace(Vec3::Y).unwrap());

    // obstacles

    let seed = 23.0;

    // let obstacles_material = materials.add(Color::GRAY.into());

    let grid_width = 10.0;
    let grid_length = 20.0;
    let obstacle_max_height = 20.0;
    let obstacle_min_height = 5.0;

    let terrain_grid_width = ((terrain_width / grid_width) as f32).floor() as i32;
    let terrain_grid_length = ((terrain_length / grid_length) as f32).floor() as i32;

    for i in 0..terrain_grid_width {
        for j in 0..terrain_grid_length {
            let noise = simplex_noise_2d_seeded(Vec2::new(i as f32, j as f32), seed);

            let height = map_range((-1.0, 1.0), (0.0, obstacle_max_height), noise);

            if height >= obstacle_min_height {
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(
                            grid_width,
                            height,
                            grid_length,
                        ))),
                        material: materials.add(
                            Color::hsl(map_range((-1.0, 1.0), (0.0, 360.0), noise), 1.0, 0.1)
                                .into(),
                        ),
                        transform: Transform::from_xyz(
                            (i as f32 * grid_width)
                                - (terrain_width / 4.0)
                                - (grid_width / 2.0)
                                - (grid_width / 4.0),
                            height * 0.5,
                            -j as f32 * grid_length - (grid_length / 2.0),
                        ),
                        ..default()
                    })
                    .insert(Collider::cuboid(
                        0.5 * grid_width,
                        0.5 * height,
                        0.5 * grid_length,
                    ));
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
    let default_position = Vec3::new(0.0, 20.0, 0.0);

    // player
    commands
        .spawn((
            SceneBundle {
                scene: player_scene,
                transform: Transform::from_translation(default_position)
                    .with_scale(Vec3::splat(10.0)),
                ..default()
            },
            Player {},
        ))
        .insert(RigidBody::Dynamic)
        .with_children(|c| {
            c.spawn(Collider::capsule_y(0.3, 0.2))
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
                full_jump_height: 20.0,
                float_height: 1.0,
                cling_distance: 0.5,
                spring_strengh: 400.0,
                spring_dampening: 1.2,
                acceleration: 100.0,
                air_acceleration: 60.0,
                coyote_time: 0.15,
                jump_start_extra_gravity: 30.0,
                jump_fall_extra_gravity: 20.0,
                jump_shorten_extra_gravity: 40.0,
                free_fall_behavior: TnuaFreeFallBehavior::LikeJumpShorten,
                tilt_offset_angvel: 5.0,
                tilt_offset_angacl: 500.0,
                turning_angvel: 10.0,
            },
        ))
        .insert(Velocity::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        // .insert(TnuaRapier3dSensorShape(Collider::cylinder(0.0, 0.5)))
        // .insert(TnuaPlatformerAnimatingOutput::default())
        .with_children(|c| {
            c.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 3.0, 6.0).looking_at(Vec3::Y, Vec3::Y),
                // transform: Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        });
}

fn player_input(keyboard: Res<Input<KeyCode>>, mut query: Query<&mut TnuaPlatformerControls>) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::Up) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::Down) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::Left) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::Right) {
        direction += Vec3::X;
    }

    direction = direction.clamp_length_max(1.0);

    let jump = keyboard.pressed(KeyCode::Space);

    for mut controls in query.iter_mut() {
        *controls = TnuaPlatformerControls {
            desired_velocity: direction,
            desired_forward: -Vec3::Z,
            jump: jump.then(|| 1.0),
        };
    }
}
