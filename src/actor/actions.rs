use crate::RandomSource;
use crate::actor::spawns::{SpawnAgent, SpawnPlayer};
use crate::actor::thinking::Decision;
use crate::building::construction::Construction;
use crate::building::resources::{
    CalculateClosestResource, ChangedStorehouseResources, Resourceable, Storehouse,
};
use crate::building::{Building, Guild, Training, TrainingClaimed, TrainingTimer};
use crate::common::Direction;
use crate::save::Saveable;
use crate::sprite_animation::{self, AnimationState};
use crate::world::WorldWithTiles;
use crate::world::goods::{Bar, Coal, Deposit, Lumber, Ore, Plank};
use bevy::ecs::component::Mutable;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::resource::IsResource;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_northstar::components::{AgentPos, NextPos, Pathfind, PathfindingFailed};
use itertools::Itertools;

use crate::actor::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_message::<SpawnPlayer>()
            .add_message::<SpawnAgent>()
            .add_systems(
                Update,
                (
                    spawns::spawn_player_actor,
                    spawns::fix_agents,
                    fix_path_finding,
                ),
            );

        app.add_message::<ActorStateChanged<AttackingState>>();
        app.add_message::<ActorStateChanged<ThinkingState>>();
        app.add_message::<ActorStateChanged<DecidingState>>();
        app.add_message::<ActorStateChanged<DecidedState>>();
        app.add_message::<ActorStateChanged<WanderingState>>();
        app.add_message::<ActorStateChanged<TrainingState>>();
        app.add_message::<ActorStateChanged<ConsumingState>>();
        app.add_message::<ActorStateChanged<HarvestingState<Lumber>>>();
        app.add_message::<ActorStateChanged<HarvestingState<Ore>>>();
        app.add_message::<ActorStateChanged<HarvestingState<Coal>>>();
        app.add_message::<ActorStateChanged<ReturningFromTheHarvestState>>();
        app.add_message::<ActorStateStateCouldBeContinuousState>();
        app.add_message::<ActorNeedsPathFound>();
        app.add_systems(
            Update,
            (
                (fix_state_changes, fix_continuous_action_changes),
                (
                    move_actors,
                    move_actors_without_path_finding,
                    attacking_actors_act,
                    resting_actors_act,
                    wander_with_aim_act,
                    wander_without_aim_act,
                    end_wandering_act,
                    trainees_take_action,
                    harvesting_actors_act::<Lumber>,
                    harvesting_actors_act::<Ore>,
                    harvesting_actors_act::<Coal>,
                    return_from_harvesting_actors_act,
                ),
            )
                .chain(),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(ContinuousActions {
        entities: EntityHashSet::new(),
        _marker: PhantomData::<RestingState>::default(),
    });
    commands.insert_resource(ContinuousActions {
        entities: EntityHashSet::new(),
        _marker: PhantomData::<WalkingState>::default(),
    });
    commands.insert_resource(ContinuousActions {
        entities: EntityHashSet::new(),
        _marker: PhantomData::<WanderingState>::default(),
    });
}

