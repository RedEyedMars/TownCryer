use bevy::prelude::*;
use bevy_northstar::nav::Nav;
use rand::{Rng, RngExt};
use saveable::Saveable;

#[derive(Component, Clone, Saveable)]
pub enum Ground {
    Grass(u8),
    Dirt(u8),
    Water,
    Shore(usize),
}
#[derive(Component, Debug, Clone, PartialEq, Eq, saveable::Saveable)]
pub enum FoodKind {
    SomeBuff,
}
#[derive(Component, Debug, Clone, PartialEq, Eq, saveable::Saveable)]
pub enum DrinkKind {
    HpPotion,
    ManaPotion,
}
#[derive(Component)]
pub struct Passable;

impl Ground {
    pub fn animation_index(&self, rng: &mut impl Rng) -> u32 {
        let mut weighed = rng.random_range(0..4);
        if weighed == 0 {
            weighed += 1;
        }
        weighed -= 1;
        match self {
            Ground::Grass(index) => weighed + *index as u32,
            Ground::Dirt(index) => weighed + *index as u32,
            Ground::Water => 4,
            Ground::Shore(index) => *index as u32,
        }
    }

    pub fn is_passable(&self) -> bool {
        match self {
            Ground::Grass(_) | Ground::Dirt(_) => true,
            Ground::Water | Ground::Shore(_) => false,
        }
    }
    pub fn nav(&self) -> Nav {
        match self {
            Ground::Grass(0) => Nav::Passable(1),
            Ground::Grass(3) => Nav::Passable(3),
            Ground::Grass(_) => Nav::Passable(2),
            Ground::Dirt(0) => Nav::Passable(4),
            Ground::Dirt(3) => Nav::Passable(8),
            Ground::Dirt(_) => Nav::Passable(7),
            Ground::Shore(_) => Nav::Passable(5),
            Ground::Water => Nav::Impassable,
        }
    }

    pub fn can_place_building(&self) -> bool {
        match self {
            Ground::Grass(_) | Ground::Dirt(_) => true,
            Ground::Water | Ground::Shore(_) => false,
        }
    }

    pub fn can_place_lumber(&self) -> bool {
        match self {
            Ground::Grass(_) => true,
            Ground::Water | Ground::Dirt(_) | Ground::Shore(_) => false,
        }
    }
    pub fn can_place_ore(&self) -> bool {
        match self {
            Ground::Dirt(_) => true,
            Ground::Water | Ground::Grass(_) | Ground::Shore(_) => false,
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..10) {
            0 | 1 | 2 => Ground::Grass(0),
            3 | 4 | 5 => Ground::Grass(3),
            6 => Ground::Dirt(0),
            7 => Ground::Dirt(3),
            _ => Ground::Water,
        }
    }

    pub fn tileset_index(&self) -> usize {
        match self {
            Ground::Dirt(_) => 0,
            Ground::Grass(_) => 1,
            Ground::Shore(_) => 2,
            Ground::Water => 3,
        }
    }
}
