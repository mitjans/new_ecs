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
pub struct Station {
    pub position: GridPosition,
    pub color: Color,
    pub connections: Connections,
    pub connections_color: Color,
    pub connections_offset: f32,
}

impl Station {
    pub fn draw(&self, world: &World, context: &mut RaylibDrawHandle) {
        let world_width = world.get_resource::<WorldWidth>().unwrap();
        let world_height = world.get_resource::<WorldHeight>().unwrap();
        let grid_divisions = world.get_resource::<GridDivisions>().unwrap();
        let tile_size = world.get_resource::<TileSize>().unwrap();

        let offset = (0.2 * tile_size.0 as f32) as i32;

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

        let connection_offset = (self.connections_offset * tile_size.0 as f32) as i32;
        // Draw connections
        for connection in &self.connections.0 {
            if connection.row as i32 == position.row as i32 + 1 {
                context.draw_rectangle(
                    x + connection_offset,
                    y + tile_size.0 - offset,
                    tile_size.0 - connection_offset * 2,
                    offset,
                    self.connections_color,
                );
            }

            if connection.row as i32 == position.row as i32 - 1 {
                context.draw_rectangle(
                    x + connection_offset,
                    y,
                    tile_size.0 - connection_offset * 2,
                    offset,
                    self.connections_color,
                );
            }

            if connection.col as i32 == position.col as i32 + 1 {
                context.draw_rectangle(
                    x + tile_size.0 - offset,
                    y + connection_offset,
                    offset,
                    tile_size.0 - connection_offset * 2,
                    self.connections_color,
                );
            }

            if connection.col as i32 == position.col as i32 - 1 {
                context.draw_rectangle(
                    x,
                    y + connection_offset,
                    offset,
                    tile_size.0 - connection_offset * 2,
                    self.connections_color,
                );
            }
        }
    }
}
