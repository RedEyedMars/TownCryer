use bevy::{ecs::entity::EntityHashSet, prelude::*};

use rand::{Rng, RngExt};
use saveable::Saveable;

use crate::{
    RandomSource,
    actor::{
        ActorHome, ActorKind, PlayerParty,
        spawns::{SpawnPlayer, spawn_player_actor},
    },
    building::{
        castle::Castle,
        construction::FinishConstruction,
        resources::{CalculateClosestResource, ResourcesPlugin, StoredResource, Storehouse},
    },
    world::goods::{Bar, Coal, Lumber, Ore, Plank},
};

pub mod castle;
pub mod construction;
pub mod resources;
pub mod trainings;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourcesPlugin)
            .add_observer(construction::handle_new_contruction)
            .add_message::<FinishConstruction>()
            .add_systems(Update, spawn_villagers)
            .add_systems(
                Update,
                (
                    construction::update_construction,
                    construction::finish_construction,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Clone, PartialEq, Debug, Eq, Saveable)]
pub enum Building {
    Castle,
    House,
    Barracks,
    Market,
    Warehouse,
}

#[derive(Component, Saveable)]
pub struct BuildingInhabitants {
    pub count: u32,
    pub max: u32,
    pub entities: Vec<Entity>,
}

#[derive(Component)]
pub struct VillagerSpawningTimer {
    pub timer: Timer,
}

#[derive(Component, Clone, Debug)]
pub struct Training {
    pub required_kind: ActorKind,
    pub end_kind: ActorKind,
    pub guild: Entity,
    pub trainee: Option<Entity>,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Saveable)]
pub struct TrainingTemplate {
    pub required_kind: ActorKind,
    pub end_kind: ActorKind,
}
impl TrainingTemplate {
    pub fn create_training(&self, guild: Entity) -> Training {
        Training {
            required_kind: self.required_kind.clone(),
            end_kind: self.end_kind.clone(),
            guild,
            trainee: None,
        }
    }
}

#[derive(Message)]
pub struct TrainingClaimed(pub Entity, pub Entity);

#[derive(Component)]
pub struct TrainingTimer {
    pub timer: Timer,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Saveable)]
pub struct Guild {
    pub members: Vec<Entity>,
    pub trainings: Vec<Entity>,
    pub max_members: usize,
    pub training_template: TrainingTemplate,
}

