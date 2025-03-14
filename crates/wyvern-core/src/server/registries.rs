use voxidian_protocol::value::{
    Biome, DamageType, EntityType, PaintingVariant as PtcPaintingVariant,
    WolfVariant as PtcWolfVariant,
};

use crate::values::{
    Id, Registry, id,
    regval::{DimensionType, PaintingVariant, WolfVariant},
};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<PtcWolfVariant>,
    pub(crate) painting_variants: Registry<PtcPaintingVariant>,
    pub(crate) dimension_types: Registry<DimensionType>,
    pub(crate) entity_types: Registry<EntityType>,
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
    pub(crate) painting_variants: Registry<PtcPaintingVariant>,
    pub(crate) dimension_types: Registry<DimensionType>,
    pub(crate) entity_types: Registry<EntityType>,
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
        }
    }
}

impl RegistryContainerBuilder {
    pub fn add_defaults(&mut self) {
        self.dimension_type(id![minecraft:overworld], DimensionType::default());
        self.wolf_variant(Id::empty(), WolfVariant {
            angry_texture: Id::empty(),
            wild_texture: Id::empty(),
            tame_texture: Id::empty(),
            biomes: Vec::new(),
        });
        self.painting_variant(Id::empty(), PaintingVariant {
            asset: Id::empty(),
            width: 1,
            height: 1,
        });
    }

    pub fn wolf_variant(&mut self, key: Id, value: WolfVariant) {
        self.wolf_variants.insert(key, value.into());
    }

    pub fn painting_variant(&mut self, key: Id, value: PaintingVariant) {
        self.painting_variants.insert(key, value.into());
    }

    pub fn dimension_type(&mut self, key: Id, value: DimensionType) {
        self.dimension_types.insert(key, value);
    }
}
