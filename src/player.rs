use crate::{DEFAULT_POSITION, LANES_WIDTH, MOVE_COOLDOWN};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use bevy_tnua::TnuaPlatformerAnimatingOutput;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig, TnuaPlatformerControls,
};

#[derive(Component)]
pub struct Player {
    target_lane: Option<i32>,
    move_cooldown: Timer,
}

#[derive(Component)]
pub struct GltfSceneHandler {
    names_from: Handle<Gltf>,
}

#[derive(Component)]
pub struct AnimationsHandler {
    pub entity: Entity,
    pub animations: HashMap<String, Handle<AnimationClip>>,
    pub player_state: PlayerStateForAnimation,
}

pub struct PlayerStateForAnimation {
    pub state: AnimationState,
    pub running_velocity: Vec3,
    pub jumping_velocity: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum AnimationState {
    Running,
    Jumping,
    Falling,
}

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let glb_filename = "player.glb";
    let player_scene = asset_server.load(format!("{}#Scene0", glb_filename).as_str());

    // player
    commands
        .spawn(SceneBundle {
            scene: player_scene,
            transform: Transform::from_translation(DEFAULT_POSITION).with_scale(Vec3::splat(3.0)),
            ..default()
        })
        .insert(Player {
            target_lane: None,
            move_cooldown: Timer::from_seconds(MOVE_COOLDOWN, TimerMode::Once),
        })
        .insert(GltfSceneHandler {
            names_from: asset_server.load(glb_filename),
        })
        .insert(RigidBody::Dynamic)
        .with_children(|c| {
            c.spawn(Collider::capsule_y(0.7, 0.2))
                .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2))
                .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
                .insert(TransformBundle {
                    local: Transform::from_xyz(0.0, 3.0, 0.0),
                    ..default()
                });
        })
        .insert(TnuaPlatformerBundle {
            config: TnuaPlatformerConfig {
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
                jump_input_buffer_time: MOVE_COOLDOWN,
                held_jump_cooldown: None,
                jump_peak_prevention_at_upward_velocity: 0.0,
                jump_peak_prevention_extra_gravity: 0.0,
                height_change_impulse_for_duration: 0.0,
                height_change_impulse_limit: 0.0,
            },
            ..default()
        })
        .insert(Velocity::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        // .insert(TnuaRapier3dSensorShape(Collider::capsule_y(0.2, 0.1)))
        // .insert(TnuaAnimatingState::<AnimationState>::default())
        .insert(TnuaPlatformerAnimatingOutput::default())
        .with_children(|c| {
            c.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 3.0, 6.0).looking_at(Vec3::Y, Vec3::Y),
                // transform: Transform::from_xyz(6.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    let mut crouched = false;

    if player.move_cooldown.tick(time.delta()).finished() {
        if keyboard.any_pressed([KeyCode::Down, KeyCode::S]) {
            direction += Vec3::Z * 0.3;
        }

        let current_lane_f32 = player_transform.translation.x / LANES_WIDTH;
        let current_lane = current_lane_f32.round() as i32;

        let relative_pos_in_lane =
            current_lane_f32 - (player_transform.translation.x / LANES_WIDTH).round();

        if keyboard.any_pressed([KeyCode::Left, KeyCode::Q]) {
            if relative_pos_in_lane > 0.0 && controls.desired_velocity.x > 0.0 {
                player.target_lane = Some(current_lane);
            } else {
                player.target_lane = Some(current_lane - 1);
            }
            moved = true;
        }
        if keyboard.any_pressed([KeyCode::Right, KeyCode::D]) {
            if relative_pos_in_lane < 0.0 && controls.desired_velocity.x < 0.0 {
                player.target_lane = Some(current_lane);
            } else {
                player.target_lane = Some(current_lane + 1);
            }
            moved = true;
        }

        jumped = keyboard.pressed(KeyCode::Space);

        crouched = keyboard.pressed(KeyCode::LShift);

        moved = moved || jumped || crouched;
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
        float_height_offset: if crouched { -10.0 } else { 0.0 },
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

pub fn animation_patcher_system(
    animation_players_query: Query<Entity, Added<AnimationPlayer>>,
    parents_query: Query<&Parent>,
    scene_handlers_query: Query<&GltfSceneHandler>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for player_entity in animation_players_query.iter() {
        let mut entity = player_entity;
        loop {
            if let Ok(GltfSceneHandler { names_from }) = scene_handlers_query.get(entity) {
                let gltf = gltf_assets.get(names_from).unwrap();
                let mut cmd = commands.entity(entity);
                cmd.remove::<GltfSceneHandler>();
                cmd.insert(AnimationsHandler {
                    entity: player_entity,
                    animations: gltf.named_animations.clone(),
                    player_state: PlayerStateForAnimation {
                        state: AnimationState::Running,
                        running_velocity: Vec3::ZERO,
                        jumping_velocity: None,
                    },
                });
                break;
            }
            entity = if let Ok(parent) = parents_query.get(entity) {
                **parent
            } else {
                break;
            };
        }
    }
}

pub fn player_animate(
    mut animations_handlers_query: Query<(&TnuaPlatformerAnimatingOutput, &mut AnimationsHandler)>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
) {
    for (animating_output, mut handler) in animations_handlers_query.iter_mut() {
        let Ok(mut player) = animation_players_query.get_mut(handler.entity) else { continue };

        let new_state = if let Some(upward_velocity) = animating_output.jumping_velocity {
            if upward_velocity > 0.0 {
                AnimationState::Jumping
            } else {
                AnimationState::Falling
            }
        } else {
            AnimationState::Running
        };

        match (handler.player_state.state, new_state) {
            (AnimationState::Falling, AnimationState::Running) => {
                player
                    .play(handler.animations["ROLL"].clone_weak())
                    .set_speed(2.5);
            }
            (_, AnimationState::Running) => {
                player
                    .play_with_transition(
                        handler.animations["RUN"].clone_weak(),
                        std::time::Duration::from_secs_f32(2.0),
                    )
                    .set_speed(2.0)
                    .repeat();
            }
            (_, AnimationState::Jumping | AnimationState::Falling) => {
                player
                    .play(handler.animations["FLIP"].clone_weak())
                    .set_speed(2.5);
            }
        }

        handler.player_state = PlayerStateForAnimation {
            state: new_state,
            running_velocity: animating_output.running_velocity,
            jumping_velocity: animating_output.jumping_velocity,
        };
    }
}
