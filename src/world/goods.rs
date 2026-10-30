use std::marker::PhantomData;

use bevy::{
    ecs::entity::unique_slice::Chunks,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::{Rng, RngExt};
use saveable::Saveable;

use crate::{
    RandomSource,
    building::resources::{Harvestable, ResourceKind, Resourceable},
    save::Saveable,
    world::{WorldWithTiles, tiles::ChangeTileComposition},
};

pub struct GoodsPlugin;
impl Plugin for GoodsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RequiredResourceGrowth<Lumber>>()
            .add_message::<RequiredResourceGrowth<Deposit<Ore>>>()
            .add_message::<RequiredResourceGrowth<Deposit<Coal>>>()
            .add_message::<CycleRequiredResourceGrowthTimer<Lumber>>()
            .add_message::<CycleRequiredResourceGrowthTimer<Deposit<Ore>>>()
            .add_message::<CycleRequiredResourceGrowthTimer<Deposit<Coal>>>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_replenishing_world_goods,
                    Lumber::handle_growth,
                    Deposit::<Ore>::handle_growth,
                    Deposit::<Coal>::handle_growth,
                    handle_replenishing_world_good::<Lumber>,
                    handle_replenishing_world_good::<Deposit<Ore>>,
                    handle_replenishing_world_good::<Deposit<Coal>>,
                ),
            )
            .add_observer(handle_despawn_resource::<Lumber>)
            .add_observer(handle_despawn_resource::<Deposit<Ore>>)
            .add_observer(handle_despawn_resource::<Deposit<Coal>>);
    }
}

#[derive(Component, Saveable)]
pub struct Lumber {
    pub hp: u32,
}

#[derive(Component, Saveable)]
pub struct Deposit<T: Resourceable + Default>
where
    T: Saveable,
{
    pub amount: u32,
    pub resource: T,
}

#[derive(Component, Saveable)]
pub struct DepositMine<T: Resourceable + Default>
where
    T: Saveable,
{
    pub amount: u32,
    pub resource: T,
}

#[derive(Component, Saveable)]
pub struct Ore {
    pub hp: u32,
}

#[derive(Component, Saveable)]
pub struct Coal {
    pub hp: u32,
}

#[derive(Component, Saveable)]
pub struct Bar {}
#[derive(Component, Saveable)]
pub struct Plank {}

#[derive(Component)]
pub struct ReplenishWorldGoodTimer(Timer);

#[derive(Message)]
pub struct RequiredResourceGrowth<T: Resourceable>(u32, PhantomData<T>);

#[derive(Message)]
pub struct CycleRequiredResourceGrowthTimer<T: Resourceable>(PhantomData<T>);

#[derive(Resource)]
pub struct ChunksWithResource<T: Resourceable> {
    pub has_good: HashMap<u32, HashMap<u32, u32>>,
    pub _marker: PhantomData<T>,
}
#[derive(Resource)]
pub struct ResourceConfiguration<T: Resourceable> {
    pub target_number: u32,
    pub _marker: PhantomData<T>,
}

