use std::{any::Any, collections::HashMap, marker::PhantomData};

use voxidian_protocol::value::{
    Biome, CatVariant, ChickenVariant, CowVariant, DamageType, EntityModelType, EntityType,
    FrogVariant, PigVariant, SoundEvent, WolfSoundVariant,
};
use wyvern_datatypes::regval::{DimensionType, PaintingVariant, WolfVariant};
use wyvern_values::{Id, Registry, id};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) registries: HashMap<Id, Box<dyn Send + Sync + Any>>,
}

impl RegistryContainer {
    pub fn new() -> Self {
        let mut rc = RegistryContainer {
            registries: HashMap::new(),
        };
        rc.add_defaults();
        rc
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, registry: RegistryKey<T>) {
        self.registries
            .insert(registry.id().clone(), Box::new(Registry::<T>::new()));
    }

    pub fn get<T: 'static + Send + Sync>(&self, registry: RegistryKey<T>) -> &Registry<T> {
        self.registries
            .get(registry.id())
            .map(|x| x.downcast_ref::<Registry<T>>().unwrap())
            .unwrap()
    }

    pub fn get_mut<T: 'static + Send + Sync>(
        &mut self,
        registry: RegistryKey<T>,
    ) -> &mut Registry<T> {
        self.registries
            .get_mut(registry.id())
            .map(|x| x.downcast_mut::<Registry<T>>().unwrap())
            .unwrap()
    }

    pub fn add_defaults(&mut self) {
        self.insert(RegistryKeys::BIOME);
        self.insert(RegistryKeys::CAT_VARIANT);
        self.insert(RegistryKeys::CHICKEN_VARIANT);
        self.insert(RegistryKeys::COW_VARIANT);
        self.insert(RegistryKeys::DAMAGE_TYPE);
        self.insert(RegistryKeys::DIMENSION_TYPE);
        self.insert(RegistryKeys::ENTITY_TYPE);
        self.insert(RegistryKeys::FROG_VARIANT);
        self.insert(RegistryKeys::PAINTING_VARIANT);
        self.insert(RegistryKeys::PIG_VARIANT);
        self.insert(RegistryKeys::WOLF_SOUND_VARIANT);
        self.insert(RegistryKeys::WOLF_VARIANT);

        for entry in DamageType::vanilla_registry().entries() {
            self.get_mut(RegistryKeys::DAMAGE_TYPE).insert(
                entry.0.clone().into(),
                DamageType {
                    message_id: entry.1.message_id.clone(),
                    scaling: entry.1.scaling,
                    exhaustion: entry.1.exhaustion,
                    effects: entry.1.effects,
                    death_message_type: entry.1.death_message_type,
                },
            );
        }

        for entry in Biome::vanilla_registry().entries() {
            self.get_mut(RegistryKeys::BIOME)
                .insert(entry.0.clone().into(), entry.1.clone());
        }

        for entry in EntityType::vanilla_registry().entries() {
            self.get_mut(RegistryKeys::ENTITY_TYPE)
                .insert(entry.0.clone().into(), entry.1.clone());
        }

        self.get_mut(RegistryKeys::DIMENSION_TYPE)
            .insert(id![minecraft:overworld], DimensionType::default());

        self.get_mut(RegistryKeys::WOLF_VARIANT).insert(
            Id::empty(),
            WolfVariant {
                angry_texture: Id::empty(),
                wild_texture: Id::empty(),
                tame_texture: Id::empty(),
            },
        );

        self.get_mut(RegistryKeys::CAT_VARIANT).insert(
            Id::empty(),
            CatVariant {
                asset_id: Id::empty().into(),
            },
        );

        self.get_mut(RegistryKeys::PIG_VARIANT).insert(
            Id::empty(),
            PigVariant {
                asset_id: Id::empty().into(),
            },
        );

        self.get_mut(RegistryKeys::CHICKEN_VARIANT).insert(
            Id::empty(),
            ChickenVariant {
                asset_id: Id::empty().into(),
                model: EntityModelType::Normal,
            },
        );

        self.get_mut(RegistryKeys::COW_VARIANT).insert(
            Id::empty(),
            CowVariant {
                asset_id: Id::empty().into(),
                model: EntityModelType::Normal,
            },
        );

        self.get_mut(RegistryKeys::FROG_VARIANT).insert(
            Id::empty(),
            FrogVariant {
                asset_id: Id::empty().into(),
            },
        );

        self.get_mut(RegistryKeys::PAINTING_VARIANT).insert(
            Id::empty(),
            PaintingVariant {
                asset: Id::empty(),
                width: 1,
                height: 1,
            },
        );

        self.get_mut(RegistryKeys::WOLF_SOUND_VARIANT).insert(
            Id::empty(),
            WolfSoundVariant {
                ambient: SoundEvent {
                    name: Id::empty().into(),
                    fixed_range: None,
                },
                death: SoundEvent {
                    name: Id::empty().into(),
                    fixed_range: None,
                },
                growl: SoundEvent {
                    name: Id::empty().into(),
                    fixed_range: None,
                },
                hurt: SoundEvent {
                    name: Id::empty().into(),
                    fixed_range: None,
                },
                pant: SoundEvent {
                    name: Id::empty().into(),
                    fixed_range: None,
                },
                whine: SoundEvent {
                    name: Id::empty().into(),
                    fixed_range: None,
                },
            },
        );
    }
}

impl Default for RegistryContainer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RegistryKey<T> {
    name: Id,
    _phantom: PhantomData<T>,
}

impl<T> RegistryKey<T> {
    pub const fn new(name: Id) -> RegistryKey<T> {
        RegistryKey {
            name,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> &Id {
        &self.name
    }
}

pub struct RegistryKeys;

impl RegistryKeys {
    pub const DAMAGE_TYPE: RegistryKey<DamageType> = RegistryKey::new(id![minecraft:damage_type]);
    pub const BIOME: RegistryKey<Biome> = RegistryKey::new(id![minecraft:biome]);
    pub const WOLF_VARIANT: RegistryKey<WolfVariant> =
        RegistryKey::new(id![minecraft:wolf_variant]);
    pub const PIG_VARIANT: RegistryKey<PigVariant> = RegistryKey::new(id![minecraft:pig_variant]);
    pub const CAT_VARIANT: RegistryKey<CatVariant> = RegistryKey::new(id![minecraft:cat_variant]);
    pub const PAINTING_VARIANT: RegistryKey<PaintingVariant> =
        RegistryKey::new(id![minecraft:painting_variant]);
    pub const DIMENSION_TYPE: RegistryKey<DimensionType> =
        RegistryKey::new(id![minecraft:dimension_type]);
    pub const ENTITY_TYPE: RegistryKey<EntityType> = RegistryKey::new(id![minecraft:entity_type]);
    pub const CHICKEN_VARIANT: RegistryKey<ChickenVariant> =
        RegistryKey::new(id![minecraft:chicken_variant]);
    pub const COW_VARIANT: RegistryKey<CowVariant> = RegistryKey::new(id![minecraft:cow_variant]);
    pub const FROG_VARIANT: RegistryKey<FrogVariant> =
        RegistryKey::new(id![minecraft:frog_variant]);
    pub const WOLF_SOUND_VARIANT: RegistryKey<WolfSoundVariant> =
        RegistryKey::new(id![minecraft:wolf_sound_variant]);
}

impl std::fmt::Debug for RegistryContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryContainer { <fields hidden> }")
    }
}
