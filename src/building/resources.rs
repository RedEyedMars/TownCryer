use std::{marker::PhantomData, time::Duration};

use bevy::{ecs::entity::EntityHashSet, prelude::*};
use itertools::{Itertools, chain};
use rand::{Rng, RngExt};
use saveable::Saveable;
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    RandomSource,
    actor::{Actor, ActorIsDead, ActorState, ConstructingState, ContinuousActions},
    building::{Building, castle::Castle},
    sprite_animation::{self, AnimationState},
    world::{
        WorldWithTiles,
        goods::{Bar, ChunksWithResource, Coal, Deposit, Lumber, Ore, Plank},
    },
};

pub struct ResourcesPlugin;
impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CalculateClosestResource>()
            .add_message::<ChangedStorehouseResources>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_new_storehouse,
                    CalculateClosestResource::handle_calculate_closest::<Lumber>,
                    CalculateClosestResource::handle_calculate_closest::<Deposit<Ore>>,
                    CalculateClosestResource::handle_calculate_closest::<Deposit<Coal>>,
                    handle_changing_storehouse_levels,
                ),
            );
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Eq, saveable::Saveable)]
pub enum ResourceKind {
    Lumber,
    Ore,
    Plank,
    Bar,
    Coal,
}

#[derive(Clone, Debug, PartialEq, Eq, Saveable)]
pub struct StoredResource {
    pub kind: ResourceKind,
    pub allowed_to_be_stored: bool,
    pub amount: u32,
    pub weight: u32,
    pub closest_entity: Option<Entity>,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Saveable)]
pub struct Storehouse {
    pub weight_capacity: u32,
    pub currently_stored_weight: u32,
    pub resources: Vec<StoredResource>,
}

#[derive(Resource)]
pub struct StorehousesResource {
    pub could_hold_more_lumber: EntityHashSet,
    pub could_hold_more_ore: EntityHashSet,
    pub could_hold_more_plank: EntityHashSet,
    pub could_hold_more_bar: EntityHashSet,
    pub could_hold_more_coal: EntityHashSet,

    pub lumber_amount: u32,
    pub ore_amount: u32,
    pub plank_amount: u32,
    pub bar_amount: u32,
    pub coal_amount: u32,
}

#[derive(Message)]
pub struct ChangedStorehouseResources {
    pub kind: ResourceKind,
    pub amount: i32,
    pub storehouse: Entity,
}

pub trait Resourceable {
    fn kind() -> ResourceKind;
}
pub trait Harvestable {
    fn can_be_harvested(&self) -> bool;
    fn harvest(&mut self, damage: u32) -> Option<ResourceKind>;
    fn despawn_when_harvested(&self) -> bool;
}

fn setup(mut commands: Commands) {
    commands.insert_resource(StorehousesResource {
        could_hold_more_lumber: EntityHashSet::new(),
        could_hold_more_ore: EntityHashSet::new(),
        could_hold_more_plank: EntityHashSet::new(),
        could_hold_more_bar: EntityHashSet::new(),
        could_hold_more_coal: EntityHashSet::new(),

        lumber_amount: 0u32,
        ore_amount: 0u32,
        coal_amount: 0u32,
        plank_amount: 0u32,
        bar_amount: 0u32,
    });
}

#[derive(Message)]
pub struct CalculateClosestResource {
    pub to: Entity,
    pub kind: ResourceKind,
}

impl Resourceable for Lumber {
    fn kind() -> ResourceKind {
        ResourceKind::Lumber
    }
}
impl Harvestable for Lumber {
    fn can_be_harvested(&self) -> bool {
        self.hp > 0
    }
    fn harvest(&mut self, damage: u32) -> Option<ResourceKind> {
        self.hp = self.hp.saturating_sub(damage);
        if self.hp == 0 {
            return Some(ResourceKind::Lumber);
        }
        None
    }
    fn despawn_when_harvested(&self) -> bool {
        true
    }
}
impl Resourceable for Ore {
    fn kind() -> ResourceKind {
        ResourceKind::Ore
    }
}
impl Harvestable for Ore {
    fn can_be_harvested(&self) -> bool {
        self.hp > 0
    }
    fn harvest(&mut self, damage: u32) -> Option<ResourceKind> {
        self.hp = self.hp.saturating_sub(damage);
        if self.hp == 0 {
            return Some(ResourceKind::Ore);
        }
        None
    }
    fn despawn_when_harvested(&self) -> bool {
        true
    }
}
impl Resourceable for Plank {
    fn kind() -> ResourceKind {
        ResourceKind::Plank
    }
}
impl Harvestable for Plank {
    fn can_be_harvested(&self) -> bool {
        true
    }
    fn harvest(&mut self, dmg: u32) -> Option<ResourceKind> {
        Some(ResourceKind::Plank)
    }
    fn despawn_when_harvested(&self) -> bool {
        true
    }
}

