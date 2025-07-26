use core::time;

use crate::common::Direction;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub enum AnimationState {
    Idle,
    Walk(u8),
    Attack(Timer),
    SpecialAction(Timer),
}

#[derive(Resource, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

impl AnimationState {
    fn x(&self) -> u32 {
        use AnimationState::*;
        use std::cmp::min;
        match self {
            Idle => 0,
            Walk(frame) => 1 + *frame as u32,
            Attack(timer) | SpecialAction(timer) => min((timer.fraction() * 4f32) as u32, 3u32),
        }
    }
    fn y(&self) -> u32 {
        use AnimationState::*;
        match self {
            AnimationState::Idle => 0,
            AnimationState::Walk(_) => 0,
            AnimationState::Attack(_) => 4,
            AnimationState::SpecialAction(_) => 8,
        }
    }

    fn animate(&mut self, duration: Duration) {
        use AnimationState::*;
        match self {
            Walk(frame) => {
                *frame = (*frame + 1) % 4;
            }
            SpecialAction(timer) | Attack(timer) => {
                if timer.finished() {
                    *self = Idle;
                } else {
                    timer.tick(duration);
                }
            }
            _ => {}
        }
    }
}

impl Direction {
    fn animation_y(&self) -> u32 {
        match self {
            Direction::South => 0,
            Direction::North => 1,
            Direction::East => 2,
            Direction::West => 3,
        }
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut animation_timer: ResMut<AnimationTimer>,
    mut query: Query<(&mut AnimationState, &Direction, &mut Sprite)>,
) {
    for (mut state, direction, mut sprite) in query.iter_mut() {
        animation_timer.tick(time.delta());

        if !animation_timer.just_finished() {
            continue;
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = atlas_index(&state, direction);
            state.animate(time.delta());
        }
    }
}

fn atlas_index(state: &AnimationState, direction: &Direction) -> usize {
    let x = state.x();
    let y = state.y() + direction.animation_y();
    (y * 5 + x) as usize
}

pub fn gen_sprite(
    sprite_path: &str,
    asset_service: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> Sprite {
    let texture = asset_service.load(sprite_path);
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 5, 12, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    Sprite::from_atlas_image(
        texture,
        TextureAtlas {
            layout: texture_atlas_layout,
            index: 1,
        },
    )
}
