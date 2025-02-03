use voxidian_protocol::{
    registry::Registry,
    value::{
        Biome, DamageType, DimType, EntityType, PaintingVariant as PtcPaintingVariant,
        WolfVariant as PtcWolfVariant,
    },
};

use crate::values::{
    Key,
    regval::{PaintingVariant, WolfVariant},
};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<PtcWolfVariant>,
    pub(crate) painting_variants: Registry<PtcPaintingVariant>,
    pub(crate) dimension_types: Registry<DimType>,
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
    pub(crate) dimension_types: Registry<DimType>,
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
    pub fn wolf_variant(&mut self, key: Key<WolfVariant>, value: WolfVariant) {
        self.wolf_variants.insert(key.into(), value.into());
    }

    pub fn painting_variant(&mut self, key: Key<PaintingVariant>, value: PaintingVariant) {
        self.painting_variants.insert(key.into(), value.into());
    }

    pub fn dimension_type(&mut self, key: Key<DimType>, value: DimType) {
        self.dimension_types.insert(key.into(), value);
    }
}
