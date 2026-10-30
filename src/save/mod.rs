use std::{cell::RefCell, fs::File, io::Write, sync::atomic::Ordering::SeqCst};

use bevy::{
    ecs::{
        entity::{EntityHashMap, EntityHashSet},
        query::{ArchetypeFilter, QueryData, QueryFilter},
    },
    prelude::*,
    tasks::{IoTaskPool, Task},
    window::WindowCloseRequested,
};
use rmp_serde::{Serializer, to_vec};
use serde::{
    Serialize,
    ser::{SerializeSeq, SerializeStruct},
};

use crate::{
    actor::{
        ActionTarget, Actor, ActorHome, ActorKind, ActorStatBlock, ActorState, HealthMana, Item,
        PersonalNeeds, PlayerParty, thinking::Decision,
    },
    building::{
        Building, BuildingInhabitants, Guild,
        construction::{Construction, ConstructionTimer},
        resources::Storehouse,
    },
    sprite_animation::AnimationState,
    world::{
        WorldWithTiles,
        goods::{Coal, Deposit, DepositMine, Lumber, Ore},
    },
};

#[derive(Resource)]
pub struct WorldSaveState(Timer, Option<Task<Result<Vec<u8>, std::io::Error>>>);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(WorldSaveState(
        Timer::from_seconds(15.0, TimerMode::Repeating),
        None,
    ));
}

pub fn save_world(
    mut event: MessageReader<WindowCloseRequested>,
    time: Res<Time>,
    mut state: ResMut<WorldSaveState>,
    world: Res<WorldWithTiles>,
    actors: Query<
        (
            Entity,
            &ActorState,
            &ActorKind,
            &ActorHome,
            &PersonalNeeds,
            &HealthMana,
            &ActorStatBlock,
            &AnimationState,
            &Transform,
        ),
        (With<Actor>, With<PlayerParty>),
    >,
    constructions: Query<(Entity, &Construction, &ConstructionTimer)>,
    guild_buildings: Query<(Entity, &Building, &Guild, &Transform), With<PlayerParty>>,
    house_buildings: Query<
        (Entity, &Building, &BuildingInhabitants, &Transform),
        With<PlayerParty>,
    >,
    storehouse_buildings: Query<(Entity, &Building, &Storehouse, &Transform), With<PlayerParty>>,
    lumber: Query<(Entity, &Lumber)>,
    ore_deposits: Query<(Entity, &Deposit<Ore>)>,
    coal_deposits: Query<(Entity, &Deposit<Coal>)>,
    ore_mines: Query<(Entity, &DepositMine<Ore>)>,
    coal_mines: Query<(Entity, &DepositMine<Coal>)>,
) {
    let mut filename = "towncryer_autosave.bin";
    let mut single_threaded = false;
    if !event.read().next().is_some() {
        state.0.tick(time.delta());
        if !state.0.just_finished() {
            return;
        }
        if let Some(ref task) = state.1 {
            if !task.is_finished() {
                return;
            }
        }
    } else {
        if state.1.is_some() {
            filename = "towncryer_backup.bin";
        }
        single_threaded = true;
    }

    info!("Start saving");
    let mut context = SaveContext {
        next_id: 032,
        entity_ids: EntityHashMap::<u32>::new(),
    };

    let world_bytes = context.save_world(&world).unwrap();
    let actor_bytes = context.save_actors(actors).unwrap();
    let constructions_bytes = context.save_query_2(constructions).unwrap();
    let guild_buildings_bytes = context.save_query_3(guild_buildings).unwrap();
    let house_buildings_bytes = context.save_query_3(house_buildings).unwrap();
    let storehouse_buildings_bytes = context.save_query_3(storehouse_buildings).unwrap();
    let lumber_bytes = context.save_simple_query(lumber).unwrap();
    let ore_deposit_bytes = context.save_simple_query(ore_deposits).unwrap();
    let ore_mines_bytes = context.save_simple_query(ore_mines).unwrap();
    let coal_deposit_bytes = context.save_simple_query(coal_deposits).unwrap();
    let coal_mines_bytes = context.save_simple_query(coal_mines).unwrap();

    let header_bytes = context.header_entities().unwrap();

    if !single_threaded {
        IoTaskPool::get()
            .spawn(async move {
                if let Ok(mut file) = File::create(filename) {
                    let _ = file.write_all(&header_bytes);
                    let _ = file.write_all(&world_bytes);
                    let _ = file.write_all(&actor_bytes);
                    let _ = file.write_all(&constructions_bytes);
                    let _ = file.write_all(&lumber_bytes);
                    let _ = file.write_all(&ore_deposit_bytes);
                    let _ = file.write_all(&ore_mines_bytes);
                    let _ = file.write_all(&coal_deposit_bytes);
                    let _ = file.write_all(&coal_mines_bytes);
                    info!(
                        "Successfully wrote {} bytes of binary data!",
                        actor_bytes.len() + constructions_bytes.len()
                    );
                }
            })
            .detach();
    } else {
        if let Ok(mut file) = File::create(filename) {
            let _ = file.write_all(&header_bytes);
            let _ = file.write_all(&world_bytes);
            let _ = file.write_all(&actor_bytes);
            let _ = file.write_all(&constructions_bytes);
            let _ = file.write_all(&lumber_bytes);
            let _ = file.write_all(&ore_deposit_bytes);
            let _ = file.write_all(&ore_mines_bytes);
            let _ = file.write_all(&coal_deposit_bytes);
            let _ = file.write_all(&coal_mines_bytes);
            info!(
                "Successfully wrote {} bytes of binary data!",
                actor_bytes.len() + constructions_bytes.len()
            );
        }
    }
}

