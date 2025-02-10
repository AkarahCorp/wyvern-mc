pub trait ComponentHolder<R: ComponentRegistry<Self>>: Sized {
    fn get<C: Component<Self, R, V>, V>(&self, component: &C) -> &V {
        component.get_component(self)
    }
}

pub trait ComponentRegistry<H: ComponentHolder<Self>>: Sized {}

pub trait Component<H: ComponentHolder<R>, R: ComponentRegistry<H>, V> {
    fn insert_component(&self, holder: &mut H, value: V);
    fn get_component<'a>(&self, holder: &'a H) -> &'a V;
}

#[cfg(test)]
mod tests {
    use crate::{dimension::entity::EntityType, values::Key};

    use super::{Component, ComponentHolder, ComponentRegistry};

    struct Entity {
        name: String,
        kind: Key<EntityType>,
    }

    impl ComponentHolder<EntityComponents> for Entity {}

    struct EntityComponents {}

    impl ComponentRegistry<Entity> for EntityComponents {}

    impl EntityComponents {
        const CUSTOM_NAME: CustomName = CustomName;
        const ENTITY_KIND: EntityKind = EntityKind;
    }

    struct CustomName;

    impl Component<Entity, EntityComponents, String> for CustomName {
        fn insert_component(&self, holder: &mut Entity, value: String) {
            holder.name = value;
        }

        fn get_component<'a>(&self, holder: &'a Entity) -> &'a String {
            &holder.name
        }
    }

    struct EntityKind;

    impl Component<Entity, EntityComponents, Key<EntityType>> for EntityKind {
        fn insert_component(&self, holder: &mut Entity, value: Key<EntityType>) {
            holder.kind = value;
        }

        fn get_component<'a>(&self, holder: &'a Entity) -> &'a Key<EntityType> {
            &holder.kind
        }
    }

    #[test]
    fn entity() {
        let entity = Entity {
            name: "Zombie".to_string(),
            kind: Key::new("minecraft", "zombie"),
        };
        let kind = entity.get(&EntityComponents::ENTITY_KIND);
        assert_eq!(*kind, Key::new("minecraft", "zombie"));
        let name = entity.get(&EntityComponents::CUSTOM_NAME);
        assert_eq!(*name, "Zombie".to_string());
    }
}
