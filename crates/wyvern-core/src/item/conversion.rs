use voxidian_protocol::value::{
    BlocksAttacksComp, CustomDataComp, DamageComp, DataComponentTypes, DataComponents,
    EquippableComp, EquippableSlot, Identifier, ItemDamageFunction, ItemModelComp, ItemNameComp,
    LengthPrefixVec, LoreComp, MaxDamageComp, Nbt as PtcNbt, NbtElement, RegOr, SlotData,
    SoundEvent, Text, VarInt,
};

use wyvern_components::{DataComponentHolder, DataComponentMap};
use wyvern_datatypes::nbt::Nbt;
use wyvern_values::Id;

use super::{EquipmentSlot, EquippableComponent, ITEM_REGISTRY, ItemComponents, ItemStack};

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        let mut components: Vec<DataComponents> = Vec::new();
        let mut filtered_components: Vec<DataComponentTypes> = Vec::new();
        if let Ok(c) = value.get(ItemComponents::DAMAGE) {
            components.push(DataComponents::DamageComp(DamageComp {
                damage: VarInt::new(c),
            }));
            filtered_components.push(DataComponentTypes::DamageComp);
        }
        if let Ok(amount) = value.get(ItemComponents::MAX_DAMAGE) {
            components.push(DataComponents::MaxDamageComp(MaxDamageComp {
                amount: VarInt::new(amount),
            }));
            filtered_components.push(DataComponentTypes::MaxDamageComp);
        }
        if let Ok(asset) = value.get(ItemComponents::ITEM_MODEL) {
            components.push(DataComponents::ItemModelComp(ItemModelComp {
                asset: asset.into(),
            }));

            filtered_components.push(DataComponentTypes::ItemModelComp);
        }
        if let Ok(data) = value.get(ItemComponents::CUSTOM_DATA) {
            if let NbtElement::Compound(root) = NbtElement::from(Nbt::Compound(data)) {
                components.push(DataComponents::CustomDataComp(CustomDataComp {
                    data: PtcNbt {
                        name: String::new(),
                        root,
                    },
                }));
                filtered_components.push(DataComponentTypes::CustomDataComp);
            }
        }
        if let Ok(name) = value.get(ItemComponents::ITEM_NAME) {
            components.push(DataComponents::ItemNameComp(ItemNameComp {
                name: Into::<Text>::into(name).to_nbt(),
            }));

            filtered_components.push(DataComponentTypes::ItemNameComp);
        }
        if let Ok(lore) = value.get(ItemComponents::LORE) {
            components.push(DataComponents::LoreComp(LoreComp {
                lines: {
                    let mut vec = Vec::new();
                    for line in lore {
                        vec.push(Into::<Text>::into(line).to_nbt());
                    }
                    LengthPrefixVec::from(vec)
                },
            }));

            filtered_components.push(DataComponentTypes::LoreComp);
        }
        if let Ok(true) = value.get(ItemComponents::CAN_BLOCK) {
            components.push(DataComponents::BlocksAttacksComp(BlocksAttacksComp {
                block_delay_seconds: 0.0,
                disable_cooldown_scale: 0.0,
                damage_reductions: vec![].into(),
                item_damage: ItemDamageFunction {
                    thresold: 0.0,
                    base: 0.0,
                    factor: 0.0,
                },
                bypassed_by: vec![].into(),
                block_sound: None,
                disable_sound: None,
            }));
            filtered_components.push(DataComponentTypes::BlocksAttacksComp);
        }
        if let Ok(component) = value.get(ItemComponents::EQUIPPABLE) {
            components.push(DataComponents::EquippableComp(EquippableComp {
                slot: match component.slot {
                    EquipmentSlot::Mainhand => EquippableSlot::MainHand,
                    EquipmentSlot::Offhand => EquippableSlot::Offhand,
                    EquipmentSlot::Helmet => EquippableSlot::Head,
                    EquipmentSlot::Chestplate => EquippableSlot::Chest,
                    EquipmentSlot::Leggings => EquippableSlot::Legs,
                    EquipmentSlot::Boots => EquippableSlot::Feet,
                    EquipmentSlot::Body => EquippableSlot::Body,
                    EquipmentSlot::Saddle => EquippableSlot::Saddle,
                },
                equip_sound: RegOr::Or(SoundEvent {
                    name: component.equip_sound.into(),
                    fixed_range: None,
                }),
                model: Some(component.model.into()),
                camera_overlay: None,
                allowed_entities: None,
                dispensable: false,
                swappable: true,
                damage_on_hurt: false,
                equip_on_interact: false,
            }));
            filtered_components.push(DataComponentTypes::EquippableComp);
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
                DataComponents::ItemNameComp(name) => {
                    let text: Text = name.name.into();
                    map.set(ItemComponents::ITEM_NAME, text.into());
                }
                DataComponents::LoreComp(lore) => {
                    let mut lines = Vec::new();
                    for line in lore.lines.iter().cloned() {
                        let text: Text = line.into();
                        lines.push(text.into());
                    }
                    map.set(ItemComponents::LORE, lines);
                }
                DataComponents::DamageComp(damage) => {
                    map.set(ItemComponents::DAMAGE, damage.damage.as_i32())
                }
                DataComponents::MaxDamageComp(damage) => {
                    map.set(ItemComponents::DAMAGE, damage.amount.as_i32())
                }
                DataComponents::CustomDataComp(data) => {
                    map.set(ItemComponents::CUSTOM_DATA, data.data.root.into());
                }
                DataComponents::ItemModelComp(id) => {
                    map.set(ItemComponents::ITEM_MODEL, id.asset.into());
                }
                DataComponents::EquippableComp(component) => {
                    map.set(
                        ItemComponents::EQUIPPABLE,
                        EquippableComponent {
                            slot: match component.slot {
                                EquippableSlot::MainHand => EquipmentSlot::Mainhand,
                                EquippableSlot::Feet => EquipmentSlot::Boots,
                                EquippableSlot::Legs => EquipmentSlot::Leggings,
                                EquippableSlot::Chest => EquipmentSlot::Chestplate,
                                EquippableSlot::Head => EquipmentSlot::Helmet,
                                EquippableSlot::Offhand => EquipmentSlot::Offhand,
                                EquippableSlot::Body => EquipmentSlot::Body,
                                EquippableSlot::Saddle => EquipmentSlot::Saddle,
                            },
                            equip_sound: match component.equip_sound {
                                RegOr::Id(_) => Id::empty(),
                                RegOr::Or(event) => event.name.into(),
                            },
                            model: component.model.unwrap_or(Identifier::new("", "")).into(),
                        },
                    );
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
