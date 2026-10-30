use bevy_northstar::OrdinalGrid;
use bevy_northstar::components::{AgentOfGrid, AgentPos};
use bevy_northstar::grid::Grid;

use crate::building::BuildingInhabitants;
use crate::common::Direction;
use crate::sprite_animation::{self};

use crate::world::WorldWithTiles;
use crate::{RandomSource, actor::*};

#[derive(Message, Clone)]
pub struct SpawnAgent(Entity);

#[derive(Message)]
pub struct SpawnPlayer {
    pub kind: ActorKind,
    pub transform: Transform,
    pub home: ActorHome,
    pub inhabitates: Option<Entity>,
}

pub fn spawn_player_actor(
    mut spawns: MessageReader<SpawnPlayer>,
    mut spawn_agents: MessageWriter<SpawnAgent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<RandomSource>,
    mut inhabitants: Query<&mut BuildingInhabitants>,
) {
    for spawn in spawns.read() {
        let (kind, transform, home) = (spawn.kind.clone(), spawn.transform, spawn.home.clone());
        let sprite = sprite_animation::gen_sprite(
            kind.sprite_path(),
            &asset_server,
            &mut texture_atlas_layouts,
        );
        let entity = commands
            .spawn((
                Actor,
                PlayerParty,
                ActorState::Idle,
                kind.clone(),
                HealthMana {
                    current_hp: 100,
                    max_hp: 100,
                    current_mp: 50,
                    max_mp: 50,
                },
                PersonalNeeds {
                    current_energy: 0.0,
                    max_energy: 100.0,
                },
                ActorStatBlock {
                    strength: 10,
                    attack: 10,
                    defense: 5,
                    menace: 1.0,
                    reward: 50,
                },
                ActorName::from(kind.clone(), &mut rng.0),
                Inventory {
                    items: Vec::new(),
                    weight: 0u32,
                },
                transform,
                sprite,
                home,
                sprite_animation::AnimationState::Idle,
                Direction::South,
            ))
            .id();
        if let Some(inhabitate) = spawn.inhabitates {
            if let Ok(mut building) = inhabitants.get_mut(inhabitate) {
                building.entities.push(entity);
                building.count += 1;
            }
        }
        spawn_agents.write(SpawnAgent(entity));
    }
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
        HealthMana {
            current_hp: 100,
            max_hp: 100,
            current_mp: 50,
            max_mp: 50,
        },
        ActorStatBlock {
            strength: 10,
            attack: 10,
            defense: 5,
            menace: 1.0,
            reward: 50,
        },
        transform,
        sprite,
        Direction::South,
    ));
}

pub fn fix_agents(
    mut agents: MessageReader<SpawnAgent>,
    world: Res<WorldWithTiles>,
    mut commands: Commands,
    grid_entity: Single<Entity, With<OrdinalGrid>>,
    transforms: Query<&Transform, With<Actor>>,
) {
    for agent in agents.read() {
        if let Ok(transform) = transforms.get(agent.0) {
            if let Some(tile_pos) = world.get_position(transform) {
                commands
                    .entity(agent.0)
                    .insert(AgentPos(UVec3::new(tile_pos.x, tile_pos.y, 0)))
                    .insert(AgentOfGrid(*grid_entity));
            }
        }
    }
}
