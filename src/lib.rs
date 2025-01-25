mod any_vec;

pub use any_vec::AnyVec;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    collections::{BTreeSet, HashMap},
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

    pub fn spawn(mut self) -> EntityRecord {
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

                    unsafe { column.components.push_raw(component) };
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

pub struct QueryCreator {
    component_ids: Vec<ComponentId>,
}

impl QueryCreator {
    pub fn with_component<T: Any>(mut self) -> Self {
        self.component_ids.push(TypeId::of::<T>());
        self
    }

    pub fn iter<'w>(&self, world: &'w World) -> QueryIter<'w> {
        let archetype_ids = self
            .component_ids
            .iter()
            .map(|component_id| {
                let Some(archetype_map) = world.component_index.get(component_id) else {
                    panic!("Component not regitered")
                };

                archetype_map.keys().copied().collect::<BTreeSet<_>>()
            })
            .reduce(|a, b| a.intersection(&b).cloned().collect())
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect();

        QueryIter {
            world,
            entity_index: 0,
            archetype_index: 0,
            archetype_ids,
            component_ids: self.component_ids.to_vec(),
        }
    }
}

pub struct QueryResult<'a> {
    entity_components: HashMap<ComponentId, &'a mut u8>,
}

impl QueryResult<'_> {
    pub fn get<T: Any>(&self) -> Option<&T> {
        let component_id = TypeId::of::<T>();
        let component = self.entity_components.get(&component_id)?;
        let component = unsafe { &(*(*component as *const u8 as *const T)) };
        Some(component)
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let component_id = TypeId::of::<T>();
        let component = self.entity_components.get_mut(&component_id)?;
        let component = unsafe { &mut (*(*component as *mut u8 as *mut T)) };
        Some(component)
    }
}

pub struct QueryIter<'w> {
    world: &'w World,
    entity_index: usize,
    archetype_index: usize,
    archetype_ids: Vec<ArchetypeId>,
    component_ids: Vec<ComponentId>,
}

impl<'a> Iterator for QueryIter<'a> {
    type Item = QueryResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.archetype_index >= self.archetype_ids.len() {
                return None;
            }

            let archetype = unsafe {
                self.world
                    .archetypes
                    .get_unchecked(self.archetype_ids[self.archetype_index])
            };

            if self.entity_index >= archetype.entities.len() {
                self.entity_index = 0;
                self.archetype_index += 1;
                continue;
            }

            let entity_components = self
                .component_ids
                .iter()
                .map(|component_id| {
                    let components =
                        &archetype.columns[archetype.column_index[component_id]].components;

                    (*component_id, unsafe {
                        &mut *(components.get_raw(self.entity_index).unwrap() as *mut u8)
                    })
                })
                .collect();

            self.entity_index += 1;
            return Some(QueryResult { entity_components });
        }
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

    pub fn query(&mut self) -> QueryCreator {
        QueryCreator {
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

        let entity_record = world.spawn().with_component(Health(150)).spawn();

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
            .spawn();

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
        let carles = world.spawn().with_component(carles).spawn();

        let queco = Name(String::from("Queco"));
        let queco = world.spawn().with_component(queco).spawn();

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
        world.spawn().with_component(carles).spawn();

        let queco = Name(String::from("Queco"));
        world
            .spawn()
            .with_component(queco)
            .with_component(Health(123))
            .spawn();

        let mut query_iter = world.query().with_component::<Name>().iter(&world);

        let x = query_iter.next().unwrap();
        let name = x.get::<Name>().unwrap();
        assert_eq!(name.0, "Carles");

        let x = query_iter.next().unwrap();
        let name = x.get::<Name>().unwrap();
        assert_eq!(name.0, "Queco");

        let health = x.get::<Health>();
        assert!(health.is_none());
    }

    #[test]
    fn complex_queries() {
        let mut world = World::default();

        let carles = Name(String::from("Carles"));
        world.spawn().with_component(carles).spawn();

        let queco = Name(String::from("Queco"));
        world
            .spawn()
            .with_component(queco)
            .with_component(Health(123))
            .spawn();

        // Check world indexes
        assert_eq!(world.component_index.len(), 2);
        assert_eq!(world.archetype_index.len(), 2);
        assert_eq!(world.entity_index.len(), 2);

        // Assert queries
        let mut query = world
            .query()
            .with_component::<Name>()
            .with_component::<Health>()
            .iter(&world);

        assert_eq!(query.next().unwrap().get::<Health>().unwrap().0, 123);
        assert!(query.next().is_none());

        let mut query = world.query().with_component::<Name>().iter(&world);
        assert_eq!(query.next().unwrap().get::<Name>().unwrap().0, "Carles");
        assert_eq!(query.next().unwrap().get::<Name>().unwrap().0, "Queco");
        assert!(query.next().is_none());
    }

    #[test]
    fn multiple_queries() {
        let mut world = World::default();

        let carles = Name(String::from("Carles"));
        world.spawn().with_component(carles).spawn();

        let queco = Name(String::from("Queco"));
        world
            .spawn()
            .with_component(queco)
            .with_component(Health(123))
            .spawn();

        let query1 = world.query().with_component::<Health>();
        let query2 = world.query().with_component::<Name>();

        let result1 = query1.iter(&world).next().unwrap();
        let mut result2 = query2.iter(&world).next().unwrap();

        let health = result1.get::<Health>().unwrap();
        assert_eq!(health.0, 123);

        let name = result2.get_mut::<Name>().unwrap();
        assert_eq!(name.0, "Carles");

        name.0 = String::from("Google");

        let mut query = world.query().with_component::<Name>().iter(&world);
        let result = query.next().unwrap();
        let name = result.get::<Name>().unwrap();
        assert_eq!(name.0, "Google");
    }
}