pub fn fix_state_changes(
    time: Res<Time>,
    query: Query<(Entity, &mut ActorState), Without<ActorIsDead>>,
    (
        mut attacking_messages,
        mut deciding_messages,
        mut decided_messages,
        mut consuming_messages,
        mut wandering_messages,
        mut training_messages,
        mut harvesting_lumber_messages,
        mut harvesting_ore_messages,
        mut harvesting_coal_messages,
        mut returning_from_harvest_messages,
    ): (
        MessageWriter<ActorStateChanged<AttackingState>>,
        MessageWriter<ActorStateChanged<DecidingState>>,
        MessageWriter<ActorStateChanged<DecidedState>>,
        MessageWriter<ActorStateChanged<ConsumingState>>,
        MessageWriter<ActorStateChanged<WanderingState>>,
        MessageWriter<ActorStateChanged<TrainingState>>,
        MessageWriter<ActorStateChanged<HarvestingState<Lumber>>>,
        MessageWriter<ActorStateChanged<HarvestingState<Ore>>>,
        MessageWriter<ActorStateChanged<HarvestingState<Coal>>>,
        MessageWriter<ActorStateChanged<ReturningFromTheHarvestState>>,
    ),

    mut walking_actions: ResMut<ContinuousActions<WalkingState>>,
    mut resting_actions: ResMut<ContinuousActions<RestingState>>,
    mut wandering_actions: ResMut<ContinuousActions<WanderingState>>,

    mut constructions: Query<&mut Construction>,
    mut trainings_query: Query<(&Training, &mut TrainingTimer)>,

    mut commands: Commands,
) {
    for (entity, mut state) in query {
        let mut new_state: Option<ActorState> = None;
        match &mut *state {
            ActorState::Idle => {
                new_state = Some(ActorState::Thinking(Timer::from_seconds(
                    1.0,
                    TimerMode::Once,
                )));
                walking_actions.entities.remove(&entity);
                resting_actions.entities.remove(&entity);
                wandering_actions.entities.remove(&entity);

                commands
                    .entity(entity)
                    .remove::<NextPos>()
                    .remove::<Pathfind>();
            }
            ActorState::Thinking(timer) => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    new_state = Some(ActorState::Deciding(Vec::<Decision>::with_capacity(
                        32usize,
                    )));
                    deciding_messages.write(ActorStateChanged(entity, PhantomData::default()));
                }
            }
            ActorState::Deciding(tree) => {
                new_state = Some(ActorState::Decided(tree.clone()));
            }
            ActorState::Decided(_) => {
                decided_messages.write(ActorStateChanged(entity, PhantomData::default()));
            }
            ActorState::Attacking(timer, _) => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    attacking_messages.write(ActorStateChanged(entity, PhantomData::default()));
                }
            }
            ActorState::Harvesting {
                timer,
                resource_kind,
                ..
            } => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    match resource_kind {
                        ResourceKind::Lumber => {
                            harvesting_lumber_messages
                                .write(ActorStateChanged(entity, PhantomData::default()));
                        }
                        ResourceKind::Ore => {
                            harvesting_ore_messages
                                .write(ActorStateChanged(entity, PhantomData::default()));
                        }
                        ResourceKind::Coal => {
                            harvesting_coal_messages
                                .write(ActorStateChanged(entity, PhantomData::default()));
                        }
                        _ => {}
                    }
                }
            }
            ActorState::ReturningFromTheHarvest { timer, .. } => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    returning_from_harvest_messages
                        .write(ActorStateChanged(entity, PhantomData::default()));
                }
            }
            ActorState::Walking(_, _) => {
                // do nothing for walking because we only stop at the target
            }
            ActorState::Dead => {
                info!("Is dead: {:?}", entity);
                panic!("Actor found without the ActorIsDead flag");
            }
            ActorState::Resting => {
                // do nothing for walking because we only stop when entity is fully rested
            }
            ActorState::Wandering(timer, _) => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    wandering_messages.write(ActorStateChanged(entity, PhantomData::default()));
                }
            }
            ActorState::Consuming { timer, .. } => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    consuming_messages.write(ActorStateChanged(entity, PhantomData::default()));
                }
            }
            ActorState::Training(training) => {
                if let Ok((t, mut timer)) = trainings_query.get_mut(*training) {
                    if let Some(trainee) = t.trainee {
                        if trainee == entity {
                            timer.timer.tick(time.delta());
                            if timer.timer.just_finished() {
                                training_messages
                                    .write(ActorStateChanged(entity, PhantomData::default()));
                            }
                        } else {
                            new_state = Some(ActorState::Idle);
                        }
                    } else {
                        new_state = Some(ActorState::Idle);
                    }
                }
            }
            ActorState::Constructing(site) => {
                if let Ok(mut construction) = constructions.get_mut(*site) {
                    if !construction.1.contains(&entity) {
                        construction.1.insert(entity);
                    }
                }
            }
        }
        if let Some(actual_new_state) = new_state {
            *state = actual_new_state;
        }
    }
}

