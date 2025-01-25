use crate::{
    schedulers::draw_scheduler::DrawSystem, GridDivisions, TileSize, WorldHeight, WorldWidth,
};
use raylib::prelude::*;

pub struct RenderGridSystem;

impl DrawSystem for RenderGridSystem {
    fn update(&self, world: &new_ecs::World, context: &mut raylib::prelude::RaylibDrawHandle) {
        let tile_size = world.get_resource::<TileSize>().unwrap();
        let divisions = world.get_resource::<GridDivisions>().unwrap();

        let world_width = world.get_resource::<WorldWidth>().unwrap();
        let world_height = world.get_resource::<WorldHeight>().unwrap();

        (0..divisions.0).for_each(|division| {
            let x = division as i32 * tile_size.0;
            context.draw_line(x, 0, x, world_height.0, Color::GRAY);
        });

        (0..divisions.0).for_each(|division| {
            let y = division as i32 * tile_size.0;
            context.draw_line(0, y, world_width.0, y, Color::GRAY);
        });
    }
}
