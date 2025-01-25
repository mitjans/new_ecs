use crate::{schedulers::update_scheduler::UpdateSystem, Ghost};

use raylib::prelude::*;

use crate::{Transformer, World};

#[derive(Default)]
pub struct GhostCursorSystem;

impl UpdateSystem for GhostCursorSystem {
    fn update(&mut self, world: &mut World, context: &mut RaylibHandle) {
        let mouse_coordinates = context.get_mouse_position();
        let mouse_position = Transformer::position(world, mouse_coordinates);

        let query = world.query().with_component::<Ghost>().iter(world);

        for mut result in query {
            let ghost = result.get_mut::<Ghost>().unwrap();

            if let Some(mouse_position) = mouse_position {
                ghost.selected_tile.set_position(mouse_position);
            }
        }
    }
}
