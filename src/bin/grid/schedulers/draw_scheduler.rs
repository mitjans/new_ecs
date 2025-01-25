use new_ecs::World;
use raylib::prelude::RaylibDrawHandle;

pub trait DrawSystem: 'static {
    fn update(&self, world: &World, context: &mut RaylibDrawHandle);
}

#[derive(Default)]
pub struct DrawScheduler {
    systems: Vec<Box<dyn DrawSystem>>,
}

impl DrawScheduler {
    pub fn render(&self, world: &mut World, context: &mut RaylibDrawHandle) {
        self.systems
            .iter()
            .for_each(|system| system.update(world, context));
    }

    pub fn add_system<T: DrawSystem>(&mut self, system: T) {
        self.systems.push(Box::new(system));
    }
}