pub struct SaveContext {
    pub next_id: u32,
    pub entity_ids: EntityHashMap<u32>,
}

use serde::ser::StdError;
impl SaveContext {
    fn save_world(
        &mut self,
        world: &WorldWithTiles,
    ) -> std::result::Result<Vec<u8>, std::boxed::Box<dyn StdError + 'static>> {
        let mut bytes1 = Vec::<u8>::new();
        let mut serializer1 = Serializer::new(&mut bytes1);

        world.width.serialize(&mut serializer1)?;
        world.height.serialize(&mut serializer1)?;
        world.tiles.iter().len().serialize(&mut serializer1)?;
        for actor in world.tiles.iter() {
            actor.save(&mut serializer1, self)?;
        }
        Ok(bytes1)
    }

    fn save_actors(
        &mut self,
        actors: Query<
            (
                Entity,
                &ActorState,
                &ActorKind,
                &ActorHome,
                &PersonalNeeds,
                &HealthMana,
                &ActorStatBlock,
                &AnimationState,
                &Transform,
            ),
            (With<Actor>, With<PlayerParty>),
        >,
    ) -> std::result::Result<Vec<u8>, std::boxed::Box<dyn StdError + 'static>> {
        let mut bytes1 = Vec::<u8>::new();
        let mut serializer1 = Serializer::new(&mut bytes1);

        actors.iter().len().serialize(&mut serializer1)?;
        for actor in actors.iter() {
            actor.save(&mut serializer1, self)?;
        }

        Ok(bytes1)
    }

    fn save_simple_query<T: Saveable + Component, F: QueryFilter + ArchetypeFilter>(
        &mut self,
        actors: Query<(Entity, &T), F>,
    ) -> std::result::Result<Vec<u8>, std::boxed::Box<dyn StdError + 'static>> {
        let mut bytes1 = Vec::<u8>::new();
        let mut serializer1 = Serializer::new(&mut bytes1);

        actors.iter().len().serialize(&mut serializer1)?;
        for actor in actors.iter() {
            actor.save(&mut serializer1, self)?;
        }

        Ok(bytes1)
    }

    fn save_query_2<
        T: Saveable + Component,
        U: Saveable + Component,
        F: QueryFilter + ArchetypeFilter,
    >(
        &mut self,
        actors: Query<(Entity, &T, &U), F>,
    ) -> std::result::Result<Vec<u8>, std::boxed::Box<dyn StdError + 'static>> {
        let mut bytes1 = Vec::<u8>::new();
        let mut serializer1 = Serializer::new(&mut bytes1);

        actors.iter().len().serialize(&mut serializer1)?;
        for actor in actors.iter() {
            actor.save(&mut serializer1, self)?;
        }

        Ok(bytes1)
    }

    fn save_query_3<
        T: Saveable + Component,
        U: Saveable + Component,
        V: Saveable + Component,
        F: QueryFilter + ArchetypeFilter,
    >(
        &mut self,
        actors: Query<(Entity, &T, &U, &V), F>,
    ) -> std::result::Result<Vec<u8>, std::boxed::Box<dyn StdError + 'static>> {
        let mut bytes1 = Vec::<u8>::new();
        let mut serializer1 = Serializer::new(&mut bytes1);

        actors.iter().len().serialize(&mut serializer1)?;
        for actor in actors.iter() {
            actor.save(&mut serializer1, self)?;
        }

        Ok(bytes1)
    }

    fn header_entities(&self) -> Result<Vec<u8>, Box<dyn StdError + 'static>> {
        let mut bytes2 = Vec::<u8>::new();
        let mut serializer2 = Serializer::new(&mut bytes2);
        self.entity_ids.len().serialize(&mut serializer2);

        Ok(bytes2)
    }

    fn id(&mut self, entity: &Entity) -> u32 {
        if !self.entity_ids.contains_key(entity) {
            self.entity_ids.insert(*entity, self.next_id);
            self.next_id += 1;
        }
        self.entity_ids[&entity]
    }
}