impl Resourceable for Bar {
    fn kind() -> ResourceKind {
        ResourceKind::Bar
    }
}
impl Harvestable for Bar {
    fn can_be_harvested(&self) -> bool {
        true
    }
    fn harvest(&mut self, dmg: u32) -> Option<ResourceKind> {
        Some(ResourceKind::Bar)
    }
    fn despawn_when_harvested(&self) -> bool {
        true
    }
}
impl Resourceable for Coal {
    fn kind() -> ResourceKind {
        ResourceKind::Coal
    }
}
impl Harvestable for Coal {
    fn can_be_harvested(&self) -> bool {
        self.hp > 0
    }
    fn harvest(&mut self, damage: u32) -> Option<ResourceKind> {
        self.hp = self.hp.saturating_sub(damage);
        if self.hp == 0 {
            return Some(ResourceKind::Ore);
        }
        None
    }
    fn despawn_when_harvested(&self) -> bool {
        true
    }
}

pub fn handle_new_storehouse(
    query: Query<(Entity, &Storehouse), Added<Storehouse>>,
    mut resource: ResMut<StorehousesResource>,
    mut closest_writer: MessageWriter<CalculateClosestResource>,
) {
    for (entity, storehouse) in query {
        for kind in ResourceKind::iter() {
            if storehouse.can_store(&kind) {
                resource.could_hold_mut(&kind).insert(entity);
            }

            closest_writer.write(CalculateClosestResource { to: entity, kind });
        }
    }
}

impl CalculateClosestResource {
    pub fn handle_calculate_closest<
        T: Component + Resourceable + Harvestable + Send + Sync + 'static,
    >(
        mut reader: MessageReader<CalculateClosestResource>,
        mut query: Query<(&Transform, &mut Storehouse)>,
        chunks: Res<ChunksWithResource<T>>,
        world: Res<WorldWithTiles>,
        resources: Query<(&Transform, &T), Without<Storehouse>>,
    ) {
        for message in reader.read() {
            let kind = <T as Resourceable>::kind();
            if kind != message.kind {
                continue;
            }
            info!("Calculate ---------- {:?}", kind);
            if let Ok((transform, mut storehouse)) = query.get_mut(message.to) {
                if let Some(stored_resource) =
                    storehouse.resources.iter_mut().find(|x| x.kind == kind)
                {
                    if let Some(entity) = stored_resource.closest_entity {
                        if let Ok((_, t)) = resources.get(entity) {
                            if t.can_be_harvested() {
                                info!("Calculating ---------- closest can be havested {:?}", kind);
                                continue;
                            }
                        }
                    }
                    let mut found_resource: Option<Entity> = None;
                    if let Some(tilepos) = world.get_position(transform) {
                        let tilepos_x = tilepos.x / 8;
                        let tilepos_y = tilepos.y / 8;
                        for extension in 0..8i32 {
                            let found_entity = chain![
                                (-extension..=extension).map(|ext_x| {
                                    (
                                        ((tilepos_x as i32 + ext_x) * 8).max(0) as u32,
                                        ((tilepos_y as i32 + extension) * 8).max(0) as u32,
                                    )
                                }),
                                (-extension..=extension).map(|ext_x| {
                                    (
                                        ((tilepos_x as i32 + ext_x) * 8).max(0) as u32,
                                        ((tilepos_y as i32 + -extension) * 8).max(0) as u32,
                                    )
                                }),
                                ((1 - extension).max(0)..=(extension - 1).min(0)).map(
                                    move |ext_y| {
                                        (
                                            ((tilepos_x as i32 + extension) * 8).max(0) as u32,
                                            ((tilepos_y as i32) + ext_y * 8).max(0) as u32,
                                        )
                                    },
                                ),
                                ((1 - extension).max(0)..=(extension - 1).min(0)).map(
                                    move |ext_y| {
                                        (
                                            ((tilepos_x as i32 + -extension) * 8).max(0) as u32,
                                            ((tilepos_y as i32 + ext_y) * 8).max(0) as u32,
                                        )
                                    },
                                )
                            ]
                            .filter(|(x, y)| chunks.has(*x, *y))
                            .flat_map(|(x, y)| {
                                (0..8)
                                    .flat_map(move |pos_x| {
                                        (0..8).map(move |pos_y| (x + pos_x, y + pos_y))
                                    })
                                    .map(|(x, y)| (world.get_resource(&kind, x, y), x, y))
                                    .filter(|(resource, _, _)| {
                                        resource.is_some()
                                            && resources.get(resource.unwrap()).is_ok()
                                    })
                                    .map(move |(resource, pos_x, pos_y)| {
                                        (
                                            resource.unwrap(),
                                            (tilepos.x as i32 - pos_x as i32).abs()
                                                + (tilepos.y as i32 - pos_y as i32).abs(),
                                        )
                                    })
                            })
                            .sorted_by(|(_, dist), (_, dist2)| dist.cmp(dist2))
                            .next();
                            if let Some(entity) = found_entity {
                                found_resource = Some(entity.0);
                                break;
                            }
                        }
                        stored_resource.closest_entity = found_resource;

                        info!(
                            "Calculated ---------- {:?}, {:?}",
                            kind, stored_resource.closest_entity
                        );
                    }
                }
            }
        }
    }
}

