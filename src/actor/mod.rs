use std::marker::PhantomData;

use bevy::{ecs::entity::EntityHashSet, prelude::*};
use rand::{Rng, RngExt};
use saveable::Saveable;

use crate::{
    actor::thinking::Decision,
    building::resources::{Harvestable, ResourceKind},
    environment::{DrinkKind, FoodKind},
    sprite_animation::AnimationState,
};
use itertools::Itertools;

#[derive(Component)]
pub struct Actor;

#[derive(Component, Clone, Debug, PartialEq, Eq, Saveable)]
pub enum ActorKind {
    TaxCollector,
    Villager,
    Swordsman,
    Archer,
    Mage,
}

#[derive(Component, Clone, Saveable)]
pub struct ActorHome {
    pub home: Option<Entity>,
}

#[derive(Message, Clone)]
pub struct ActorStateChanged<T>(Entity, PhantomData<T>);

#[derive(Message)]
pub struct ActorStateStateCouldBeContinuousState {
    entity: Entity,
}

#[derive(Message)]
pub struct ActorNeedsPathFound {
    entity: Entity,
}

#[derive(Component, Debug, Clone, Saveable)]
pub enum ActorState {
    Dead,
    Idle,
    Walking(ActionTarget, Box<ActorState>),
    Attacking(Timer, ActionTarget),
    Thinking(Timer),
    Deciding(Vec<Decision>),
    Decided(Vec<Decision>),
    Resting,
    Consuming {
        item: Item,
        item_index: usize,
        timer: Timer,
    },
    Wandering(Timer, Option<WanderingTarget>),
    Training(Entity),
    Constructing(Entity),
    Harvesting {
        timer: Timer,
        resource_entity: Entity,
        storehouse_entity: Entity,
        resource_kind: ResourceKind,
    },
    ReturningFromTheHarvest {
        timer: Timer,
        entity: Entity,
        kind: ResourceKind,
    },
}

pub trait ActorStateTrait {}

#[derive(Resource)]
pub struct ContinuousActions<T: ActorStateTrait> {
    pub entities: EntityHashSet,
    _marker: PhantomData<T>,
}

pub struct DeadState;
impl ActorStateTrait for DeadState {}
pub struct IdleState;
impl ActorStateTrait for IdleState {}
pub struct WalkingState;
impl ActorStateTrait for WalkingState {}
pub struct AttackingState;
impl ActorStateTrait for AttackingState {}
pub struct ThinkingState;
impl ActorStateTrait for ThinkingState {}
pub struct DecidingState;
impl ActorStateTrait for DecidingState {}
pub struct DecidedState;
impl ActorStateTrait for DecidedState {}
pub struct RestingState;
impl ActorStateTrait for RestingState {}
pub struct ConsumingState;
impl ActorStateTrait for ConsumingState {}
pub struct WanderingState;
impl ActorStateTrait for WanderingState {}
pub struct TrainingState;
impl ActorStateTrait for TrainingState {}
pub struct ConstructingState;
impl ActorStateTrait for ConstructingState {}
pub struct HarvestingState<T: Harvestable>(PhantomData<T>);
impl<T: Harvestable> ActorStateTrait for HarvestingState<T> {}
pub struct ReturningFromTheHarvestState;
impl ActorStateTrait for ReturningFromTheHarvestState {}

#[derive(Component, Saveable)]
pub struct ActorStatBlock {
    pub strength: u32,
    pub attack: u32,
    pub defense: u32,
    pub menace: f32,
    pub reward: u32,
}

#[derive(Component)]
pub struct ActorIsDead;

#[derive(Component)]
pub struct Attack {
    pub damage: u32,
}
#[derive(Component, Saveable)]
pub struct HealthMana {
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_mp: u32,
    pub max_mp: u32,
}
#[derive(Component, Debug, Saveable)]
pub struct PersonalNeeds {
    pub current_energy: f32,
    pub max_energy: f32,
}

#[derive(Component)]
pub struct EnergySource {
    pub recovery_rate: f32,
}

#[derive(Component)]
pub struct AttackTimer {
    pub timer: Timer,
}

#[derive(Debug, Clone, Saveable)]
pub struct ActionTarget {
    pub entity: Entity,
    pub range: f32,
}

#[derive(Debug, Clone, Saveable)]
pub struct WanderingTarget {
    pub target: Vec3,
}

#[derive(Clone, Debug)]
pub enum Need {
    Energy,
}

#[derive(Component, Debug)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub weight: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, saveable::Saveable)]
pub enum Item {
    Food(FoodKind),
    Drink(DrinkKind),
    Good(ResourceKind),
}

#[derive(Component)]
pub struct PlayerParty;

#[derive(Component, Clone)]
pub struct ActorName(pub String);

pub mod actions;
pub mod spawns;
pub mod thinking;

