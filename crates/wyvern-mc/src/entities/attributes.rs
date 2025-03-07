use std::sync::LazyLock;

use voxidian_protocol::{
    packet::s2c::play::{Attribute, UpdateAttributesS2CPlayPacket},
    value::{AttributeType, LengthPrefixVec, VarInt},
};

use crate::{
    components::{DataComponentHolder, DataComponentMap, DataComponentType},
    values::Id,
    values::Registry,
};

pub static ATTRIBUTES: LazyLock<Registry<AttributeType>> =
    LazyLock::new(|| AttributeType::vanilla_registry().into());

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

        for attr in &self.attributes.inner {
            let float = (*attr.1.as_any()).downcast_ref::<f64>().unwrap_or(&0.0);
            if let Some(entry) = ATTRIBUTES.get_entry(attr.0.clone()) {
                properties.push(Attribute {
                    id: entry,
                    value: *float,
                    mods: LengthPrefixVec::new(),
                });
            }
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
wyvern_macros::generate_attrs_types!();
