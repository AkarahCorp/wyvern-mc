use voxidian_protocol::value::{
    Biome, CatVariant, ChickenVariant, CowVariant, DamageType, EntityModelType, EntityType,
    FrogVariant, PaintingVariant as PtcPaintingVariant, PigVariant, SoundEvent, WolfSoundVariant,
    WolfVariant as PtcWolfVariant,
};

use wyvern_datatypes::regval::{DimensionType, PaintingVariant, WolfVariant};
use wyvern_values::{Id, Registry, id};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<PtcWolfVariant>,
    pub(crate) pig_variants: Registry<PigVariant>,
    pub(crate) cat_variants: Registry<CatVariant>,
    pub(crate) painting_variants: Registry<PtcPaintingVariant>,
    pub(crate) dimension_types: Registry<DimensionType>,
    pub(crate) entity_types: Registry<EntityType>,
    pub(crate) chicken_variants: Registry<ChickenVariant>,
    pub(crate) cow_variants: Registry<CowVariant>,
    pub(crate) frog_variants: Registry<FrogVariant>,
    pub(crate) wolf_sound_variants: Registry<WolfSoundVariant>,
}

impl std::fmt::Debug for RegistryContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryContainer { <fields hidden> }")
    }
}

pub struct RegistryContainerBuilder {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<PtcWolfVariant>,
    pub(crate) pig_variants: Registry<PigVariant>,
    pub(crate) cat_variants: Registry<CatVariant>,
    pub(crate) painting_variants: Registry<PtcPaintingVariant>,
    pub(crate) dimension_types: Registry<DimensionType>,
    pub(crate) entity_types: Registry<EntityType>,
    pub(crate) chicken_variants: Registry<ChickenVariant>,
    pub(crate) cow_variants: Registry<CowVariant>,
    pub(crate) frog_variants: Registry<FrogVariant>,
    pub(crate) wolf_sound_variants: Registry<WolfSoundVariant>,
}

impl From<RegistryContainerBuilder> for RegistryContainer {
    fn from(value: RegistryContainerBuilder) -> Self {
        Self {
            damage_types: value.damage_types,
            biomes: value.biomes,
            wolf_variants: value.wolf_variants,
            painting_variants: value.painting_variants,
            dimension_types: value.dimension_types,
            entity_types: value.entity_types,
            pig_variants: value.pig_variants,
            cat_variants: value.cat_variants,
            cow_variants: value.cow_variants,
            chicken_variants: value.chicken_variants,
            frog_variants: value.frog_variants,
            wolf_sound_variants: value.wolf_sound_variants,
        }
    }
}

impl RegistryContainerBuilder {
    pub fn add_defaults(&mut self) {
        self.dimension_type(id![minecraft:overworld], DimensionType::default());
        self.wolf_variant(
            Id::empty(),
            WolfVariant {
                angry_texture: Id::empty(),
                wild_texture: Id::empty(),
                tame_texture: Id::empty(),
            },
        );
        self.cat_variant(
            Id::empty(),
            CatVariant {
                asset_id: Id::empty().into(),
            },
        );
        self.pig_variant(
            Id::empty(),
            PigVariant {
                asset_id: Id::empty().into(),
            },
        );
        self.chicken_variant(
            Id::empty(),
            ChickenVariant {
                asset_id: Id::empty().into(),
                model: EntityModelType::Normal,
            },
        );
        self.cow_variant(
            Id::empty(),
            CowVariant {
                asset_id: Id::empty().into(),
                model: EntityModelType::Normal,
            },
        );
        self.frog_variant(
            Id::empty(),
            FrogVariant {
                asset_id: Id::empty().into(),
            },
        );
        self.painting_variant(
            Id::empty(),
            PaintingVariant {
                asset: Id::empty(),
                width: 1,
                height: 1,
            },
        );
        self.wolf_sound_variant(
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

    pub fn wolf_variant(&mut self, key: Id, value: WolfVariant) {
        self.wolf_variants.insert(key, value.into());
    }

    pub fn wolf_sound_variant(&mut self, key: Id, value: WolfSoundVariant) {
        self.wolf_sound_variants.insert(key, value);
    }

    pub fn cat_variant(&mut self, key: Id, value: CatVariant) {
        self.cat_variants.insert(key, value);
    }

    pub fn frog_variant(&mut self, key: Id, value: FrogVariant) {
        self.frog_variants.insert(key, value);
    }

    pub fn pig_variant(&mut self, key: Id, value: PigVariant) {
        self.pig_variants.insert(key, value);
    }

    pub fn chicken_variant(&mut self, key: Id, value: ChickenVariant) {
        self.chicken_variants.insert(key, value);
    }

    pub fn cow_variant(&mut self, key: Id, value: CowVariant) {
        self.cow_variants.insert(key, value);
    }

    pub fn painting_variant(&mut self, key: Id, value: PaintingVariant) {
        self.painting_variants.insert(key, value.into());
    }

    pub fn dimension_type(&mut self, key: Id, value: DimensionType) {
        self.dimension_types.insert(key, value);
    }
}