pub fn handle_changing_storehouse_levels(
    mut messages: MessageReader<ChangedStorehouseResources>,
    mut resources: ResMut<StorehousesResource>,
    mut storehouses: Query<&mut Storehouse>,
) {
    for message in messages.read() {
        if let Ok(mut storehouse) = storehouses.get_mut(message.storehouse) {
            if (message.amount > 0 && !storehouse.can_store(&message.kind))
                || (message.amount < 0
                    && !storehouse.has(&message.kind, message.amount.abs() as u32))
            {
                continue;
            }
            storehouse.add(&message.kind, message.amount);
            resources.add(&message.kind, message.amount);
            if message.amount > 0 && !storehouse.can_store(&message.kind) {
                resources
                    .could_hold_mut(&message.kind)
                    .remove(&message.storehouse);
            } else if message.amount < 0
                && !storehouse.has(&message.kind, message.amount.abs() as u32)
            {
                resources
                    .could_hold_mut(&message.kind)
                    .insert(message.storehouse);
            }
        }
    }
}

// TODO on despawn storehouse remove from StorehousesResource

impl StoredResource {
    pub fn default_warehouse_store(
        closest_lumber: Option<Entity>,
        closest_ore: Option<Entity>,
        closest_plank: Option<Entity>,
        closest_bar: Option<Entity>,
        closest_coal: Option<Entity>,
    ) -> Vec<StoredResource> {
        vec![
            StoredResource {
                kind: ResourceKind::Lumber,
                allowed_to_be_stored: true,
                amount: 0u32,
                weight: 0u32,
                closest_entity: closest_lumber,
            },
            StoredResource {
                kind: ResourceKind::Ore,
                allowed_to_be_stored: true,
                amount: 0u32,
                weight: 0u32,
                closest_entity: closest_ore,
            },
            StoredResource {
                kind: ResourceKind::Plank,
                allowed_to_be_stored: false,
                amount: 0u32,
                weight: 0u32,
                closest_entity: closest_plank,
            },
            StoredResource {
                kind: ResourceKind::Bar,
                allowed_to_be_stored: false,
                amount: 0u32,
                weight: 0u32,
                closest_entity: closest_bar,
            },
            StoredResource {
                kind: ResourceKind::Coal,
                allowed_to_be_stored: true,
                amount: 0u32,
                weight: 0u32,
                closest_entity: closest_coal,
            },
        ]
    }
}

