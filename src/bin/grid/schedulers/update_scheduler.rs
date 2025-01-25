use new_ecs::World;
use raylib::RaylibHandle;

pub trait UpdateSystem: 'static {
    fn update(&mut self, world: &mut World, context: &mut RaylibHandle);
}

#[derive(Default)]
pub struct UpdateScheduler {
    systems: Vec<Box<dyn UpdateSystem>>,
}

impl UpdateScheduler {
    pub fn update(&mut self, world: &mut World, context: &mut RaylibHandle) {
        self.systems
            .iter_mut()
            .for_each(|system| system.update(world, context));
    }

    pub fn add_system<T: UpdateSystem>(&mut self, system: T) {
        self.systems.push(Box::new(system));
    }
}