pub fn setup(
    mut commands: Commands,
    mut lumber_writer: MessageWriter<RequiredResourceGrowth<Lumber>>,
    mut ore_writer: MessageWriter<RequiredResourceGrowth<Deposit<Ore>>>,
    mut coal_writer: MessageWriter<RequiredResourceGrowth<Deposit<Coal>>>,
) {
    let mut timer = Timer::from_seconds(10f32, TimerMode::Repeating);
    timer.almost_finish();
    commands.spawn(ReplenishWorldGoodTimer(timer));
    lumber_writer.write(RequiredResourceGrowth(150u32, PhantomData::default()));
    lumber_writer.write(RequiredResourceGrowth(50u32, PhantomData::default()));
    ore_writer.write(RequiredResourceGrowth(50u32, PhantomData::default()));
    coal_writer.write(RequiredResourceGrowth(50u32, PhantomData::default()));

    commands.insert_resource(ChunksWithResource {
        has_good: HashMap::new(),
        _marker: PhantomData::<Lumber>::default(),
    });
    commands.insert_resource(ChunksWithResource {
        has_good: HashMap::new(),
        _marker: PhantomData::<Deposit<Ore>>::default(),
    });
    commands.insert_resource(ChunksWithResource {
        has_good: HashMap::new(),
        _marker: PhantomData::<Plank>::default(),
    });
    commands.insert_resource(ChunksWithResource {
        has_good: HashMap::new(),
        _marker: PhantomData::<Bar>::default(),
    });
    commands.insert_resource(ChunksWithResource {
        has_good: HashMap::new(),
        _marker: PhantomData::<Deposit<Coal>>::default(),
    });
    commands.insert_resource(ResourceConfiguration {
        target_number: 2000u32,
        _marker: PhantomData::<Lumber>::default(),
    });
    commands.insert_resource(ResourceConfiguration {
        target_number: 50u32,
        _marker: PhantomData::<Deposit<Ore>>::default(),
    });
    commands.insert_resource(ResourceConfiguration {
        target_number: 0u32,
        _marker: PhantomData::<Plank>::default(),
    });
    commands.insert_resource(ResourceConfiguration {
        target_number: 0u32,
        _marker: PhantomData::<Bar>::default(),
    });
    commands.insert_resource(ResourceConfiguration {
        target_number: 50u32,
        _marker: PhantomData::<Deposit<Coal>>::default(),
    });
}

pub fn handle_replenishing_world_goods(
    time: Res<Time>,
    mut timer: Single<&mut ReplenishWorldGoodTimer>,
    mut lumber_writer: MessageWriter<CycleRequiredResourceGrowthTimer<Lumber>>,
    mut ore_writer: MessageWriter<CycleRequiredResourceGrowthTimer<Deposit<Ore>>>,
    mut coal_writer: MessageWriter<CycleRequiredResourceGrowthTimer<Deposit<Coal>>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        lumber_writer.write(CycleRequiredResourceGrowthTimer(PhantomData::default()));
        ore_writer.write(CycleRequiredResourceGrowthTimer(PhantomData::default()));
        coal_writer.write(CycleRequiredResourceGrowthTimer(PhantomData::default()));
    }
}

pub fn handle_replenishing_world_good<T: Component + Resourceable>(
    mut reader: MessageReader<CycleRequiredResourceGrowthTimer<T>>,
    mut writer: MessageWriter<RequiredResourceGrowth<T>>,
    resources: Query<&T>,
    configuration: Res<ResourceConfiguration<T>>,
) {
    if let Some(_) = reader.read().next() {
        let needed = configuration
            .target_number
            .saturating_sub(resources.iter().len() as u32);
        if needed == 0 {
            info!("{:?} has {:?}", T::kind(), resources.iter().len());
            return;
        }
        writer.write(RequiredResourceGrowth(
            needed as u32,
            PhantomData::default(),
        ));
    }
}