impl ActorKind {
    pub fn icon(
        &self,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        rng: &mut impl Rng,
    ) -> Sprite {
        let icon_texture = asset_server.load(self.sprite_path());
        let icon_index = self.gen_icon_sprite_index(rng);
        let (icon_width, icon_height) = self.icon_sprite_dimensions();

        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(16),
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
            ActorKind::Swordsman => {
                "tinyworld/Characters/Soldiers/Melee/PurpleMelee/SwordsmanPurple.png"
            }
            ActorKind::Archer => {
                "tinyworld/Characters/Soldiers/Ranged/PurpleRanged/ArcherPurple.png"
            }
            ActorKind::Mage => "tinyworld/Characters/Soldiers/Magic/PurpleMagic/MagePurple.png",
            ActorKind::Villager { .. } => {
                "tinyworld/Characters/Workers/PurpleWorker/VillagerPurple.png"
            }
            ActorKind::TaxCollector => {
                "tinyworld/Characters/Workers/PurpleWorker/VillagerPurple.png" // Placeholder, can be replaced with a unique sprite later
            }
        }
    }

    pub fn judge(
        &self,
        enemy_stat_block: &ActorStatBlock,
        stat_block: &ActorStatBlock,
        distance: f32,
    ) -> f32 {
        let greed = match self {
            ActorKind::Swordsman => 1.0,
            ActorKind::Archer => 0.8,
            ActorKind::Mage => 0.5,
            ActorKind::Villager => 0.2,
            ActorKind::TaxCollector => 0.0,
        };
        let ego = match self {
            ActorKind::Swordsman => 1.0,
            ActorKind::Archer => 0.8,
            ActorKind::Mage => 0.5,
            ActorKind::Villager => 0.2,
            ActorKind::TaxCollector => 0.0,
        };
        let normalized_distance = distance / 100.0;
        let attack_diff = stat_block.attack as f32 * ego - enemy_stat_block.defense as f32;
        let defense_diff = stat_block.defense as f32 - enemy_stat_block.attack as f32;
        let distance_factor = 1.0 / (1.0 + normalized_distance * normalized_distance);
        (attack_diff * enemy_stat_block.menace as f32 + defense_diff)
            * (enemy_stat_block.reward as f32 / 100f32 * greed)
            * distance_factor
    }

    pub fn weight_capacity(&self, stat_block: &ActorStatBlock) -> u32 {
        stat_block.strength * 10
    }

    pub fn gen_icon_sprite_index(&self, _rng: &mut impl Rng) -> usize {
        match self {
            ActorKind::Swordsman => 0,
            ActorKind::Archer => 0,
            ActorKind::Mage => 0,
            ActorKind::Villager => 0,
            ActorKind::TaxCollector => 0,
        }
    }

    pub fn icon_sprite_dimensions(&self) -> (u32, u32) {
        match self {
            ActorKind::Swordsman => (16, 16),
            ActorKind::Archer => (16, 16),
            ActorKind::Mage => (16, 16),
            ActorKind::Villager => (16, 16),
            ActorKind::TaxCollector => (16, 16),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ActorKind::Swordsman => "Swordsman",
            ActorKind::Archer => "Archer",
            ActorKind::Mage => "Mage",
            ActorKind::Villager => "Villager",
            ActorKind::TaxCollector => "Tax Collector",
        }
    }
}

impl PersonalNeeds {
    fn iter(&self) -> impl IntoIterator<Item = Need> {
        let energy_ratio = self.current_energy / self.max_energy;

        vec![(Need::Energy, energy_ratio)]
            .into_iter()
            .filter(|(_, ratio)| *ratio < 0.5f32)
            .sorted_by(|(_, ratio), (_, ratio2)| {
                ratio
                    .partial_cmp(ratio2)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(need, _)| need)
    }
}

impl ActorState {
    pub fn get_animation_state(&self) -> AnimationState {
        match self {
            ActorState::Dead => AnimationState::Idle,
            ActorState::Idle => AnimationState::Idle,
            ActorState::Walking(_, _) => AnimationState::Walk(0),
            ActorState::Attacking(timer, _) => AnimationState::Attack(timer.clone()),
            ActorState::Thinking(_) => AnimationState::Idle,
            ActorState::Deciding(_) => AnimationState::Idle,
            ActorState::Decided(_) => AnimationState::Idle,
            ActorState::Resting { .. } => AnimationState::Idle,
            ActorState::Consuming { .. } => AnimationState::Idle,
            ActorState::Wandering(_, _) => AnimationState::Idle,
            ActorState::Training(_) => AnimationState::Idle,
            ActorState::Constructing(_entity) => AnimationState::Idle,
            ActorState::Harvesting { .. } => AnimationState::Idle,
            ActorState::ReturningFromTheHarvest { .. } => AnimationState::Idle,
        }
    }
}

impl ActorName {
    pub fn from(kind: ActorKind, rng: &mut impl Rng) -> Self {
        let name = match kind {
            ActorKind::Swordsman => {
                let names = vec!["Arthas", "Grom", "Thorin", "Dain", "Eddard"];
                names[rng.random_range(0..names.len())]
            }
            ActorKind::Archer => {
                let names = vec!["Legolas", "Tauriel", "Robin", "Artemis", "Sylvanas"];
                names[rng.random_range(0..names.len())]
            }
            ActorKind::Mage => {
                let names = vec!["Gandalf", "Merlin", "Morgana", "Saruman", "Radagast"];
                names[rng.random_range(0..names.len())]
            }
            ActorKind::Villager => {
                let names = vec!["John", "Jane", "Bob", "Alice", "Charlie"];
                names[rng.random_range(0..names.len())]
            }
            ActorKind::TaxCollector => {
                let names = vec!["Taxman", "Collector", "Revenue", "Levy", "Toll"];
                names[rng.random_range(0..names.len())]
            }
        };
        ActorName(name.to_string())
    }
}

impl Item {
    pub fn weight(&self) -> u32 {
        match &self {
            Item::Good(kind) => kind.weight(),
            Item::Drink(_) => 1,
            Item::Food(_) => 2,
        }
    }
}

impl Inventory {
    pub fn add(&mut self, item: Item) {
        self.weight += item.weight();
        self.items.push(item);
    }

    pub fn remove(&mut self, item: &Item) {
        if let Some(index) = self.items.iter().position(|x| *x == *item) {
            self.items.remove(index);
        }
    }
    pub fn has(&self, item: &Item) -> bool {
        self.items.iter().any(|x| *x == *item)
    }
}
