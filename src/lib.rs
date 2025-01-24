mod any_vec;

use any_vec::AnyVec;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    collections::BTreeSet,
    collections::HashMap,
};

type ArchetypeMap = HashMap<ArchetypeId, usize>;

type ComponentType = BTreeSet<ComponentId>;

type EntityId = usize;
type ComponentId = TypeId;
type ArchetypeId = usize;

pub struct Column {
    components: AnyVec,
}

pub struct Archetype {
    columns: Vec<Column>,
}

#[derive(Clone, Copy)]
pub struct EntityRecord {
    pub id: EntityId,
    pub archetype_id: ArchetypeId,
    pub row: usize,
}

pub struct EntityCreator<'a> {
    world: &'a mut World,
    archetype: Archetype,
    components_set: ComponentType,
}

impl EntityCreator<'_> {
    pub fn with_component<T: Any>(mut self, component: T) -> Self {
        let mut any_vec = AnyVec::new(Layout::array::<T>(1).unwrap());
        any_vec.push(component);

        self.archetype.columns.push(Column {
            components: any_vec,
        });

        let component_id = TypeId::of::<T>();
        self.components_set.insert(component_id);

        self.world
            .component_index
            .entry(component_id)
            .or_default()
            .insert(
                self.world.archetypes.len(),
                self.archetype.columns.len() - 1,
            );
        self
    }

    pub fn finish(self) -> EntityRecord {
        let entity_id = self.world.entity_index.len();
        let entity_record = EntityRecord {
            id: entity_id,
            archetype_id: self.world.archetypes.len(),
            row: 0,
        };

        self.world.entity_index.insert(entity_id, entity_record);

        self.world
            .archetype_index
            .insert(self.components_set, self.world.archetypes.len());

        self.world.archetypes.push(self.archetype);

        entity_record
    }
}

#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,

    entity_index: HashMap<EntityId, EntityRecord>,
    archetype_index: HashMap<ComponentType, ArchetypeId>,
    component_index: HashMap<ComponentId, ArchetypeMap>,
}

impl World {
    pub fn has_component<T: Any>(&self, entity: EntityId) -> bool {
        let Some(entity_record) = self.entity_index.get(&entity) else {
            return false;
        };

        let Some(archetype_map) = self.component_index.get(&TypeId::of::<T>()) else {
            return false;
        };

        archetype_map.contains_key(&entity_record.archetype_id)
    }

    pub fn get_component<T: Any>(&self, entity: EntityId) -> Option<&T> {
        let entity_record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(entity_record.archetype_id)?;

        let archetype_map = self.component_index.get(&TypeId::of::<T>())?;
        let column_id = archetype_map.get(&entity_record.archetype_id)?;

        archetype
            .columns
            .get(*column_id)?
            .components
            .get::<T>(entity_record.row)
    }

    pub fn get_component_mut<T: Any>(&mut self, entity: EntityId) -> Option<&mut T> {
        let entity_record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get_mut(entity_record.archetype_id)?;

        let archetype_map = self.component_index.get(&TypeId::of::<T>())?;
        let column_id = archetype_map.get(&entity_record.archetype_id)?;

        archetype
            .columns
            .get_mut(*column_id)?
            .components
            .get_mut::<T>(entity_record.row)
    }

    pub fn spawn(&mut self) -> EntityCreator {
        EntityCreator {
            world: self,
            components_set: BTreeSet::new(),
            archetype: Archetype { columns: vec![] },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Health(u32);
    #[derive(Debug)]
    struct Name(String);

    #[test]
    fn spawn_entity_with_single_component() {
        let mut world = World::default();

        let entity_record = world.spawn().with_component(Health(150)).finish();

        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(entity_record.archetype_id, 0);
        assert_eq!(entity_record.row, 0);

        let archetype_set = world.component_index.get(&TypeId::of::<Health>()).unwrap();
        assert_eq!(archetype_set.len(), 1);

        let column_id = archetype_set.get(&entity_record.archetype_id).unwrap();
        assert_eq!(*column_id, 0);

        let health = world.archetypes[entity_record.archetype_id].columns[*column_id]
            .components
            .get::<Health>(entity_record.row);

        assert_eq!(health.unwrap().0, 150);
    }

    #[test]
    fn spawn_entity_with_multiple_components() {
        let mut world = World::default();

        let entity_record = world
            .spawn()
            .with_component(Health(40))
            .with_component(Name(String::from("Carles")))
            .finish();

        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(entity_record.archetype_id, 0);
        assert_eq!(entity_record.row, 0);

        let name_id = TypeId::of::<Name>();
        let health_id = TypeId::of::<Health>();
        let components_set = BTreeSet::from([name_id, health_id]);
        let archetype_id = world.archetype_index.get(&components_set).unwrap();

        assert_eq!(*archetype_id, 0);

        let archetype = world.archetypes.get(*archetype_id).unwrap();

        assert_eq!(archetype.columns.len(), 2);

        let healths = &archetype.columns.first().unwrap().components;
        let names = &archetype.columns.last().unwrap().components;

        assert_eq!(healths.len(), 1);
        assert_eq!(healths.len(), names.len());

        let health = healths.get::<Health>(0).unwrap();
        let name = names.get::<Name>(0).unwrap();

        assert_eq!(health.0, 40);
        assert_eq!(name.0, "Carles");

        let health = world.get_component::<Health>(entity_record.id).unwrap();
        let name = world.get_component::<Name>(entity_record.id).unwrap();
        assert_eq!(health.0, 40);
        assert_eq!(name.0, "Carles");

        let name = world.get_component_mut::<Name>(entity_record.id).unwrap();
        name.0 = "Queco".to_owned();

        let name = world.get_component::<Name>(entity_record.id).unwrap();
        assert_eq!(name.0, "Queco");
    }
}