impl Lumber {
    pub fn handle_growth(
        mut reader: MessageReader<RequiredResourceGrowth<Lumber>>,
        mut writer: MessageWriter<CycleRequiredResourceGrowthTimer<Lumber>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        other_lumber_locations: Query<&Transform, With<Lumber>>,
        mut world: ResMut<WorldWithTiles>,
        mut rng: ResMut<RandomSource>,
        mut world_change_registry: MessageWriter<ChangeTileComposition>,
        mut lumber_chunks: ResMut<ChunksWithResource<Lumber>>,
    ) {
        let lumber_locations = other_lumber_locations.iter().collect::<Vec<&Transform>>();
        for event in reader.read() {
            if event.0 > 50u32 {
                writer.write(CycleRequiredResourceGrowthTimer(PhantomData::default()));
            }
            'outer: for _ in 0..event.0.min(50u32) {
                if lumber_locations.len() > 0 {
                    for _ in 0..5 {
                        let location =
                            lumber_locations[rng.0.random_range(0..lumber_locations.len())];
                        let theta = rng.0.random_range(0.0..2.0) * std::f32::consts::PI;
                        let direction = Vec3::new(theta.cos(), theta.sin(), 0.0);

                        let trial_location = location.translation
                            + (direction * (16f32 + rng.0.random_range(0.0..32.0)));
                        let trial_position = world.get_xy(trial_location.xy());
                        if let Some(trial_xy) = trial_position {
                            if !world.can_place_lumber(trial_xy) {
                                continue;
                            }
                            let entity = Lumber::spawn(
                                &mut commands,
                                &asset_server,
                                &mut texture_atlas_layouts,
                                world.get_transform(trial_xy),
                                &mut rng.0,
                            );
                            world.insert_resource(
                                trial_xy,
                                entity,
                                &mut world_change_registry,
                                &mut lumber_chunks,
                            );
                            continue 'outer;
                        }
                    }
                }
                for _ in 0..3 {
                    let trial_xy = TilePos::new(
                        rng.0.random_range(0..world.width),
                        rng.0.random_range(0..world.height),
                    );
                    if !world.can_place_lumber(trial_xy) {
                        continue;
                    }
                    let entity = Lumber::spawn(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlas_layouts,
                        world.get_transform(trial_xy),
                        &mut rng.0,
                    );
                    world.insert_resource(
                        trial_xy,
                        entity,
                        &mut world_change_registry,
                        &mut lumber_chunks,
                    );
                    continue 'outer;
                }

                info!("Failed to place a wood");
                writer.write(CycleRequiredResourceGrowthTimer(PhantomData::default()));
            }
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        position: Vec3,
        mut rng: &mut impl Rng,
    ) -> Entity {
        let texture = asset_server.load("tinyworld/Nature/Trees.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 4, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        commands
            .spawn((
                Lumber::default(),
                Transform::from_translation(position),
                Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: rng.random_range(1..4),
                    },
                ),
            ))
            .id()
    }
}

impl Default for Lumber {
    fn default() -> Self {
        Lumber { hp: 200 }
    }
}

impl<T: Resourceable + Default + Saveable + Send + Sync + 'static> Deposit<T> {
    pub fn handle_growth(
        mut reader: MessageReader<RequiredResourceGrowth<Deposit<T>>>,
        mut writer: MessageWriter<CycleRequiredResourceGrowthTimer<Deposit<T>>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        mut world: ResMut<WorldWithTiles>,
        mut rng: ResMut<RandomSource>,
        mut world_change_registry: MessageWriter<ChangeTileComposition>,
        mut chunks: ResMut<ChunksWithResource<Deposit<T>>>,
    ) {
        for event in reader.read() {
            'outer: for _ in 0..event.0 {
                for _ in 0..10 {
                    let trial_xy = TilePos::new(
                        rng.0.random_range(0..world.width),
                        rng.0.random_range(0..world.height),
                    );
                    if !world.can_place_deposit(trial_xy)
                        || chunks.get(trial_xy.x / 8, trial_xy.y / 8) > 5
                    {
                        continue;
                    }
                    let entity = Deposit::<T>::spawn(
                        &mut commands,
                        &asset_server,
                        &mut texture_atlas_layouts,
                        world.get_transform(trial_xy),
                    );
                    world.insert_resource(
                        trial_xy,
                        entity,
                        &mut world_change_registry,
                        &mut chunks,
                    );
                    continue 'outer;
                }

                info!("Failed to place a ore deposit");
                writer.write(CycleRequiredResourceGrowthTimer(PhantomData::default()));
            }
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        position: Vec3,
    ) -> Entity {
        let texture = asset_server.load("tinyworld/Buildings/Purple/PurpleResources2.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 3, 7, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        commands
            .spawn((
                Deposit::<T>::default(),
                Transform::from_translation(position),
                Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: 20
                            - match T::kind() {
                                ResourceKind::Coal => 1,
                                _ => 0,
                            },
                    },
                ),
            ))
            .id()
    }
}

