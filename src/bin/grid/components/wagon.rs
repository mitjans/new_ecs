use crate::{resources::tile_size::TileSize, Transformer};

use super::grid_position::GridPosition;
use new_ecs::World;
use raylib::prelude::*;

pub struct Wagon {
    pub position: GridPosition,
}

impl Wagon {
    pub fn draw(&self, world: &World, context: &mut RaylibDrawHandle) {
        let tile_size = world.get_resource::<TileSize>().unwrap();

        let coordinates = Transformer::coordinate(
            world,
            self.position,
            crate::components::anchor::Anchor::TopLeft,
        );

        context.draw_rectangle(
            coordinates.x as i32,
            coordinates.y as i32,
            tile_size.0,
            tile_size.0,
            Color::RED,
        );
    }
}