pub fn fix_continuous_action_changes(
    mut messages: MessageReader<ActorStateStateCouldBeContinuousState>,
    entities: Query<(Entity, &ActorState), Without<ActorIsDead>>,
    mut walking_actions: ResMut<ContinuousActions<WalkingState>>,
    mut resting_actions: ResMut<ContinuousActions<RestingState>>,
    mut wandering_actions: ResMut<ContinuousActions<WanderingState>>,
    mut pathers: MessageWriter<ActorNeedsPathFound>,
) {
    for message in messages.read() {
        if let Ok((entity, state)) = entities.get(message.entity) {
            match state {
                ActorState::Resting => {
                    resting_actions.entities.insert(entity);
                }
                ActorState::Walking(_, _) => {
                    walking_actions.entities.insert(entity);
                    pathers.write(ActorNeedsPathFound { entity });
                }
                ActorState::Wandering(_, _) => {
                    wandering_actions.entities.insert(entity);
                }
                _ => {}
            }
        }
    }
}

pub fn fix_path_finding(
    mut messages: MessageReader<ActorNeedsPathFound>,
    mut commands: Commands,
    entities: Query<(Entity, &ActorState), (With<Actor>, Without<Pathfind>)>,
    building_targets: Query<&Transform, (Without<Actor>,)>,
    actor_targets: Query<&Transform, (With<Actor>, Without<PlayerParty>, Without<ActorIsDead>)>,
    world: Res<WorldWithTiles>,
) {
    for message in messages.read() {
        if let Ok((entity, ActorState::Walking(action_target, _))) = entities.get(message.entity) {
            if let Ok(target) = building_targets.get(action_target.entity) {
                if let Some(position) = world.get_position(target) {
                    commands
                        .entity(entity)
                        .remove::<PathfindingFailed>()
                        .insert(Pathfind::new(UVec3::new(position.x, position.y - 1, 0)));
                }
            } else if let Ok(target) = actor_targets.get(action_target.entity) {
                if let Some(position) = world.get_position(target) {
                    commands
                        .entity(entity)
                        .remove::<PathfindingFailed>()
                        .insert(Pathfind::new(UVec3::new(position.x, position.y, 0)));
                }
            }
        } else if let Ok((entity, ActorState::Wandering(_, action_target))) =
            entities.get(message.entity)
        {
            if let Some(action_target) = action_target {
                if let Some(position) = world.get_xy(action_target.target.xy()) {
                    commands
                        .entity(entity)
                        .remove::<PathfindingFailed>()
                        .insert(Pathfind::new(UVec3::new(position.x, position.y, 0)));
                }
            }
        }
    }
}

