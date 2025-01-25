use crate::{
    components::{grid_position::GridPosition, train::Direction},
    schedulers::update_scheduler::UpdateSystem,
    Train, Transformer,
};

pub struct TrainMoveSystem;

impl UpdateSystem for TrainMoveSystem {
    fn update(&mut self, world: &mut new_ecs::World, rl: &mut raylib::RaylibHandle) {
        let query = world.query().with_component::<Train>().iter(world);

        for mut result in query {
            let train = result.get_mut::<Train>().unwrap();
            train.elapsed += rl.get_frame_time();

            if train.elapsed < 0.2 {
                continue;
            }

            train.elapsed = 0f32;

            if let Some(position) = train.route.pop_front() {
                let current_position = Transformer::position(world, train.coordinates).unwrap();
                train.last_position = Some(current_position);
                train.direction =
                    Direction::get_direction_from_positions(current_position, position);
                train.coordinates = Transformer::coordinate(world, position, crate::Anchor::Center);

                let mut next_position = current_position;
                let mut tmp = GridPosition::default();
                train.wagons.iter_mut().for_each(|wagon| {
                    tmp = wagon.position;
                    wagon.position = next_position;
                    next_position = tmp;
                })
            }
        }
    }
}
