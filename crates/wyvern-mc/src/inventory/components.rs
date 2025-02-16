use voxidian_protocol::value::{Damage, DataComponentTypes, DataComponents, ItemModel, MaxDamage};

use crate::{
    components::{ComponentKind, ComponentRegistry},
    values::{Key, resource::Texture},
};

use super::ItemStack;

pub struct ItemComponents;

impl ItemComponents {
    pub const ITEM_COUNT: ItemCountComponentType = ItemCountComponentType;
    pub const MAX_DAMAGE: MaxDamageComponentType = MaxDamageComponentType;
    pub const DAMAGE: DamageComponentType = DamageComponentType;
    pub const ITEM_MODEL: ItemModelComponentType = ItemModelComponentType;
}

impl ComponentRegistry<ItemStack> for ItemComponents {}

pub struct ItemCountComponentType;
impl ComponentKind<ItemStack, ItemComponents, u16> for ItemCountComponentType {
    fn insert_component(&self, holder: &mut ItemStack, value: u16) {
        holder.count = value;
    }

    fn get_component(&self, holder: &ItemStack) -> Option<u16> {
        Some(holder.count)
    }
}

pub struct MaxDamageComponentType;
impl ComponentKind<ItemStack, ItemComponents, u32> for MaxDamageComponentType {
    fn insert_component(&self, holder: &mut ItemStack, value: u32) {
        holder.added_components.insert(
            DataComponentTypes::MaxDamage,
            DataComponents::MaxDamage(MaxDamage {
                amount: (value as i32).into(),
            }),
        );
        holder
            .removed_components
            .remove(&DataComponentTypes::MaxDamage);
    }

    fn get_component(&self, holder: &ItemStack) -> Option<u32> {
        holder
            .added_components
            .get(&DataComponentTypes::MaxDamage)
            .map(|value| {
                let DataComponents::MaxDamage(value) = value else {
                    unreachable!()
                };
                value.amount.as_i32() as u32
            })
    }
}
pub struct DamageComponentType;
impl ComponentKind<ItemStack, ItemComponents, u32> for DamageComponentType {
    fn insert_component(&self, holder: &mut ItemStack, value: u32) {
        holder.added_components.insert(
            DataComponentTypes::Damage,
            DataComponents::Damage(Damage {
                damage: (value as i32).into(),
            }),
        );
        holder
            .removed_components
            .remove(&DataComponentTypes::Damage);
    }

    fn get_component(&self, holder: &ItemStack) -> Option<u32> {
        holder
            .added_components
            .get(&DataComponentTypes::Damage)
            .map(|value| {
                let DataComponents::Damage(value) = value else {
                    unreachable!()
                };
                value.damage.as_i32() as u32
            })
    }
}

pub struct ItemModelComponentType;
impl ComponentKind<ItemStack, ItemComponents, Key<Texture>> for ItemModelComponentType {
    fn insert_component(&self, holder: &mut ItemStack, value: Key<Texture>) {
        holder.added_components.insert(
            DataComponentTypes::ItemModel,
            DataComponents::ItemModel(ItemModel {
                asset: value.into(),
            }),
        );
        holder
            .removed_components
            .remove(&DataComponentTypes::ItemModel);
    }

    fn get_component(&self, holder: &ItemStack) -> Option<Key<Texture>> {
        holder
            .added_components
            .get(&DataComponentTypes::ItemModel)
            .map(|value| {
                let DataComponents::ItemModel(value) = value else {
                    unreachable!()
                };
                value.asset.clone().into()
            })
    }
}
