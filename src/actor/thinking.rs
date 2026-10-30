use crate::building::construction::Construction;
use crate::building::resources::{
    CalculateClosestResource, Harvestable, ResourceKind, Resourceable, Storehouse,
    StorehousesResource,
};
use crate::building::{Building, Guild, Training, TrainingClaimed};
use crate::save::Saveable;
use crate::sprite_animation::{self};
use crate::world::WorldWithTiles;
use crate::world::goods::{Bar, Coal, Deposit, Lumber, Ore, Plank};
use bevy::ecs::event::Trigger;
use bevy_northstar::OrdinalGrid;
use itertools::Itertools;

use crate::{RandomSource, actor::*};

pub struct ThinkingPlugin;
impl Plugin for ThinkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TrainingClaimed>().add_systems(
            Update,
            (
                think_about_attacking_enemy,
                think_about_training,
                think_about_fulfilling_need,
                think_about_constructing_building,
                think_about_lumberjacking::<Lumber>,
                think_about_lumberjacking::<Ore>,
                think_about_lumberjacking::<Coal>,
                actor_decide,
            ),
        );
    }
}

#[derive(Debug, Clone, Saveable)]
pub enum Decision {
    ClaimAndMoveToTarget(ActionTarget, Box<ActorState>),
    MoveToTarget(ActionTarget, Box<ActorState>),
    Act(ActorState),
}

pub fn think_about_attacking_enemy(
    mut messages: MessageReader<ActorStateChanged<DecidingState>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &ActorStatBlock,
            &ActorKind,
            &mut ActorState,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    enemy_query: Query<
        (Entity, &ActorState, &ActorStatBlock, &Transform),
        (Without<PlayerParty>, Without<ActorIsDead>),
    >,
) {
    for entity1 in messages.read() {
        if let Ok((entity, transform, stat_block, kind, mut state)) = query.get_mut(entity1.0) {
            if let ActorState::Deciding(tree) = &mut *state {
                let most_interesting_enemy = enemy_query
                    .iter()
                    .map(|(enemy_entity, _, enemy_stat_block, enemy_transform)| {
                        let distance = transform.translation.distance(enemy_transform.translation);
                        ActorJudgement {
                            entity: enemy_entity,
                            transform: enemy_transform,
                            stat_block: enemy_stat_block,
                            distance: distance.clone(),
                            judgement: kind.judge(enemy_stat_block, stat_block, distance),
                        }
                    })
                    .sorted_by(|a, b| a.judgement.partial_cmp(&b.judgement).unwrap())
                    .next();
                if let Some(judgement) = most_interesting_enemy {
                    if judgement.judgement < 1f32 && judgement.judgement > -1f32 {
                        let target = ActionTarget {
                            entity: judgement.entity,
                            range: 16f32,
                        };
                        tree.push(Decision::MoveToTarget(
                            target.clone(),
                            Box::new(ActorState::Attacking(
                                Timer::from_seconds(0.1, TimerMode::Repeating),
                                target.clone(),
                            )),
                        ));
                        info!(
                            "Actor {:?} decided to attack enemy {:?} (judgement: {})",
                            entity, judgement.entity, judgement.judgement
                        );
                    }
                }
            }
        }
    }
}