impl Default for Ore {
    fn default() -> Self {
        Ore { hp: 400 }
    }
}

impl Default for Coal {
    fn default() -> Self {
        Coal { hp: 300 }
    }
}

impl<T: Resourceable + Default + Saveable> Default for Deposit<T> {
    fn default() -> Deposit<T> {
        Deposit {
            amount: 1000u32,
            resource: T::default(),
        }
    }
}

pub fn handle_despawn_resource<T: Component + Resourceable>(
    event: On<Despawn, T>,
    lumbers: Query<&Transform, With<T>>,
    mut world: ResMut<WorldWithTiles>,
    mut change_registry: MessageWriter<ChangeTileComposition>,
    mut chunks: ResMut<ChunksWithResource<T>>,
) {
    if let Ok(transform) = lumbers.get(event.entity) {
        if let Some(tile_pos) = world.get_position(transform) {
            world.remove_resource(tile_pos, &mut change_registry, &mut chunks);
        }
    }
}

impl<T: Resourceable> ChunksWithResource<T> {
    pub fn add(&mut self, x: u32, y: u32) {
        let x = x / 8;
        let y = y / 8;
        if !self.has_good.contains_key(&x) {
            self.has_good.insert(x, HashMap::new());
        }
        if !self.has_good[&x].contains_key(&y) {
            self.has_good.get_mut(&x).unwrap().insert(y, 0u32);
        }
        *self.has_good.get_mut(&x).unwrap().get_mut(&y).unwrap() += 1;
    }
    pub fn remove(&mut self, x: u32, y: u32) {
        let x = x / 8;
        let y = y / 8;
        if !self.has_good.contains_key(&x) {
            return;
        }
        if !self.has_good[&x].contains_key(&y) {
            return;
        }
        *self.has_good.get_mut(&x).unwrap().get_mut(&y).unwrap() = self
            .has_good
            .get_mut(&x)
            .unwrap()
            .get_mut(&y)
            .unwrap()
            .saturating_sub(0);
    }
    pub fn has(&self, x: u32, y: u32) -> bool {
        let x = x / 8;
        let y = y / 8;
        if let ResourceKind::Lumber = T::kind() {
            //info!("({:?},{:?})", x, y);
        }
        if !self.has_good.contains_key(&x) {
            return false;
        }
        if !self.has_good[&x].contains_key(&y) {
            return false;
        }
        *self.has_good.get(&x).unwrap().get(&y).unwrap() > 0
    }

    pub fn get(&self, x: u32, y: u32) -> u32 {
        let x = x / 8;
        let y = y / 8;
        if let ResourceKind::Lumber = T::kind() {
            //info!("({:?},{:?})", x, y);
        }
        if !self.has_good.contains_key(&x) {
            return 0;
        }
        if !self.has_good[&x].contains_key(&y) {
            return 0;
        }
        *self.has_good.get(&x).unwrap().get(&y).unwrap()
    }
}

impl<T: Resourceable + Default + Saveable> Resourceable for Deposit<T> {
    fn kind() -> ResourceKind {
        T::kind()
    }
}
impl<T: Resourceable + Harvestable + Default + Saveable> Harvestable for Deposit<T> {
    fn can_be_harvested(&self) -> bool {
        self.amount > 0
    }
    fn harvest(&mut self, damage: u32) -> Option<ResourceKind> {
        if self.amount == 0 {
            return None;
        }
        let harvest = self.resource.harvest(damage / 2);
        if harvest.is_some() {
            self.amount = self.amount.saturating_sub(1);
        }
        return harvest;
    }
    fn despawn_when_harvested(&self) -> bool {
        false
    }
}
