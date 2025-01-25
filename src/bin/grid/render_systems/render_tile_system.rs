use new_ecs::World;
use raylib::prelude::RaylibDrawHandle;

use crate::{components::tile::Tile, schedulers::draw_scheduler::DrawSystem};

#[derive(Default)]
pub struct RenderTileSystem;

impl DrawSystem for RenderTileSystem {
    fn update(&self, world: &World, context: &mut RaylibDrawHandle) {
        let query = world.query().with_component::<Tile>().iter(world);

        for result in query {
            let tile = result.get::<Tile>().unwrap();
            tile.draw(world, context);
        }
    }
}