impl Storehouse {
    pub fn can_store(&self, kind: &ResourceKind) -> bool {
        if self.currently_stored_weight + kind.weight() > self.weight_capacity {
            return false;
        }
        self.resources
            .iter()
            .any(|x| x.kind == *kind && x.allowed_to_be_stored)
    }
    pub fn has(&self, kind: &ResourceKind, amount: u32) -> bool {
        if self.currently_stored_weight < amount * kind.weight() {
            return false;
        }
        self.resources
            .iter()
            .any(|x| x.kind == *kind && x.amount >= amount)
    }
    pub fn add(&mut self, kind: &ResourceKind, amount: i32) {
        let found = self.resources.iter_mut().find(|x| x.kind == *kind);
        if let Some(found) = found {
            found.amount = (found.amount as i32 + amount) as u32;
            self.currently_stored_weight =
                (self.currently_stored_weight as i32 + amount * kind.weight() as i32) as u32;
        }
    }
    pub fn closest(&self, kind: &ResourceKind) -> Option<Entity> {
        self.resources
            .iter()
            .find(|x| x.kind == *kind)
            .map_or(None, |x| x.closest_entity)
    }
}

impl StorehousesResource {
    pub fn could_hold_mut(&mut self, kind: &ResourceKind) -> &mut EntityHashSet {
        match kind {
            ResourceKind::Lumber => &mut self.could_hold_more_lumber,
            ResourceKind::Ore => &mut self.could_hold_more_ore,
            ResourceKind::Plank => &mut self.could_hold_more_plank,
            ResourceKind::Bar => &mut self.could_hold_more_bar,
            ResourceKind::Coal => &mut self.could_hold_more_coal,
        }
    }
    pub fn could_hold(&self, kind: &ResourceKind) -> &EntityHashSet {
        match kind {
            ResourceKind::Lumber => &self.could_hold_more_lumber,
            ResourceKind::Ore => &self.could_hold_more_ore,
            ResourceKind::Plank => &self.could_hold_more_plank,
            ResourceKind::Bar => &self.could_hold_more_bar,
            ResourceKind::Coal => &self.could_hold_more_coal,
        }
    }

    pub fn add(&mut self, kind: &ResourceKind, amount: i32) {
        match kind {
            ResourceKind::Lumber => {
                self.lumber_amount = ((self.lumber_amount as i32) + amount).max(0) as u32
            }
            ResourceKind::Ore => {
                self.ore_amount = ((self.ore_amount as i32) + amount).max(0) as u32
            }
            ResourceKind::Plank => {
                self.plank_amount = ((self.plank_amount as i32) + amount).max(0) as u32
            }
            ResourceKind::Bar => {
                self.lumber_amount = ((self.bar_amount as i32) + amount).max(0) as u32
            }
            ResourceKind::Coal => {
                self.lumber_amount = ((self.coal_amount as i32) + amount).max(0) as u32
            }
        }
    }
}

impl ResourceKind {
    pub fn weight(&self) -> u32 {
        match self {
            ResourceKind::Lumber => 40,
            ResourceKind::Ore => 60,
            ResourceKind::Plank => 30,
            ResourceKind::Bar => 20,
            ResourceKind::Coal => 30,
        }
    }

    pub fn get_mut<'b, 'a>(
        self,
        entity: Entity,
        lumber: &'a mut Query<&'b mut Lumber>,
        ore: &'a mut Query<&'b mut Ore>,
        coal: &'a mut Query<&'b mut Coal>,
    ) -> Option<Box<dyn Harvestable + 'a>> {
        match self {
            ResourceKind::Lumber => lumber
                .get_mut(entity)
                .map_or(None, |lumber| Some(Box::new(lumber))),
            ResourceKind::Ore => ore.get_mut(entity).map_or(None, |ore| Some(Box::new(ore))),
            ResourceKind::Coal => coal
                .get_mut(entity)
                .map_or(None, |coal| Some(Box::new(coal))),
            _ => None,
        }
    }
}
impl<T: Harvestable> Harvestable for Mut<'_, T> {
    fn can_be_harvested(&self) -> bool {
        self.as_ref().can_be_harvested()
    }
    fn harvest(&mut self, damage: u32) -> Option<ResourceKind> {
        self.as_mut().harvest(damage)
    }
    fn despawn_when_harvested(&self) -> bool {
        self.as_ref().despawn_when_harvested()
    }
}
