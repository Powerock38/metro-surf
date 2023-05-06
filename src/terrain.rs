use crate::{
    LANES, LANES_WIDTH, MAX_OBSTACLES_PER_LANE, MAX_OBSTACLE_STACK, OBSTACLE_HEIGHT,
    OBSTACLE_LENGTH, SEED,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use noisy_bevy::simplex_noise_2d_seeded;
use std::f32::consts::PI;

fn map_range(from_range: (f32, f32), to_range: (f32, f32), s: f32) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub fn setup_terrain(
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

            if noise > 0.0 {
                let height = map_range((0.0, 1.0), (1.0, MAX_OBSTACLE_STACK as f32), noise).round()
                    * OBSTACLE_HEIGHT;

                let mut transform = Transform::from_xyz(
                    (i as f32 * LANES_WIDTH) - (terrain_width / 2.0),
                    height * 0.5,
                    -j as f32 * OBSTACLE_LENGTH - (OBSTACLE_LENGTH / 2.0),
                );

                if noise < 0.5 {
                    transform = transform.with_rotation(Quat::from_rotation_x(0.3));
                }

                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(
                            LANES_WIDTH,
                            height,
                            OBSTACLE_LENGTH,
                        ))),
                        material: materials.add(
                            Color::hsl(map_range((0.0, 1.0), (0.0, 360.0), noise), 1.0, 0.1).into(),
                        ),
                        transform,
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
