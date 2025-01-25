use crate::{schedulers::draw_scheduler::DrawSystem, Train};

pub struct RenderTrainSystem;

impl DrawSystem for RenderTrainSystem {
    fn update(&self, world: &new_ecs::World, context: &mut raylib::prelude::RaylibDrawHandle) {
        let query = world.query().with_component::<Train>().iter(world);

        for result in query {
            let train = result.get::<Train>().unwrap();

            train.draw(world, context);

            train.wagons.iter().for_each(|wagon| {
                wagon.draw(world, context);
            });
        }
    }
}
