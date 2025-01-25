use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Default)]
pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn add<T: Any>(&mut self, resource: T) {
        let type_id = resource.type_id();
        self.resources.insert(type_id, Box::new(resource));
    }

    pub fn get_ref<T: Any>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref())
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut())
    }

    pub fn delete<T: Any>(&mut self) {
        self.resources.remove(&TypeId::of::<T>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct WorldWidth(u32);

    #[test]
    fn test_add_resource() {
        let mut resources = Resources::default();

        let resource = WorldWidth(150);
        let resource_type_id = resource.type_id();

        resources.add(resource);

        let extracted = resources.resources.get(&resource_type_id).unwrap();
        let extracted = extracted.downcast_ref::<WorldWidth>().unwrap();

        assert_eq!(extracted.0, 150);
    }

    #[test]
    fn test_get_resource() {
        let mut resources = Resources::default();

        let resource = WorldWidth(150);

        resources.add(resource);

        let extracted = resources.get_ref::<WorldWidth>().unwrap();

        assert_eq!(extracted.0, 150);
    }

    #[test]
    fn test_get_resource_mut() {
        let mut resources = Resources::default();

        let resource = WorldWidth(150);

        resources.add(resource);

        let extracted = resources.get_mut::<WorldWidth>().unwrap();

        extracted.0 = 300;

        let extracted = resources.get_ref::<WorldWidth>().unwrap();

        assert_eq!(extracted.0, 300);
    }

    #[test]
    fn test_delete_resource() {
        let mut resources = Resources::default();

        let resource = WorldWidth(150);
        resources.add(resource);

        let extracted = resources.get_ref::<WorldWidth>();
        assert!(extracted.is_some());

        resources.delete::<WorldWidth>();
        let extracted = resources.get_ref::<WorldWidth>();
        assert!(extracted.is_none());
    }
}
