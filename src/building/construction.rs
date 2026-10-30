use std::time::Duration;

use bevy::{ecs::entity::EntityHashSet, prelude::*};
use rand::{Rng, RngExt};
use saveable::Saveable;

use crate::{
    RandomSource,
    actor::{Actor, ActorIsDead, ActorState, ConstructingState, ContinuousActions},
    building::{Building, castle::Castle},
    sprite_animation::{self, AnimationState},
    world::{WorldWithTiles, tiles::ChangeTileComposition},
};

#[derive(Component, Clone, Saveable)]
pub struct Construction(pub Building, pub EntityHashSet);

#[derive(Component, Saveable)]
pub struct ConstructionTimer {
    pub timer: Timer,
}

#[derive(Event)]
pub struct StartConstruction {
    pub building: Building,
}

#[derive(Message)]
pub struct FinishConstruction(pub Entity);

pub fn handle_new_contruction(
    event: On<StartConstruction>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    castle_location: Single<&Transform, With<Castle>>,
    building_locations: Query<(&Transform, &Building)>,
    mut world: ResMut<WorldWithTiles>,
    mut rng: ResMut<RandomSource>,
    mut world_change_registry: MessageWriter<ChangeTileComposition>,
) {
    let building_kind = event.building.clone();
    for (location, building) in building_locations {
        if *building == building_kind {
            for _ in 0..5 {
                let theta = rng.0.random_range(0.0..2.0) * std::f32::consts::PI;
                let direction = Vec3::new(theta.cos(), theta.sin(), 0.0);

                let trial_location =
                    location.translation + (direction * (16f32 + rng.0.random_range(0.0..32.0)));
                let trial_position = world.get_xy(trial_location.xy());
                if let Some(trial_xy) = trial_position {
                    if !world.can_place_building(trial_xy) {
                        continue;
                    }
                    world.insert_construction(
                        trial_xy,
                        Construction(building_kind.clone(), EntityHashSet::new()),
                        &mut world_change_registry,
                    );
                    start_construction(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlas_layouts,
                        building_kind,
                        10f32,
                        world.get_transform(trial_xy),
                    );
                    return;
                }
            }
        }
    }
    for factor in 1u32..16u32 {
        for _ in 0..3 {
            let theta = rng.0.random_range(0.0..2.0) * std::f32::consts::PI;
            let direction = Vec3::new(theta.cos(), theta.sin(), 0.0);

            let trial_location = castle_location.translation
                + (direction * (16f32 * factor as f32 + rng.0.random_range(0.0..32.0)));
            let trial_position = world.get_xy(trial_location.xy());
            if let Some(trial_xy) = trial_position {
                if !world.can_place_building(trial_xy) {
                    continue;
                }
                world.insert_construction(
                    trial_xy,
                    Construction(building_kind.clone(), EntityHashSet::new()),
                    &mut world_change_registry,
                );
                start_construction(
                    &mut commands,
                    &asset_server,
                    &mut texture_atlas_layouts,
                    building_kind,
                    10f32,
                    world.get_transform(trial_xy),
                );
                return;
            }
        }
    }

    info!("HEYYYY, we failed to place a construction!!!");
}

pub fn start_construction(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    building: Building,
    construction_time: f32,
    position: Vec3,
) -> Entity {
    let texture = asset_server.load("tinyworld/User Interface/BoxSelector.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 2, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands
        .spawn((
            Construction(building, EntityHashSet::new()),
            ConstructionTimer {
                timer: Timer::from_seconds(construction_time, TimerMode::Once),
            },
            Transform::from_translation(position),
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
            ),
        ))
        .id()
}

pub fn update_construction(
    time: Res<Time>,
    query: Query<(Entity, &Transform, &mut ConstructionTimer, &Construction)>,
    mut builders: Query<(&mut ActorState, &mut AnimationState), Without<ActorIsDead>>,
    mut writer: MessageWriter<FinishConstruction>,
) {
    for (entity, transform, mut timer, construction) in query {
        let number_of_builders_on_this_site: i32 = construction
            .1
            .iter()
            .map(|builder| {
                builders.get(*builder).map_or(0, |(state, _)| match state {
                    ActorState::Constructing(site) if *site == entity => 1,
                    _ => 0,
                })
            })
            .sum();
        let mut factor = match construction.0 {
            Building::House => 1.0,
            _ => 0.0,
        };
        for i in 1..(number_of_builders_on_this_site + 1) {
            factor += 1.0 / (i * i) as f64;
        }
        timer
            .timer
            .tick(Duration::from_secs_f64(time.delta_secs_f64() * factor));
        if timer.timer.just_finished() {
            writer.write(FinishConstruction(entity));

            for (mut state, mut animation_state) in builders.iter_mut() {
                if let ActorState::Constructing(site) = *state {
                    if site == entity {
                        *state = ActorState::Idle;
                        *animation_state = sprite_animation::AnimationState::Idle;
                    }
                }
            }
        }
    }
}

pub fn finish_construction(
    mut reader: MessageReader<FinishConstruction>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<RandomSource>,
    constructions: Query<(Entity, &Construction, &Transform)>,
    mut world: ResMut<WorldWithTiles>,
    mut world_change_registry: MessageWriter<ChangeTileComposition>,
) {
    for message in reader.read() {
        if let Ok((entity, construction, transform)) = constructions.get(message.0) {
            let building_entity = construction.0.spawn(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                &mut rng.0,
                transform.translation,
            );

            world.insert_building(
                transform.translation,
                construction.0.clone(),
                building_entity,
                &mut world_change_registry,
            );
            commands.entity(entity).despawn();
        }
    }
}
