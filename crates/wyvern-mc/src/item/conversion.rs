use voxidian_protocol::value::{
    CustomData, Damage, DataComponentTypes, DataComponents, ItemModel, ItemName, LengthPrefixVec,
    Lore, MaxDamage, Nbt as PtcNbt, NbtElement, SlotData, Text, VarInt,
};

use crate::{
    components::{DataComponentHolder, DataComponentMap},
    values::nbt::Nbt,
};

use super::{ITEM_REGISTRY, ItemComponents, ItemStack};

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        let mut components: Vec<DataComponents> = Vec::new();
        let mut filtered_components: Vec<DataComponentTypes> = Vec::new();
        if let Ok(c) = value.get(ItemComponents::DAMAGE) {
            components.push(DataComponents::Damage(Damage {
                damage: VarInt::new(c),
            }));
            filtered_components.push(DataComponentTypes::Damage);
        }
        if let Ok(amount) = value.get(ItemComponents::MAX_DAMAGE) {
            components.push(DataComponents::MaxDamage(MaxDamage {
                amount: VarInt::new(amount),
            }));
            filtered_components.push(DataComponentTypes::MaxDamage);
        }
        if let Ok(asset) = value.get(ItemComponents::ITEM_MODEL) {
            components.push(DataComponents::ItemModel(ItemModel {
                asset: asset.into(),
            }));

            filtered_components.push(DataComponentTypes::ItemModel);
        }
        if let Ok(data) = value.get(ItemComponents::CUSTOM_DATA) {
            if let NbtElement::Compound(root) = NbtElement::from(Nbt::Compound(data)) {
                components.push(DataComponents::CustomData(CustomData {
                    data: PtcNbt {
                        name: String::new(),
                        root,
                    },
                }));
                filtered_components.push(DataComponentTypes::CustomData);
            }
        }
        if let Ok(name) = value.get(ItemComponents::ITEM_NAME) {
            components.push(DataComponents::ItemName(ItemName {
                name: Into::<Text>::into(name).to_nbt(),
            }));

            filtered_components.push(DataComponentTypes::ItemName);
        }
        if let Ok(lore) = value.get(ItemComponents::LORE) {
            components.push(DataComponents::Lore(Lore {
                lines: {
                    let mut vec = Vec::new();
                    for line in lore {
                        vec.push(Into::<Text>::into(line).to_nbt());
                    }
                    LengthPrefixVec::from(vec)
                },
            }));

            filtered_components.push(DataComponentTypes::Lore);
        }

        let count = value
            .get(ItemComponents::ITEM_COUNT)
            .expect("All items must have an ItemComponents::ITEM_COUNT component")
            as i32;

        let removed_components = DataComponentTypes::all_types()
            .into_iter()
            .filter(|x| !filtered_components.contains(x))
            .collect();

        SlotData {
            id: ITEM_REGISTRY.get_entry(&value.id.into()).unwrap(),
            count: count.into(),
            components,
            removed_components,
        }
    }
}

impl From<SlotData> for ItemStack {
    fn from(value: SlotData) -> Self {
        let mut map =
            DataComponentMap::new().with(ItemComponents::ITEM_COUNT, value.count.as_i32() as u16);
        for component in value.components {
            match component {
                DataComponents::ItemName(name) => {
                    let text: Text = name.name.into();
                    map.set(ItemComponents::ITEM_NAME, text.into());
                }
                DataComponents::Lore(lore) => {
                    let mut lines = Vec::new();
                    for line in lore.lines.iter().cloned() {
                        let text: Text = line.into();
                        lines.push(text.into());
                    }
                    map.set(ItemComponents::LORE, lines);
                }
                DataComponents::Damage(damage) => {
                    map.set(ItemComponents::DAMAGE, damage.damage.as_i32())
                }
                DataComponents::MaxDamage(damage) => {
                    map.set(ItemComponents::DAMAGE, damage.amount.as_i32())
                }
                DataComponents::CustomData(data) => {
                    map.set(ItemComponents::CUSTOM_DATA, data.data.root.into());
                }
                DataComponents::ItemModel(id) => {
                    map.set(ItemComponents::ITEM_MODEL, id.asset.into());
                }
                _ => {}
            }
        }
        ItemStack {
            id: ITEM_REGISTRY.lookup(&value.id).unwrap().id.clone().into(),
            map,
        }
    }
}
