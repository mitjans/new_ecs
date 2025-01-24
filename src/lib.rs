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

#[derive(Debug)]
pub struct Column {
    components: AnyVec,
}

#[derive(Debug)]
pub struct Archetype {
    columns: Vec<Column>,
    entities: Vec<EntityId>,
    column_index: HashMap<ComponentId, usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

        let component_id = TypeId::of::<T>();

        self.archetype.columns.push(Column {
            components: any_vec,
        });

        self.archetype
            .column_index
            .insert(component_id, self.archetype.column_index.len());

        self.components_set.insert(component_id);

        self
    }

    pub fn finish(mut self) -> EntityRecord {
        if let Some(archetype_id) = self.world.archetype_index.get(&self.components_set) {
            let archetype = self.world.archetypes.get_mut(*archetype_id).unwrap();

            let entity_record = EntityRecord {
                archetype_id: *archetype_id,
                id: self.world.entity_index.len(),
                row: archetype.column_index.len(),
            };

            archetype.entities.push(entity_record.id);

            archetype
                .column_index
                .iter()
                .for_each(|(component_id, index)| {
                    let column = archetype.columns.get_mut(*index).unwrap();
                    let component_index = self.archetype.column_index.get(component_id).unwrap();
                    let component = self
                        .archetype
                        .columns
                        .get(*component_index)
                        .unwrap()
                        .components
                        .get_raw(*component_index)
                        .unwrap();

                    column.components.push_raw(component);
                });

            self.world
                .entity_index
                .insert(entity_record.id, entity_record);

            entity_record
        } else {
            let entity_id = self.world.entity_index.len();
            let entity_record = EntityRecord {
                id: entity_id,
                archetype_id: self.world.archetypes.len(),
                row: 0,
            };

            self.components_set
                .iter()
                .enumerate()
                .for_each(|(index, component_id)| {
                    self.world
                        .component_index
                        .entry(*component_id)
                        .or_default()
                        .insert(self.world.archetypes.len(), index);
                });

            self.archetype.entities.push(entity_id);
            self.world.entity_index.insert(entity_id, entity_record);

            self.world
                .archetype_index
                .insert(self.components_set, self.world.archetypes.len());

            self.world.archetypes.push(self.archetype);

            entity_record
        }
    }
}

pub struct QueryCreator<'a> {
    world: &'a World,
    component_ids: Vec<ComponentId>,
}

impl<'a> QueryCreator<'a> {
    pub fn with_component<T: Any>(mut self) -> Self {
        self.component_ids.push(TypeId::of::<T>());
        self
    }

    pub fn run(self) -> (Vec<&'a AnyVec>, Vec<Vec<EntityId>>) {
        let mut components = vec![];
        let mut entities = vec![];

        self.component_ids.iter().for_each(|component_id| {
            let Some(archetype_ids) = self.world.component_index.get(component_id) else {
                panic!("Component not registered!")
            };

            archetype_ids.iter().for_each(|(archetype_id, column_id)| {
                let Some(archetype) = self.world.archetypes.get(*archetype_id) else {
                    panic!("Archetype not found!")
                };

                let Some(column) = archetype.columns.get(*column_id) else {
                    panic!("Column not found");
                };

                entities.push(archetype.entities.to_vec());
                components.push(&column.components);
            });
        });

        (components, entities)
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
            archetype: Archetype {
                columns: vec![],
                entities: Vec::new(),
                column_index: HashMap::new(),
            },
        }
    }

    pub fn query(&self) -> QueryCreator {
        QueryCreator {
            world: self,
            component_ids: vec![],
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

        assert_eq!(world.entity_index.len(), 1);

        assert_eq!(
            *world.entity_index.get(&entity_record.id).unwrap(),
            entity_record
        );
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

        let health = healths.first::<Health>().unwrap();
        let name = names.first::<Name>().unwrap();

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

    #[test]
    fn spawn_two_entities_with_the_same_component() {
        let mut world = World::default();

        let carles = Name(String::from("Carles"));
        let carles = world.spawn().with_component(carles).finish();

        let queco = Name(String::from("Queco"));
        let queco = world.spawn().with_component(queco).finish();

        assert_eq!(carles.id, 0);
        assert_eq!(carles.row, 0);

        assert_eq!(queco.id, 1);
        assert_eq!(queco.row, 1);

        assert_eq!(world.archetype_index.len(), 1);
        assert_eq!(world.entity_index.len(), 2);
        assert_eq!(world.component_index.len(), 1);

        let carles = world.get_component::<Name>(carles.id).unwrap();
        let queco = world.get_component::<Name>(queco.id).unwrap();

        assert_eq!(carles.0, "Carles");
        assert_eq!(queco.0, "Queco");
    }

    #[test]
    fn query() {
        let mut world = World::default();

        let carles = Name(String::from("Carles"));
        let carles = world.spawn().with_component(carles).finish();

        let queco = Name(String::from("Queco"));
        let queco = world.spawn().with_component(queco).finish();

        let (components, entities) = world.query().with_component::<Name>().run();

        assert_eq!(components.len(), 1);
        assert_eq!(components.first().unwrap().len(), 2);

        assert_eq!(entities.len(), 1);
        assert_eq!(entities.first().unwrap().len(), 2);

        let names = *components.first().unwrap();
        let entities = entities.first().unwrap();

        let first_name = names.first::<Name>().unwrap();
        let second_name = names.get::<Name>(1).unwrap();

        assert_eq!(first_name.0, "Carles");
        assert_eq!(second_name.0, "Queco");

        let first_entity = entities.first().unwrap();
        let second_entity = entities.get(1).unwrap();

        assert_eq!(first_entity, &carles.row);
        assert_eq!(second_entity, &queco.row);
    }
}
