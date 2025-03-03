mod components;
pub use components::*;

use std::sync::LazyLock;

use voxidian_protocol::{
    registry::Registry,
    value::{
        CustomData, Damage, DataComponents, Item, ItemModel, ItemName, LengthPrefixVec, Lore,
        MaxDamage, Nbt as PtcNbt, NbtElement, SlotData, Text, VarInt,
    },
};

use crate::{
    components::{DataComponentHolder, DataComponentMap},
    values::{Id, TextKinds},
};

pub struct ItemType;

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub(crate) id: Id,
    pub(crate) map: DataComponentMap,
}

impl DataComponentHolder for ItemStack {
    fn component_map(&self) -> &crate::components::DataComponentMap {
        &self.map
    }

    fn component_map_mut(&mut self) -> &mut crate::components::DataComponentMap {
        &mut self.map
    }
}

impl ItemStack {
    pub fn new(id: Id) -> ItemStack {
        ItemStack {
            id,
            map: DataComponentMap::new().with(ItemComponents::ITEM_COUNT, 1),
        }
    }

    pub fn air() -> ItemStack {
        ItemStack {
            id: Id::constant("minecraft", "air"),
            map: DataComponentMap::new().with(ItemComponents::ITEM_COUNT, 1),
        }
    }

    pub fn kind(&self) -> Id {
        self.id.clone()
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::air()
    }
}

pub(crate) static ITEM_REGISTRY: LazyLock<Registry<Item>> = LazyLock::new(Item::vanilla_registry);

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        let mut components: Vec<DataComponents> = Vec::new();

        if let Ok(c) = value.get(ItemComponents::DAMAGE) {
            components.push(DataComponents::Damage(Damage {
                damage: VarInt::new(c),
            }));
        }
        if let Ok(amount) = value.get(ItemComponents::MAX_DAMAGE) {
            components.push(DataComponents::MaxDamage(MaxDamage {
                amount: VarInt::new(amount),
            }));
        }
        if let Ok(asset) = value.get(ItemComponents::ITEM_MODEL) {
            components.push(DataComponents::ItemModel(ItemModel {
                asset: asset.into(),
            }));
        }
        if let Ok(data) = value.get(ItemComponents::CUSTOM_DATA) {
            if let NbtElement::Compound(root) = NbtElement::from(data) {
                components.push(DataComponents::CustomData(CustomData {
                    data: PtcNbt {
                        name: String::new(),
                        root,
                    },
                }));
            }
        }
        if let Ok(name) = value.get(ItemComponents::ITEM_NAME) {
            components.push(DataComponents::ItemName(ItemName {
                name: {
                    let mut text = Text::new();
                    let component = match name {
                        TextKinds::Literal(text_literal) => text_literal.into(),
                    };
                    text.push(component);
                    text
                }
                .to_nbt(),
            }));
        }
        if let Ok(lore) = value.get(ItemComponents::LORE) {
            components.push(DataComponents::Lore(Lore {
                lines: {
                    let mut vec = Vec::new();
                    for line in lore {
                        vec.push({
                            let mut text = Text::new();
                            let component = match line {
                                TextKinds::Literal(text_literal) => text_literal.into(),
                            };
                            text.push(component);
                            text.to_nbt()
                        });
                    }
                    LengthPrefixVec::from(vec)
                },
            }));
        }

        let count = value
            .get(ItemComponents::ITEM_COUNT)
            .expect("All items must have an ItemComponents::ITEM_COUNT component")
            as i32;
        SlotData {
            id: ITEM_REGISTRY.get_entry(&value.id.into()).unwrap(),
            count: count.into(),
            components,
            removed_components: Vec::new(),
        }
    }
}

impl From<SlotData> for ItemStack {
    fn from(value: SlotData) -> Self {
        ItemStack {
            id: ITEM_REGISTRY.lookup(&value.id).unwrap().id.clone().into(),
            map: DataComponentMap::new()
                .with(ItemComponents::ITEM_COUNT, value.count.as_i32() as u16),
        }
    }
}
