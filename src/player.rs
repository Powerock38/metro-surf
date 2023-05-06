use crate::{DEFAULT_POSITION, LANES_WIDTH};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig, TnuaPlatformerControls,
};

#[derive(Component)]
pub struct Player {
    target_lane: Option<i32>,
    move_cooldown: Timer,
}

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                full_jump_height: 200.0,
                float_height: 2.0,
                cling_distance: 1.0,
                spring_strengh: 400.0,
                spring_dampening: 1.2,
                acceleration: 1000.0,
                air_acceleration: 1000.0,
                coyote_time: 0.15,
                jump_start_extra_gravity: 0.0,
                jump_fall_extra_gravity: 50.0,
                jump_shorten_extra_gravity: 100.0,
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

pub fn player_movement(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut TnuaPlatformerControls, &mut Player, &mut Transform)>,
) {
    let (mut controls, mut player, mut player_transform) = query.single_mut();

    let mut direction = -Vec3::Z;

    let mut moved = false;

    let mut jumped = false;

    if player.move_cooldown.tick(time.delta()).finished() {
        if keyboard.pressed(KeyCode::Down) {
            direction += Vec3::Z * 0.3;
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

pub fn check_wall_crashes(
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for contact_force_event in contact_force_events.iter() {
        // purposely no checks on -Z
        let x = contact_force_event.max_force_direction.x.abs();
        let z = contact_force_event.max_force_direction.z;
        if x > 0.5 || z > 0.5 {
            println!("GAME OVER");
            let (mut player, mut player_transform) = query.single_mut();

            player.target_lane = None;
            player_transform.translation = DEFAULT_POSITION;

            break;
        }
    }
}
