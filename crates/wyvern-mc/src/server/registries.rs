use std::sync::Arc;

use voxidian_protocol::{
    registry::Registry,
    value::{
        Biome, DamageType, DimType, PaintingVariant as PtcPaintingVariant,
        WolfVariant as PtcWolfVariant,
    },
};

use crate::values::{
    Key,
    regval::{PaintingVariant, WolfVariant},
};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) damage_types: Arc<Registry<DamageType>>,
    pub(crate) biomes: Arc<Registry<Biome>>,
    pub(crate) wolf_variants: Arc<Registry<PtcWolfVariant>>,
    pub(crate) painting_variants: Arc<Registry<PtcPaintingVariant>>,
    pub(crate) dimension_types: Arc<Registry<DimType>>,
}

pub struct RegistryContainerBuilder {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<PtcWolfVariant>,
    pub(crate) painting_variants: Registry<PtcPaintingVariant>,
    pub(crate) dimension_types: Registry<DimType>,
}

impl From<RegistryContainerBuilder> for RegistryContainer {
    fn from(value: RegistryContainerBuilder) -> Self {
        Self {
            damage_types: Arc::new(value.damage_types),
            biomes: Arc::new(value.biomes),
            wolf_variants: Arc::new(value.wolf_variants),
            painting_variants: Arc::new(value.painting_variants),
            dimension_types: Arc::new(value.dimension_types),
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