pub fn move_actors(
    walkers: Res<ContinuousActions<WalkingState>>,
    time: Res<Time>,
    mut attacker_query: Query<
        (
            Entity,
            &mut ActorState,
            &mut AnimationState,
            &mut Transform,
            &mut Direction,
            &mut PersonalNeeds,
            &mut AgentPos,
            &NextPos,
        ),
        (With<Actor>, With<PlayerParty>, Without<ActorIsDead>),
    >,
    defender_query: Query<
        &Transform,
        (
            Or<(Without<PlayerParty>, Without<Actor>)>,
            Without<ActorIsDead>,
        ),
    >,
    world: Res<WorldWithTiles>,
    mut commands: Commands,
    mut can_be_continuous_messages: MessageWriter<ActorStateStateCouldBeContinuousState>,
) {
    for entity in walkers.entities.iter() {
        if let Ok((
            entity,
            mut state,
            mut animation_state,
            mut transform,
            mut direction,
            mut personal_needs,
            mut agent_pos,
            next_pos,
        )) = attacker_query.get_mut(*entity)
        {
            if let ActorState::Walking(target, state_after_found) = &*state {
                if let Ok(target_transform) = defender_query.get(target.entity) {
                    if transform.translation.distance(target_transform.translation) < target.range {
                        *state = *state_after_found.clone();
                        *animation_state = state.get_animation_state();
                        info!("Set to {:?}", state);
                        can_be_continuous_messages
                            .write(ActorStateStateCouldBeContinuousState { entity });
                        continue;
                    }
                    if let Some(next_tile_pos) = TilePos::from_i32_pair(
                        next_pos.0.x as i32,
                        next_pos.0.y as i32,
                        &world.map_size,
                    ) {
                        let next_transform = world.get_transform(next_tile_pos);
                        let speed = 32f64;
                        let delta = time.delta_secs_f64() * speed;
                        let direction_vector = (next_transform - transform.translation).normalize();
                        transform.translation += direction_vector * delta as f32;
                        *direction = Direction::from_vector(direction_vector);
                        personal_needs.current_energy =
                            (personal_needs.current_energy - time.delta().as_secs_f32()).max(0f32);
                        if let Some(current_pos) = world.get_position(&transform) {
                            if next_pos.0.x == current_pos.x && next_pos.0.y == current_pos.y {
                                agent_pos.0 = next_pos.0;
                                commands.entity(entity).remove::<NextPos>();
                            }
                        } else {
                            warn!("Went off the end of the world");
                            *state = ActorState::Idle;
                            *animation_state = AnimationState::Idle;
                        }
                    } else {
                        warn!("Went off the end of the world");
                        *state = ActorState::Idle;
                        *animation_state = AnimationState::Idle;
                    }
                } else {
                    *state = ActorState::Idle;
                    *animation_state = AnimationState::Idle;
                }
            }
        }
    }
}

pub fn move_actors_without_path_finding(
    walkers: Res<ContinuousActions<WalkingState>>,
    time: Res<Time>,
    mut attacker_query: Query<
        (
            Entity,
            &mut ActorState,
            &mut AnimationState,
            &mut Transform,
            &mut Direction,
            &mut PersonalNeeds,
        ),
        (
            With<Actor>,
            Or<(Without<NextPos>, With<PathfindingFailed>)>,
            With<PlayerParty>,
            Without<ActorIsDead>,
        ),
    >,
    defender_query: Query<
        &Transform,
        (
            Or<(Without<PlayerParty>, Without<Actor>)>,
            Without<ActorIsDead>,
        ),
    >,
    mut can_be_continuous_messages: MessageWriter<ActorStateStateCouldBeContinuousState>,
) {
    for entity in walkers.entities.iter() {
        if let Ok((
            entity,
            mut state,
            mut animation_state,
            mut transform,
            mut direction,
            mut personal_needs,
        )) = attacker_query.get_mut(*entity)
        {
            if let ActorState::Walking(target, state_after_found) = &*state {
                if let Ok(target_transform) = defender_query.get(target.entity) {
                    let distance = transform.translation.distance(target_transform.translation);
                    if distance < target.range {
                        *state = *state_after_found.clone();
                        *animation_state = state.get_animation_state();
                        info!("Set to {:?}", state);
                        can_be_continuous_messages
                            .write(ActorStateStateCouldBeContinuousState { entity });
                        continue;
                    }
                    let speed = 32f64;
                    let delta = time.delta_secs_f64() * speed;
                    let direction_vector =
                        (target_transform.translation - transform.translation).normalize();
                    transform.translation += direction_vector * delta as f32;
                    *direction = Direction::from_vector(direction_vector);
                    personal_needs.current_energy =
                        (personal_needs.current_energy - time.delta().as_secs_f32()).max(0f32);
                } else {
                    *state = ActorState::Idle;
                    *animation_state = AnimationState::Idle;
                }
            }
        }
    }
}

