use voxidian_protocol::{
    packet::s2c::play::{Attribute, UpdateAttributesS2CPlayPacket},
    registry::RegEntry,
    value::{LengthPrefixVec, VarInt},
};

use crate::{
    components::{DataComponentHolder, DataComponentMap, DataComponentType},
    id,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeContainer {
    #[allow(unused)]
    attributes: DataComponentMap,
}

impl DataComponentHolder for AttributeContainer {
    fn component_map(&self) -> &DataComponentMap {
        &self.attributes
    }

    fn component_map_mut(&mut self) -> &mut DataComponentMap {
        &mut self.attributes
    }
}

impl AttributeContainer {
    pub fn new() -> AttributeContainer {
        AttributeContainer {
            attributes: DataComponentMap::new(),
        }
    }

    pub fn into_packet(&self, entity_id: i32) -> UpdateAttributesS2CPlayPacket {
        let mut properties = LengthPrefixVec::new();

        if let Ok(attack_speed) = self.get(Attributes::ATTACK_SPEED) {
            const ATTACK_SPEED: u32 = 0x04;
            properties.push(Attribute {
                id: unsafe { RegEntry::new_unchecked(ATTACK_SPEED) },
                value: attack_speed,
                mods: LengthPrefixVec::new(),
            });
        }
        if let Ok(max_health) = self.get(Attributes::MAX_HEALTH) {
            const MAX_HEALTH: u32 = 0x10;
            properties.push(Attribute {
                id: unsafe { RegEntry::new_unchecked(MAX_HEALTH) },
                value: max_health,
                mods: LengthPrefixVec::new(),
            });
        }
        if let Ok(follow_range) = self.get(Attributes::FOLLOW_RANGE) {
            const FOLLOW_RANGE: u32 = 0x0A;
            properties.push(Attribute {
                id: unsafe { RegEntry::new_unchecked(FOLLOW_RANGE) },
                value: follow_range,
                mods: LengthPrefixVec::new(),
            });
        }

        UpdateAttributesS2CPlayPacket {
            entity: VarInt::new(entity_id),
            properties,
        }
    }
}

impl Default for AttributeContainer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Attributes;
impl Attributes {
    pub const MAX_HEALTH: DataComponentType<f64> =
        DataComponentType::new(id![minecraft:max_health]);
    pub const ATTACK_SPEED: DataComponentType<f64> =
        DataComponentType::new(id![minecraft:attack_speed]);
    pub const FOLLOW_RANGE: DataComponentType<f64> =
        DataComponentType::new(id![minecraft:follow_range]);
}