impl Building {
    pub fn icon(
        &self,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        rng: &mut impl Rng,
    ) -> Sprite {
        let icon_texture = asset_server.load(self.sprite_path());
        let icon_index = self.gen_sprite_index(rng);
        let (pixels, icon_width, icon_height) = self.sprite_dimensions();

        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(pixels as u32),
            icon_width as u32,
            icon_height as u32,
            None,
            None,
        );
        let icon_texture_atlas_layout = texture_atlas_layouts.add(layout);
        Sprite::from_atlas_image(
            icon_texture,
            TextureAtlas {
                layout: icon_texture_atlas_layout,
                index: icon_index,
            },
        )
    }

    pub fn sprite_path(&self) -> &'static str {
        match self {
            Building::House => "tinyworld/Buildings/Purple/PurpleHouses.png",
            Building::Barracks => "tinyworld/Buildings/Purple/PurpleBarracks.png",
            Building::Market => "tinyworld/Buildings/Purple/PurpleMarket.png",
            Building::Castle => "tinyworld/Buildings/Purple/PurpleKeep.png",
            Building::Warehouse => "tinyworld/Buildings/Purple/PurpleDocks.png",
        }
    }

    pub fn gen_sprite_index(&self, rng: &mut impl Rng) -> usize {
        match self {
            Building::House => rng.random_range(0..12),
            Building::Barracks => 0,
            Building::Market => rng.random_range(0..12),
            Building::Castle => rng.random_range(0..6),
            Building::Warehouse => 0,
        }
    }

    pub fn sprite_dimensions(&self) -> (u8, u8, u8) {
        match self {
            Building::House => (16, 3, 4),
            Building::Barracks => (16, 4, 5),
            Building::Market => (16, 3, 4),
            Building::Castle => (32, 3, 2),
            Building::Warehouse => (16, 4, 2),
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            Building::House => {
                "Houses are the homes of the deizens. Holds capacity of (3/5/7) population."
            }
            Building::Barracks => {
                "Barracks can train Warriors. Strength focused swordsmen that weild melee weapons. Trains capacity of (3/4/5) swordsmen."
            }
            Building::Market => {
                "At the markets deizens can exchange their gold for tradeable goods such as food and potions."
            }
            Building::Castle => "Protect your castle to stay alive.",
            Building::Warehouse => {
                "Store your raw materials like lumber, ore, planks, or metal bars."
            }
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..4) {
            0 => Building::House,
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
    ) -> Entity {
        let texture = asset_server.load(self.sprite_path());
        let index = self.gen_sprite_index(rng);
        let (pixels, width, height) = self.sprite_dimensions();

        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(pixels as u32),
            width as u32,
            height as u32,
            None,
            None,
        );
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let transform = Transform::from_translation(position);
        let mut entity = commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index,
                },
            ),
            transform,
            PlayerParty,
            self.clone(),
        ));
        match self {
            Building::Castle => {
                entity.insert(Castle);
            }
            Building::House => {
                entity
                    .insert(BuildingInhabitants {
                        count: 0,
                        max: 1, //here
                        entities: vec![],
                    })
                    .insert(VillagerSpawningTimer {
                        timer: Timer::from_seconds(5.0, TimerMode::Repeating),
                    });
            }
            Building::Barracks => {
                entity.insert(Guild {
                    members: vec![],
                    trainings: vec![],
                    max_members: 4,
                    training_template: TrainingTemplate {
                        required_kind: ActorKind::Villager,
                        end_kind: ActorKind::Swordsman,
                    },
                });
            }
            Building::Market => {
                // TODO add stuff
            }
            Building::Warehouse => {
                entity.insert(Storehouse {
                    weight_capacity: 1000, // probably need more,
                    currently_stored_weight: 0,
                    resources: StoredResource::default_warehouse_store(
                        None, None, None, None, None,
                    ),
                });
            }
        }
        entity.id()
    }

    pub fn name(&self) -> &str {
        match self {
            Building::Castle => "Castle",
            Building::House => "House",
            Building::Barracks => "Barracks",
            Building::Market => "Market",
            Building::Warehouse => "Warehouse",
        }
    }

    pub fn is_passable(&self) -> bool {
        match self {
            _ => false,
        }
    }

    pub fn can_place_building(&self) -> bool {
        match self {
            _ => false,
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
    query: Query<(Entity, &Building, &mut BuildingState, &mut BuildingTimer), With<PlayerParty>>,
    mut spawn_writer: MessageWriter<SpawnPlayer>,
) {
    for (entity, _, mut state, mut timer) in query {
        if let Some(action) = &state.action {
            timer.timer.tick(time.delta());
            if timer.timer.just_finished() {
                match action {
                    BuildingAction::Train(actor_kind) => {
                        spawn_writer.write(SpawnPlayer {
                            kind: actor_kind.clone(),
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                            home: ActorHome { home: Some(entity) },
                            inhabitates: None,
                        });
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
    mut rng: ResMut<RandomSource>,
    houses: Query<(
        Entity,
        &Transform,
        &Building,
        &BuildingInhabitants,
        &mut VillagerSpawningTimer,
    )>,
    mut spawn_writer: MessageWriter<SpawnPlayer>,
) {
    for (house, transform, building, inhabitants, mut timer) in houses {
        if let Building::House = building {
            timer.timer.tick(time.delta());
            if timer.timer.just_finished() {
                if inhabitants.count < inhabitants.max {
                    let translation = Transform::from_translation(
                        transform.translation
                            + Vec3::new(rng.0.random_range(-8.0..8.0), -16.0, 0.0),
                    );

                    spawn_writer.write(SpawnPlayer {
                        kind: ActorKind::Villager,
                        transform: translation,
                        home: ActorHome { home: Some(house) },
                        inhabitates: Some(house),
                    });
                }
            }
        }
    }
}
