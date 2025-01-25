use super::{connections::Connections, grid_position::GridPosition};
use crate::resources::{
    grid_divisions::GridDivisions, tile_size::TileSize, world_height::WorldHeight,
    world_width::WorldWidth,
};
use new_ecs::World;
use raylib::{
    color::Color,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

#[derive(Default, Debug)]
pub struct Rail {
    pub position: GridPosition,
    pub color: Color,
    pub connections: Connections,
}

impl Rail {
    pub fn draw(&self, world: &World, context: &mut RaylibDrawHandle) {
        let world_width = world.get_resource::<WorldWidth>().unwrap();
        let world_height = world.get_resource::<WorldHeight>().unwrap();
        let grid_divisions = world.get_resource::<GridDivisions>().unwrap();
        let tile_size = world.get_resource::<TileSize>().unwrap();

        let offset = (0.25 * tile_size.0 as f32) as i32;

        let position = self.position;

        let x = position.col as i32 * (world_width.0 / grid_divisions.0 as i32);
        let y = position.row as i32 * (world_height.0 / grid_divisions.0 as i32);

        // Draw square
        context.draw_rectangle(
            x + offset,
            y + offset,
            tile_size.0 - offset * 2,
            tile_size.0 - offset * 2,
            self.color,
        );

        // Draw connections
        for connection in &self.connections.0 {
            if connection.row as i32 == position.row as i32 + 1 {
                context.draw_rectangle(
                    x + offset,
                    y + tile_size.0 - offset,
                    tile_size.0 - offset * 2,
                    offset,
                    self.color,
                );
            }

            if connection.row as i32 == position.row as i32 - 1 {
                context.draw_rectangle(x + offset, y, tile_size.0 - offset * 2, offset, self.color);
            }

            if connection.col as i32 == position.col as i32 + 1 {
                context.draw_rectangle(
                    x + tile_size.0 - offset,
                    y + offset,
                    offset,
                    tile_size.0 - offset * 2,
                    self.color,
                );
            }

            if connection.col as i32 == position.col as i32 - 1 {
                context.draw_rectangle(x, y + offset, offset, tile_size.0 - offset * 2, self.color);
            }
        }
    }
}
