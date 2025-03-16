use voxidian_protocol::{
    registry::RegValue,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};

#[derive(Debug, Clone)]
pub struct DimensionType {
    pub has_skylight: bool,
    pub natural: bool,

    pub min_y: i32,
    pub height: u32,
    pub effects: DimensionEffects,

    pub ambient_light_level: f32,
    pub piglin_safe: bool,
}

impl Default for DimensionType {
    fn default() -> Self {
        Self {
            has_skylight: true,
            natural: true,
            min_y: 0,
            height: 256,
            effects: DimensionEffects::Overworld,
            ambient_light_level: 15.0,
            piglin_safe: true,
        }
    }
}

impl DimensionType {
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn min_y(mut self, min_y: i32) -> Self {
        self.min_y = min_y;
        self
    }

    pub fn effects(mut self, effects: DimensionEffects) -> Self {
        self.effects = effects;
        self
    }
}

impl RegValue for DimensionType {
    const REGISTRY_ID: voxidian_protocol::value::Identifier = DimType::REGISTRY_ID;

    fn to_registry_data_packet(&self) -> Option<voxidian_protocol::value::Nbt> {
        let dt = self.clone();
        let dt: DimType = From::from(dt);
        dt.to_registry_data_packet()
    }
}

impl From<DimensionType> for DimType {
    fn from(value: DimensionType) -> Self {
        DimType {
            fixed_time: None,
            has_skylight: value.has_skylight,
            has_ceiling: false,
            ultrawarm: false,
            natural: value.natural,
            coordinate_scale: 1.0,
            bed_works: true,
            respawn_anchor_works: true,
            min_y: value.min_y,
            height: value.height,
            logical_height: value.height,
            infiniburn: "#minecraft:overworld_infiniburn".to_string(),
            effects: value.effects.into(),
            ambient_light: value.ambient_light_level,
            piglin_safe: value.piglin_safe,
            has_raids: false,
            monster_spawn_light_level: DimMonsterSpawnLightLevel::Constant(15),
            monster_spawn_block_light_limit: 15,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DimensionEffects {
    Overworld,
    Nether,
    End,
}

impl From<DimensionEffects> for DimEffects {
    fn from(value: DimensionEffects) -> Self {
        match value {
            DimensionEffects::Overworld => DimEffects::Overworld,
            DimensionEffects::Nether => DimEffects::Nether,
            DimensionEffects::End => DimEffects::End,
        }
    }
}
