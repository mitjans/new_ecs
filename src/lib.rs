use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

type ArchetypeMap = HashMap<ArchetypeId, usize>;

type Component = Box<dyn Any>;
type ComponentType = Vec<ComponentId>;

type EntityId = usize;
type ComponentId = TypeId;
type ArchetypeId = usize;

pub struct Archetype {
    components: Vec<Component>,
}

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,

    entity_index: HashMap<EntityId, ArchetypeId>,
    archetype_index: HashMap<ComponentType, ArchetypeId>,
    component_index: HashMap<ComponentId, ArchetypeMap>,
}

impl World {
    pub fn register_component<T: Any>(&mut self) {
        let component_id = TypeId::of::<T>();
        let archetype_id = self.archetypes.len();

        self.archetypes.push(Archetype { components: vec![] });

        self.archetype_index
            .insert(vec![component_id], archetype_id);
    }

    pub fn has_component<T: Any>(&self, entity: EntityId) -> bool {
        let Some(archetype_id) = self.entity_index.get(&entity) else {
            return false;
        };

        let Some(archetype_map) = self.component_index.get(&TypeId::of::<T>()) else {
            return false;
        };

        archetype_map.contains_key(archetype_id)
    }

    pub fn get_component<T: Any>(&self, entity: EntityId) -> Option<&T> {
        let archetype_id = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(*archetype_id)?;

        let archetype_map = self.component_index.get(&TypeId::of::<T>())?;
        let column_id = archetype_map.get(archetype_id)?;

        archetype.components.get(*column_id)?.downcast_ref::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Health;

    #[test]
    fn it_registers_components() {
        let mut world = World::default();
        let component_id = TypeId::of::<Health>();

        world.register_component::<Health>();

        assert_eq!(world.archetype_index.len(), 1);

        let archetype_id = world.archetype_index.get(&vec![component_id]).unwrap();

        assert_eq!(*archetype_id, 0);
    }
}