pub fn think_about_training(
    mut messages: MessageReader<ActorStateChanged<DecidingState>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &ActorStatBlock,
            &ActorKind,
            &mut ActorState,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    mut trainings_query: Query<(Entity, &mut Training)>,
    guids_query: Query<(&Transform, &Guild), With<Building>>,
) {
    for entity1 in messages.read() {
        if let Ok((entity, transform, stat_block, kind, mut state)) = query.get_mut(entity1.0) {
            if let ActorState::Deciding(tree) = &mut *state {
                for (training_entity, mut training) in trainings_query.iter_mut() {
                    if training.trainee.is_none() && training.required_kind == *kind {
                        if let Ok((guild_transform, _guild)) = guids_query.get(training.guild) {
                            if transform.translation.distance(guild_transform.translation) < 160.0 {
                                tree.push(Decision::ClaimAndMoveToTarget(
                                    ActionTarget {
                                        entity: training.guild,
                                        range: 8f32,
                                    },
                                    Box::new(ActorState::Training(training_entity)),
                                ));
                                training.trainee = Some(entity);
                                info!("Actor {:?} is now training {:?}", entity, training_entity);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn think_about_constructing_building(
    mut messages: MessageReader<ActorStateChanged<DecidingState>>,
    mut query: Query<
        (&Transform, &ActorStatBlock, &ActorKind, &mut ActorState),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    mut construction_query: Query<(Entity, &Transform, &Construction)>,
) {
    for entity1 in messages.read() {
        if let Ok((transform, _stat_block, _kind, mut state)) = query.get_mut(entity1.0) {
            if let ActorState::Deciding(tree) = &mut *state {
                let mut closest_construction_site = None;
                let mut closest_distance = f32::MAX;
                for (construction_entity, construction_transform, _construction) in
                    construction_query.iter_mut()
                {
                    let distance = transform
                        .translation
                        .distance(construction_transform.translation);
                    if distance < 160.0 && distance < closest_distance {
                        closest_construction_site = Some(construction_entity);
                        closest_distance = distance;
                    }
                }
                if let Some(closest_construction_site) = closest_construction_site {
                    tree.push(Decision::MoveToTarget(
                        ActionTarget {
                            entity: closest_construction_site,
                            range: 8f32,
                        },
                        Box::new(ActorState::Constructing(closest_construction_site)),
                    ));
                }
            }
        }
    }
}

pub fn think_about_fulfilling_need(
    mut messages: MessageReader<ActorStateChanged<DecidingState>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &ActorStatBlock,
            &PersonalNeeds,
            &Inventory,
            &ActorKind,
            &ActorHome,
            &mut ActorState,
        ),
        (With<PlayerParty>, Without<ActorIsDead>, Without<Building>),
    >,
    _need_source_query: Query<(Entity, &Transform), Or<(With<ActorIsDead>, With<Building>)>>,
    home_query: Query<(&Transform, &Building)>,
) {
    for entity1 in messages.read() {
        if let Ok((
            entity,
            transform,
            _stat_block,
            personal_needs,
            _inventory,
            _kind,
            actor_home,
            mut state,
        )) = query.get_mut(entity1.0)
        {
            if let ActorState::Deciding(tree) = &mut *state {
                let mut home_is_close = false;
                if let Some(home) = actor_home.home {
                    if let Ok((home_transform, _)) = home_query.get(home) {
                        let distance_to_home =
                            transform.translation.distance(home_transform.translation);
                        if distance_to_home < 200.0 {
                            home_is_close = true;
                        }
                    }
                }

                let mut found = false;
                'outer_need: for need in personal_needs.iter() {
                    if actor_home.home.is_some() {
                        tree.push(Decision::MoveToTarget(
                            ActionTarget {
                                entity: actor_home.home.unwrap(),
                                range: 8f32,
                            },
                            Box::new(ActorState::Resting),
                        ));
                        found = true;
                        break 'outer_need;
                    }
                }

                if !found {
                    tree.push(Decision::Act(ActorState::Wandering(
                        Timer::from_seconds(15.0, TimerMode::Once),
                        None,
                    )));
                }
            }
        }
    }
}

pub fn think_about_lumberjacking<T: Resourceable + Harvestable + Saveable + Component + Default>(
    mut messages: MessageReader<ActorStateChanged<DecidingState>>,
    mut query: Query<
        (Entity, &Transform, &ActorKind, &mut ActorState),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    storehouses: Query<(&Transform, &Storehouse)>,
    storehouses_resource: Res<StorehousesResource>,
    raw: Query<(&Transform, &T)>,
    deposit: Query<(&Transform, &Deposit<T>)>,
    // workshop for planks
    // smelter for bars
    mut needs_calc: MessageWriter<CalculateClosestResource>,
) {
    for entity1 in messages.read() {
        if let Ok((entity, transform, kind, mut state)) = query.get_mut(entity1.0) {
            if let ActorState::Deciding(tree) = &mut *state {
                let mut closest_resource: Option<(ResourceKind, Entity, Entity, &Transform, f32)> =
                    None;
                let kind = T::kind();
                if let Some(storehouse_entity) =
                    storehouses_resource.could_hold(&kind).iter().next()
                {
                    if let Ok((transform, storehouse)) = storehouses.get(*storehouse_entity) {
                        if let Some(resource_entity) = storehouse.closest(&kind) {
                            let result: Option<(&Transform, &dyn Harvestable)> =
                                raw.get(resource_entity).map_or_else(
                                    |_| {
                                        deposit.get(resource_entity).map_or(
                                            None,
                                            |(transform, d)| {
                                                Some((transform, d as &dyn Harvestable))
                                            },
                                        )
                                    },
                                    |(transform, d)| Some((transform, d as &dyn Harvestable)),
                                );
                            if let Some((resource_transform, harvestable)) = result {
                                if !harvestable.can_be_harvested() {
                                    needs_calc.write(CalculateClosestResource {
                                        to: storehouse_entity.clone(),
                                        kind: kind.clone(),
                                    });
                                    continue;
                                }

                                let distance = transform
                                    .translation
                                    .distance(resource_transform.translation);
                                match closest_resource {
                                    Some((_, _, _, _, closest_distance))
                                        if closest_distance < distance =>
                                    {
                                        closest_resource = Some((
                                            kind.clone(),
                                            resource_entity,
                                            *storehouse_entity,
                                            resource_transform,
                                            distance,
                                        ));
                                    }
                                    None => {
                                        closest_resource = Some((
                                            kind.clone(),
                                            resource_entity,
                                            *storehouse_entity,
                                            resource_transform,
                                            distance,
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    if let Some((kind, resource_entity, storehouse_entity, _, _)) = closest_resource
                    {
                        tree.push(Decision::MoveToTarget(
                            ActionTarget {
                                entity: resource_entity,
                                range: 8f32,
                            },
                            Box::new(ActorState::Harvesting {
                                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                                resource_entity,
                                storehouse_entity,
                                resource_kind: kind,
                            }),
                        ));
                    }
                }
            }
        }
    }
}

pub fn actor_decide(
    mut messages: MessageReader<ActorStateChanged<DecidedState>>,
    mut trainings_changed: MessageWriter<TrainingClaimed>,
    mut actors: Query<
        (
            Entity,
            &mut ActorState,
            &mut sprite_animation::AnimationState,
        ),
        (With<PlayerParty>, Without<ActorIsDead>),
    >,
    mut trainings: Query<(Entity, &mut Training)>,
    mut can_be_continuous_messages: MessageWriter<ActorStateStateCouldBeContinuousState>,
    mut rng: ResMut<RandomSource>,
) {
    for entity1 in messages.read() {
        if let Ok((entity, mut actor_state, mut animation_state)) = actors.get_mut(entity1.0) {
            if let ActorState::Decided(tree) = &mut *actor_state {
                if tree.is_empty() {
                    info!(
                        "Actor {:?} has no valid decisions. Current state is {:?}, setting to wandering",
                        entity, actor_state
                    );
                    *actor_state =
                        ActorState::Wandering(Timer::from_seconds(15.0, TimerMode::Once), None);
                    *animation_state = sprite_animation::AnimationState::Walk(0);
                    continue;
                }
                let decisions = tree
                    .iter()
                    .sorted_by(|a, b| a.priority().partial_cmp(&b.priority()).unwrap())
                    .collect::<Vec<&Decision>>();
                let top_priority = decisions.iter().next().unwrap().priority();
                let top_decisions: Vec<&Decision> = decisions
                    .into_iter()
                    .take_while(|a| a.priority() == top_priority)
                    .collect();
                let decision = match top_decisions.len() {
                    1 => top_decisions[0],
                    x => top_decisions[rng.0.random_range(0..x)],
                };
                info!("Actor {:?} decided on action {:?}", entity, decision);

                match decision {
                    Decision::ClaimAndMoveToTarget(target, state_after_found) => {
                        if let Ok((training_entity, mut training)) =
                            trainings.get_mut(target.entity)
                        {
                            if training.trainee.is_none() {
                                trainings_changed
                                    .write(TrainingClaimed(training_entity, target.entity));
                                training.trainee = Some(entity);
                                *actor_state =
                                    ActorState::Walking(target.clone(), state_after_found.clone());
                                *animation_state = sprite_animation::AnimationState::Walk(0);
                            } else {
                                *actor_state = ActorState::Idle;
                                *animation_state = sprite_animation::AnimationState::Idle;
                            }
                        } else {
                            *actor_state = ActorState::Idle;
                            *animation_state = sprite_animation::AnimationState::Idle;
                        }
                    }
                    Decision::MoveToTarget(target, state_after_found) => {
                        *actor_state =
                            ActorState::Walking(target.clone(), state_after_found.clone());
                        *animation_state = sprite_animation::AnimationState::Walk(0);
                    }
                    Decision::Act(state) => {
                        *actor_state = state.clone();
                    }
                }
                can_be_continuous_messages
                    .write(ActorStateStateCouldBeContinuousState { entity: entity });
            }
        }
    }
}

impl Decision {
    pub fn priority(&self) -> u32 {
        match self {
            Decision::ClaimAndMoveToTarget(_, state_after_found) => {
                Decision::state_priority(state_after_found)
            }
            Decision::MoveToTarget(_, state_after_found) => {
                Decision::state_priority(state_after_found)
            }
            Decision::Act(state) => Decision::state_priority(state),
        }
    }

    fn state_priority(state: &ActorState) -> u32 {
        match state {
            ActorState::Attacking(_, _) => 1,
            ActorState::Training(_) => 2,
            ActorState::Constructing(_) => 3,
            ActorState::Consuming { .. } => 4,
            ActorState::Harvesting { .. } => 4,
            ActorState::Resting { .. } => 5,
            ActorState::Wandering(_, _) => 10,
            ActorState::Dead => 25,
            ActorState::Idle => 25,
            ActorState::Thinking(_) => 25,
            ActorState::Deciding(_) => 25,
            ActorState::Decided(_) => 25,
            ActorState::Walking(_, _) => 25,
            ActorState::ReturningFromTheHarvest { .. } => 25,
        }
    }
}

struct ActorJudgement<'a> {
    entity: Entity,
    transform: &'a Transform,
    stat_block: &'a ActorStatBlock,
    distance: f32,
    judgement: f32,
}
