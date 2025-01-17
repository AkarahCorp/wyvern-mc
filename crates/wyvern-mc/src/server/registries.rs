use std::sync::Arc;

use voxidian_protocol::{
    registry::Registry,
    value::{Biome, DamageType, DimType, PaintingVariant, WolfVariant},
};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) damage_types: Arc<Registry<DamageType>>,
    pub(crate) biomes: Arc<Registry<Biome>>,
    pub(crate) wolf_variants: Arc<Registry<WolfVariant>>,
    pub(crate) painting_variants: Arc<Registry<PaintingVariant>>,
    pub(crate) dimension_types: Arc<Registry<DimType>>,
}

pub struct RegistryContainerBuilder {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<WolfVariant>,
    pub(crate) painting_variants: Registry<PaintingVariant>,
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
