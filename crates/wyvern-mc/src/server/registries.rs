use voxidian_protocol::{registry::Registry, value::{Biome, DamageType, DimType, PaintingVariant, WolfVariant}};

#[allow(dead_code)]
pub struct RegistryContainer {
    pub(crate) damage_types: Registry<DamageType>,
    pub(crate) biomes: Registry<Biome>,
    pub(crate) wolf_variants: Registry<WolfVariant>,
    pub(crate) painting_variants: Registry<PaintingVariant>,
    pub(crate) dimension_types: Registry<DimType>
}