use crate::{schedulers::draw_scheduler::DrawSystem, Ghost};

pub struct RenderGhostSystem;

impl DrawSystem for RenderGhostSystem {
    fn update(&self, world: &new_ecs::World, context: &mut raylib::prelude::RaylibDrawHandle) {
        let query = world.query().with_component::<Ghost>().iter(world);

        for result in query {
            let ghost = result.get::<Ghost>().unwrap();
            ghost.selected_tile.draw(world, context);
        }
    }
}
