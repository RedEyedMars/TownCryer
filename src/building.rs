use bevy::prelude::*;

use rand::Rng;

use crate::{
    RandomSource,
    actor::{ActorKind, EnergySource, NeedsStorage, PlayerParty, spawns::spawn_player_actor},
    building,
    environment::{DrinkItem, FoodItem},
};

#[derive(Component, Clone)]
pub enum Building {
    House,
    Farm,
    Barracks,
    Market,
}

#[derive(Component)]
pub struct BuildingInhabitants {
    pub count: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct VillagerSpawningTimer {
    pub timer: Timer,
}

impl Building {
    pub fn sprite_path(&self) -> &str {
        match self {
            Building::House => "tinyworld/Buildings/Purple/PurpleHouses.png",
            Building::Farm => "tinyworld/Buildings/Purple/PurpleResources.png",
            Building::Barracks => "tinyworld/Buildings/Purple/PurpleBarracks.png",
            Building::Market => "tinyworld/Buildings/Purple/PurpleMarket.png",
        }
    }

    pub fn gen_sprite_index(&self, rng: &mut impl Rng) -> usize {
        match self {
            Building::House => rng.random_range(0..12),
            Building::Farm => rng.random_range(0..3),
            Building::Barracks => 0,
            Building::Market => rng.random_range(0..12),
        }
    }

    pub fn sprite_dimensions(&self) -> (u8, u8) {
        match self {
            Building::House => (3, 4),
            Building::Farm => (3, 5),
            Building::Barracks => (4, 5),
            Building::Market => (3, 4),
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..4) {
            0 => Building::House,
            1 => Building::Farm,
            2 => Building::Barracks,
            _ => Building::Market,
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        rng: &mut impl Rng,
        position: Vec3,
    ) {
        let texture = asset_server.load(self.sprite_path());
        let index = self.gen_sprite_index(rng);
        let (width, height) = self.sprite_dimensions();

        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            width as u32,
            height as u32,
            None,
            None,
        );
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let transform = Transform::from_translation(position);
        //transform.scale = Vec3::splat(2.0);
        let mut entity = commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index,
                },
            ),
            NeedsStorage {
                food: Vec::new(),
                drink: Vec::new(),
            },
            transform,
            PlayerParty,
            self.clone(),
        ));
        if let Building::House = self {
            entity
                .insert(BuildingInhabitants { count: 0, max: 5 })
                .insert(VillagerSpawningTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Repeating),
                })
                .insert(EnergySource {
                    recovery_rate: 10f32,
                });
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Building::House => "House",
            Building::Farm => "Farm",
            Building::Barracks => "Barracks",
            Building::Market => "Market",
        }
    }

    pub fn is_passable(&self) -> bool {
        match self {
            Building::House | Building::Farm | Building::Barracks | Building::Market => false,
        }
    }
}

#[derive(Component)]
pub enum BuildingAction {
    Train(ActorKind),
    Heal,
    Upgrade,
}

#[derive(Component)]
pub struct BuildingState {
    pub action: Option<BuildingAction>,
}

#[derive(Component)]
pub struct BuildingTimer {
    pub timer: Timer,
}

pub fn handle_building_action(
    time: Res<Time>,
    mut query: Query<
        (Entity, &Building, &mut BuildingState, &mut BuildingTimer),
        With<PlayerParty>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, _, mut state, mut timer) in query.iter_mut() {
        if let Some(action) = &state.action {
            timer.timer.tick(time.delta());
            if timer.timer.just_finished() {
                match action {
                    BuildingAction::Train(actor_kind) => {
                        spawn_player_actor(
                            &mut commands,
                            &asset_server,
                            &mut texture_atlas_layouts,
                            actor_kind.clone(),
                            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                            Some(entity),
                        );
                    }
                    BuildingAction::Heal => {
                        // Implement healing logic here
                    }
                    BuildingAction::Upgrade => {
                        // Implement upgrade logic here
                    }
                }
                state.action = None; // Reset action after completion
            }
        }
    }
}

pub fn spawn_villagers(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<RandomSource>,
    mut houses: Query<(
        Entity,
        &Transform,
        &Building,
        &mut BuildingInhabitants,
        &mut VillagerSpawningTimer,
    )>,
) {
    for (house, transform, building, mut inhabitants, mut timer) in houses.iter_mut() {
        if let Building::House = building {
            timer.timer.tick(time.delta());
            if timer.timer.just_finished() {
                if inhabitants.count < inhabitants.max {
                    spawn_player_actor(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlas_layouts,
                        ActorKind::Villager,
                        Transform::from_translation(
                            transform.translation
                                + Vec3::new(rng.0.random_range(-8.0..8.0), -16.0, 0.0),
                        ),
                        Some(house),
                    );
                    inhabitants.count += 1;
                }
            }
        }
    }
}
