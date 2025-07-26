use bevy::prelude::*;

use crate::common::Direction;
use crate::sprite_animation::{self, AnimationState};

#[derive(Component)]
pub struct Actor;

#[derive(Component)]
pub enum ActorState {
    Dead,
    Idle,
    Walking,
    Attacking(Timer),
    Thinking(Timer),
}

#[derive(Component)]
pub struct ActorIsDead;

#[derive(Component)]
pub struct Attack {
    pub damage: u32,
}
#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}
#[derive(Component)]
pub struct AttackTimer {
    pub timer: Timer,
}
#[derive(Component)]
pub struct AttackTarget {
    pub target: Entity,
}

#[derive(Component)]
pub struct PlayerParty;

pub fn spawn_player_actor(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    transform: Transform,
) {
    let sprite = sprite_animation::gen_sprite(
        "tinyworld/Characters/Soldiers/Melee/PurpleMelee/SwordsmanPurple.png",
        asset_server,
        texture_atlas_layouts,
    );
    commands.spawn((
        Actor,
        PlayerParty,
        ActorState::Idle,
        Health {
            current: 100,
            max: 100,
        },
        transform,
        sprite,
        sprite_animation::AnimationState::Idle,
        Direction::South,
    ));
}

pub fn spawn_enemy_actor(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    transform: Transform,
) {
    let sprite = sprite_animation::gen_sprite(
        "tinyworld/Characters/Soldiers/Melee/RedMelee/SwordsmanRed.png",
        asset_server,
        texture_atlas_layouts,
    );
    commands.spawn((
        Actor,
        ActorState::Idle,
        sprite_animation::AnimationState::Idle,
        Health {
            current: 100,
            max: 100,
        },
        transform,
        sprite,
        Direction::South,
    ));
}

pub fn fix_idlers(mut query: Query<&mut ActorState>) {
    for mut state in query.iter_mut() {
        if let ActorState::Idle = *state {
            *state = ActorState::Thinking(Timer::from_seconds(1.0, TimerMode::Once));
        }
    }
}

pub fn fix_thinkers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut ActorState,
            &mut sprite_animation::AnimationState,
            &Transform,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    enemy_query: Query<
        (Entity, &ActorState, &Transform),
        (Without<PlayerParty>, Without<ActorIsDead>),
    >,
) {
    for (entity, mut state, mut animation_state, transform) in query.iter_mut() {
        if let ActorState::Thinking(ref mut timer) = *state {
            timer.tick(time.delta());
            if !timer.finished() {
                continue;
            }
            let mut closest_distance = f32::MAX;
            let mut closest_enemy = None;

            for (enemy_entity, _, enemy_transform) in enemy_query.iter() {
                let distance = transform.translation.distance(enemy_transform.translation);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_enemy = Some(enemy_entity);
                }
            }
            if let Some(enemy) = closest_enemy {
                *state = ActorState::Walking;
                *animation_state = sprite_animation::AnimationState::Walk(0);
                commands
                    .entity(entity)
                    .insert(AttackTarget { target: enemy });
            } else {
                *state = ActorState::Idle;
            }
        }
    }
}

pub fn move_attacking_actors(
    time: Res<Time>,
    mut attacker_query: Query<
        (
            &mut ActorState,
            &mut AnimationState,
            &AttackTarget,
            &mut Transform,
            &mut Direction,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    defender_query: Query<(&ActorState, &Transform), (Without<PlayerParty>, Without<ActorIsDead>)>,
) {
    for (mut state, mut animation_state, attack_target, mut transform, mut direction) in
        attacker_query.iter_mut()
    {
        if let ActorState::Walking = *state {
            let speed = 32f64;
            let delta = time.delta_secs_f64() * speed;
            if let Ok((_, target_transform)) = defender_query.get(attack_target.target) {
                if transform.translation.distance(target_transform.translation) < 16.0 {
                    *state = ActorState::Attacking(Timer::from_seconds(1.0, TimerMode::Once));
                    *animation_state =
                        AnimationState::Attack(Timer::from_seconds(0.1, TimerMode::Once));
                    continue;
                }
                let direction_vector =
                    (target_transform.translation - transform.translation).normalize();
                transform.translation += direction_vector * delta as f32;
                *direction = Direction::from_vector(direction_vector);
            } else {
                *state = ActorState::Idle;
            }
        }
    }
}

pub fn attacking_actors_act(
    time: Res<Time>,
    mut commands: Commands,
    mut attacker_query: Query<
        (Entity, &mut ActorState, &AttackTarget, &mut Transform),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    mut defender_query: Query<
        (&mut ActorState, &mut Health, &Transform),
        (Without<PlayerParty>, Without<ActorIsDead>),
    >,
) {
    for (entity, mut state, attack_target, mut transform) in attacker_query.iter_mut() {
        if let ActorState::Attacking(ref mut timer) = *state {
            if let Ok((mut defender_state, mut health, target_transform)) =
                defender_query.get_mut(attack_target.target)
            {
                timer.tick(time.delta());
                if timer.finished() {
                    health.current = health.current.saturating_sub(10);
                    if health.current == 0 {
                        *state = ActorState::Idle;
                        commands.entity(entity).remove::<AttackTarget>();
                        *defender_state = ActorState::Dead;
                        commands.entity(attack_target.target).insert(ActorIsDead);
                    } else {
                        *state = ActorState::Walking;
                    }
                }
            } else {
                *state = ActorState::Idle;
            }
        }
    }
}