pub trait Saveable {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut crate::save::SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error>;
}

impl Saveable for EntityHashSet {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.len().serialize(&mut *serializer)?;
        for element in self.iter() {
            element.save(&mut *serializer, context)?;
        }
        Ok(())
    }
}

impl<T: Saveable> Saveable for Vec<T> {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.len().serialize(&mut *serializer)?;
        for element in self.iter() {
            element.save(&mut *serializer, context)?;
        }
        Ok(())
    }
}

impl<T: Saveable> Saveable for Option<T> {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        if let Some(s) = self {
            1u8.serialize(&mut *serializer)?;
            s.save(&mut *serializer, context)?;
        } else {
            0u8.serialize(&mut *serializer)?;
        }
        Ok(())
    }
}
impl<T: Saveable, U: Saveable> Saveable for (T, &U) {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<A: Saveable, B: Saveable, C: Saveable> Saveable for (A, &B, &C) {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<A: Saveable, B: Saveable, C: Saveable, D: Saveable> Saveable for (A, &B, &C, &D) {
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        self.3.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<A: Saveable, B: Saveable, C: Saveable, D: Saveable, E: Saveable> Saveable
    for (A, &B, &C, &D, &E)
{
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        self.3.save(&mut *serializer, context)?;
        self.4.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<A: Saveable, B: Saveable, C: Saveable, D: Saveable, E: Saveable, F: Saveable> Saveable
    for (A, &B, &C, &D, &E, &F)
{
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        self.3.save(&mut *serializer, context)?;
        self.4.save(&mut *serializer, context)?;
        self.5.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<A: Saveable, B: Saveable, C: Saveable, D: Saveable, E: Saveable, F: Saveable, G: Saveable>
    Saveable for (A, &B, &C, &D, &E, &F, &G)
{
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        self.3.save(&mut *serializer, context)?;
        self.4.save(&mut *serializer, context)?;
        self.5.save(&mut *serializer, context)?;
        self.6.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<
    A: Saveable,
    B: Saveable,
    C: Saveable,
    D: Saveable,
    E: Saveable,
    F: Saveable,
    G: Saveable,
    H: Saveable,
> Saveable for (A, &B, &C, &D, &E, &F, &G, &H)
{
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        self.3.save(&mut *serializer, context)?;
        self.4.save(&mut *serializer, context)?;
        self.5.save(&mut *serializer, context)?;
        self.6.save(&mut *serializer, context)?;
        self.7.save(&mut *serializer, context)?;
        Ok(())
    }
}
impl<
    A: Saveable,
    B: Saveable,
    C: Saveable,
    D: Saveable,
    E: Saveable,
    F: Saveable,
    G: Saveable,
    H: Saveable,
    I: Saveable,
> Saveable for (A, &B, &C, &D, &E, &F, &G, &H, &I)
{
    fn save(
        &self,
        serializer: &mut Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.0.save(&mut *serializer, context)?;
        self.1.save(&mut *serializer, context)?;
        self.2.save(&mut *serializer, context)?;
        self.3.save(&mut *serializer, context)?;
        self.4.save(&mut *serializer, context)?;
        self.5.save(&mut *serializer, context)?;
        self.6.save(&mut *serializer, context)?;
        self.7.save(&mut *serializer, context)?;
        self.8.save(&mut *serializer, context)?;
        Ok(())
    }
}

impl crate::save::Saveable for Entity {
    fn save(
        &self,
        serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        context.id(self).serialize(serializer)
    }
}

impl crate::save::Saveable for Timer {
    fn save(
        &self,
        serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.serialize(serializer)
    }
}

impl crate::save::Saveable for Vec3 {
    fn save(
        &self,
        serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.serialize(serializer)
    }
}

impl crate::save::Saveable for Transform {
    fn save(
        &self,
        serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
        context: &mut SaveContext,
    ) -> std::result::Result<(), rmp_serde::encode::Error> {
        self.serialize(serializer)
    }
}