pub fn attacking_actors_act(
    mut attack_messages: MessageReader<ActorStateChanged<AttackingState>>,
    mut commands: Commands,
    mut attacker_query: Query<
        (
            Entity,
            &mut ActorState,
            &mut AnimationState,
            &mut Transform,
            &mut PersonalNeeds,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    mut defender_query: Query<
        (&mut ActorState, &mut HealthMana, &Transform),
        (Without<PlayerParty>, Without<ActorIsDead>),
    >,
    mut can_be_continuous_messages: MessageWriter<ActorStateStateCouldBeContinuousState>,
) {
    for message in attack_messages.read() {
        if let Ok((entity, mut state, mut animation_state, mut transform, mut personal_needs)) =
            attacker_query.get_mut(message.0)
        {
            let mut become_idle = false;
            if let ActorState::Attacking(_, attack_target) = &*state {
                if let Ok((mut defender_state, mut health, _)) =
                    defender_query.get_mut(attack_target.entity)
                {
                    health.current_hp = health.current_hp.saturating_sub(10);
                    if health.current_hp == 0 {
                        become_idle = true;
                        *defender_state = ActorState::Dead;
                        commands.entity(attack_target.entity).insert(ActorIsDead);
                        info!("Kill! {:?}", attack_target.entity);
                    } else {
                        *state =
                            ActorState::Walking(attack_target.clone(), Box::new(state.clone()));
                        *animation_state = AnimationState::Walk(0);
                        can_be_continuous_messages
                            .write(ActorStateStateCouldBeContinuousState { entity });
                    }
                    personal_needs.current_energy =
                        (personal_needs.current_energy - 5f32).max(0f32);
                } else {
                    become_idle = true;
                }
            }
            if become_idle {
                *state = ActorState::Idle;
                *animation_state = AnimationState::Idle;
            }
        }
    }
}

pub fn harvesting_actors_act<
    T: Component<Mutability = Mutable>
        + Resourceable
        + Harvestable
        + Saveable
        + Default
        + Send
        + Sync
        + 'static,
>(
    mut harvesting_messages: MessageReader<ActorStateChanged<HarvestingState<T>>>,
    mut commands: Commands,
    mut havester_query: Query<
        (
            Entity,
            &mut ActorState,
            &mut AnimationState,
            &mut Transform,
            &mut Inventory,
            &mut PersonalNeeds,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    mut raws: Query<&mut T>,
    mut deposits: Query<&mut Deposit<T>>,
    mut needs_calc: MessageWriter<CalculateClosestResource>,
    mut can_be_continuous_messages: MessageWriter<ActorStateStateCouldBeContinuousState>,
) {
    for message in harvesting_messages.read() {
        if let Ok((
            entity,
            mut state,
            mut animation_state,
            mut _transform,
            mut inventory,
            mut personal_needs,
        )) = havester_query.get_mut(message.0)
        {
            let mut become_idle = false;
            if let ActorState::Harvesting {
                resource_kind,
                resource_entity,
                storehouse_entity,
                ..
            } = &*state
            {
                let result: Option<&mut dyn Harvestable> =
                    raws.get_mut(*resource_entity).map_or_else(
                        |_| {
                            deposits
                                .get_mut(*resource_entity)
                                .map_or(None, |d| Some(d.into_inner() as &mut dyn Harvestable))
                        },
                        |d| Some(d.into_inner() as &mut dyn Harvestable),
                    );
                if let Some(resource_state) = result {
                    if resource_state.can_be_harvested() {
                        if let Some(resource_harvested) = resource_state.harvest(10) {
                            if resource_state.despawn_when_harvested() {
                                commands.entity(*resource_entity).despawn();
                                needs_calc.write(CalculateClosestResource {
                                    to: *storehouse_entity,
                                    kind: resource_harvested.clone(),
                                });
                            }
                            inventory.add(Item::Good(resource_harvested.clone()));
                            *state = ActorState::Walking(
                                ActionTarget {
                                    entity: *storehouse_entity,
                                    range: 8f32,
                                },
                                Box::new(ActorState::ReturningFromTheHarvest {
                                    kind: resource_harvested,
                                    entity: *storehouse_entity,
                                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                                }),
                            );

                            *animation_state = AnimationState::Walk(0);
                            can_be_continuous_messages
                                .write(ActorStateStateCouldBeContinuousState { entity });
                        }
                        personal_needs.current_energy =
                            (personal_needs.current_energy - 5f32).max(0f32);
                    } else {
                        become_idle = true;
                    }
                } else {
                    become_idle = true;
                }
            }
            if become_idle {
                *state = ActorState::Idle;
                *animation_state = AnimationState::Idle;
            }
        }
    }
}

pub fn return_from_harvesting_actors_act(
    mut harvesting_messages: MessageReader<ActorStateChanged<ReturningFromTheHarvestState>>,
    mut havester_query: Query<
        (
            Entity,
            &mut ActorState,
            &mut AnimationState,
            &mut Transform,
            &mut Inventory,
            &mut PersonalNeeds,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    storehouse_query: Query<&mut Storehouse>,
    mut changed_storehouse_messages: MessageWriter<ChangedStorehouseResources>,
) {
    for message in harvesting_messages.read() {
        if let Ok((
            entity,
            mut state,
            mut animation_state,
            mut _transform,
            mut inventory,
            mut personal_needs,
        )) = havester_query.get_mut(message.0)
        {
            let mut become_idle = false;
            if let ActorState::ReturningFromTheHarvest {
                kind,
                entity: storehouse_entity,
                ..
            } = &*state
            {
                if let Ok(storehouse) = storehouse_query.get(*storehouse_entity) {
                    if storehouse.can_store(kind) {
                        let item = Item::Good(kind.clone());
                        inventory.remove(&item);
                        if !inventory.has(&item) {
                            // otherwise continue emptying pockets
                            become_idle = true;
                        }
                        changed_storehouse_messages.write(ChangedStorehouseResources {
                            kind: kind.clone(),
                            amount: 1,
                            storehouse: *storehouse_entity,
                        });
                        personal_needs.current_energy =
                            (personal_needs.current_energy - 5f32).max(0f32);
                    } else {
                        become_idle = true;
                    }
                } else {
                    become_idle = true;
                }
            }
            if become_idle {
                info!("Become idle");
                *state = ActorState::Idle;
                *animation_state = AnimationState::Idle;
            }
        }
    }
}

pub fn resting_actors_act(
    resters: Res<ContinuousActions<RestingState>>,
    time: Res<Time>,
    _rng: ResMut<RandomSource>,
    mut query: Query<
        (&mut ActorState, &mut AnimationState, &mut PersonalNeeds),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
) {
    for entity in resters.entities.iter() {
        if let Ok((mut state, mut animation_state, mut personal_needs)) = query.get_mut(*entity) {
            if let ActorState::Resting = *state {
                personal_needs.current_energy += time.delta().as_secs_f32() * 20f32;

                if personal_needs.current_energy >= personal_needs.max_energy {
                    *state = ActorState::Idle;
                    *animation_state = AnimationState::Idle;
                    personal_needs.current_energy += personal_needs.max_energy;

                    println!("Rested!");
                } else {
                    /*println!(
                        "Reaping energy, current energy: {}",
                        personal_needs.current_energy
                    );*/
                }
            }
        }
    }
}

fn wander_without_aim_act(
    wanderers: Res<ContinuousActions<WanderingState>>,
    mut messages: MessageWriter<ActorNeedsPathFound>,
    mut query: Query<
        (Entity, &mut ActorState, &mut Transform, &mut AnimationState),
        (
            Or<(With<PathfindingFailed>, Without<NextPos>)>,
            Without<ActorIsDead>,
        ),
    >,
    world: Res<WorldWithTiles>,
    mut rng: ResMut<RandomSource>,
) {
    for wanderer in wanderers.entities.iter() {
        if let Ok((entity, mut state, mut transform, mut animation_state)) =
            query.get_mut(*wanderer)
        {
            if let ActorState::Wandering(_, ref mut target) = *state {
                let random_offset = Vec2::new(
                    rng.0.random_range(-80.0..80.0),
                    rng.0.random_range(-80.0..80.0),
                );
                if let Some(tile_pos) = world.get_xy(random_offset) {
                    if world.is_passable(tile_pos) {
                        *target = Some(WanderingTarget {
                            target: transform.translation + random_offset.extend(0.0),
                        });
                        *animation_state = AnimationState::Walk(0);
                        messages.write(ActorNeedsPathFound { entity });
                    }
                }
            }
        }
    }
}

fn wander_with_aim_act(
    wanderers: Res<ContinuousActions<WanderingState>>,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut ActorState,
            &mut Transform,
            &mut AnimationState,
            &mut Direction,
            &mut PersonalNeeds,
            &mut AgentPos,
            &NextPos,
        ),
        Without<ActorIsDead>,
    >,
    mut commands: Commands,
    world: Res<WorldWithTiles>,
) {
    for wanderer in wanderers.entities.iter() {
        if let Ok((
            entity,
            mut state,
            mut transform,
            mut animation_state,
            mut direction,
            mut personal_needs,
            mut agent_pos,
            next_pos,
        )) = query.get_mut(*wanderer)
        {
            if let ActorState::Wandering(_, ref mut target) = *state {
                if let Some(wandering_target) = target {
                    if transform.translation.distance(wandering_target.target) < 16.0 {
                        *target = None;
                        *animation_state = AnimationState::Walk(0);
                        commands.entity(entity).remove::<(Pathfind, NextPos)>();
                        continue;
                    }
                    if let Some(next_tile_pos) = TilePos::from_i32_pair(
                        next_pos.0.x as i32,
                        next_pos.0.y as i32,
                        &world.map_size,
                    ) {
                        let next_transform = world.get_transform(next_tile_pos);
                        let speed = 32f64;
                        let delta = time.delta_secs_f64() * speed;
                        let direction_vector = (next_transform - transform.translation).normalize();
                        transform.translation += direction_vector * delta as f32;
                        *direction = Direction::from_vector(direction_vector);
                        personal_needs.current_energy =
                            (personal_needs.current_energy - time.delta().as_secs_f32()).max(0f32);
                        if let Some(current_pos) = world.get_position(&transform) {
                            if next_pos.0.x == current_pos.x && next_pos.0.y == current_pos.y {
                                agent_pos.0 = next_pos.0;
                                commands.entity(entity).remove::<NextPos>();
                            }
                        } else {
                            warn!("Went off the end of the world");
                            *state = ActorState::Idle;
                            *animation_state = AnimationState::Idle;
                        }
                    } else {
                        warn!("Went off the end of the world");
                        *state = ActorState::Idle;
                        *animation_state = AnimationState::Idle;
                    }
                }
            }
        }
    }
}

pub fn end_wandering_act(
    mut messages: MessageReader<ActorStateChanged<WanderingState>>,
    mut query: Query<(&mut ActorState, &mut AnimationState)>,
) {
    for wandering_entity in messages.read() {
        if let Ok((mut state, mut animation_state)) = query.get_mut(wandering_entity.0) {
            if let ActorState::Wandering(_, _) = *state {
                *state = ActorState::Idle;
                *animation_state = AnimationState::Idle;
            }
        }
    }
}

pub fn trainees_take_action(
    mut messages: MessageReader<ActorStateChanged<TrainingState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut training_query: Query<(Entity, &Training, &mut TrainingTimer)>,
    mut trainee_query: Query<
        (
            &mut ActorState,
            &mut ActorKind,
            &mut AnimationState,
            &mut Sprite,
        ),
        With<PlayerParty>,
    >,
    mut guild_query: Query<&mut Guild, With<Building>>,
) {
    for message in messages.read() {
        if let Ok((entity, training, _)) = training_query.get_mut(message.0) {
            if let Some(trainee) = training.trainee {
                if let Ok(mut guild) = guild_query.get_mut(training.guild) {
                    if let Ok((mut state, mut kind, mut animation_state, mut sprite)) =
                        trainee_query.get_mut(trainee)
                    {
                        *state = ActorState::Idle;
                        *animation_state = AnimationState::Idle;
                        *sprite = sprite_animation::gen_sprite(
                            training.end_kind.sprite_path(),
                            &asset_server,
                            &mut texture_atlas_layouts,
                        );
                        *kind = training.end_kind.clone();
                        guild.members.push(trainee);
                        println!("Trainee finished training: {:?}", training.end_kind);
                        guild.trainings.retain(|t| t.index() != entity.index());
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
