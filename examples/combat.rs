use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState, Blocks},
    components::DataComponentHolder,
    datatypes::{particle::Particle, sound::Sounds, text::Text},
    entities::{AttributeContainer, Attributes, EntityComponents, PlayerSkinData},
    events::{
        DimensionCreateEvent, PlayerAttackEntityEvent, PlayerAttackPlayerEvent, PlayerJoinEvent,
        ServerStartEvent,
    },
    inventory::Inventory,
    item::{EquipmentSlot, EquippableComponent, ItemComponents, ItemStack, Items},
    player::PlayerComponents,
    runtime::Runtime,
    server::Server,
    values::{Id, Vec3, id},
};

const TEXTURE: &str = include_str!("./texture.txt");
const SIGNATURE: &str = include_str!("./signature.txt");
fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_attack)
        .event(on_attack_player)
        .registries(|registries| {
            registries.add_defaults();
        })
        .run();
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(example:root))?;

    Ok(())
}

fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    for x in -20..20 {
        for z in -20..20 {
            event.dimension.set_block(
                Vec3::new(x, 0, z),
                BlockState::new(Blocks::GRASS_BLOCK).with(BlockComponents::SNOWY, false),
            )?;
        }
    }

    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    event
        .player
        .set(PlayerComponents::POSITION, Vec3::new(0.0, 1.0, 0.0))?;

    event.player.inventory()?.set_slot(
        36,
        ItemStack::new(Items::DIAMOND_SWORD)
            .with(ItemComponents::ITEM_NAME, Text::literal("Diamond Sword")),
    )?;

    event.player.set(
        PlayerComponents::ATTRIBUTES,
        AttributeContainer::new()
            .with(Attributes::MAX_HEALTH, 30.0)
            .with(Attributes::ATTACK_SPEED, 100.0)
            .with(Attributes::FOLLOW_RANGE, 0.0)
            .with(Attributes::ENTITY_INTERACTION_RANGE, 20.0),
    )?;

    Runtime::spawn_task(move || {
        let entity = event
            .player
            .dimension()?
            .spawn_human_entity(PlayerSkinData {
                texture: TEXTURE.into(),
                signature: SIGNATURE.into(),
            })?;
        entity.set(EntityComponents::POSITION, Vec3::new(3.0, 10.0, 3.0))?;
        entity.set(EntityComponents::PHYSICS_ENABLED, true)?;
        entity.set(EntityComponents::GRAVITY_ENABLED, true)?;

        entity.set(
            EntityComponents::HELMET_ITEM,
            ItemStack::new(Items::GOLDEN_HELMET).with(
                ItemComponents::EQUIPPABLE,
                EquippableComponent {
                    slot: EquipmentSlot::Helmet,
                    equip_sound: Id::empty(),
                    model: Id::new("minecraft", "gold"),
                },
            ),
        )?;
        entity.set(
            EntityComponents::CHESTPLATE_ITEM,
            ItemStack::new(Items::CHAINMAIL_CHESTPLATE).with(
                ItemComponents::EQUIPPABLE,
                EquippableComponent {
                    slot: EquipmentSlot::Chestplate,
                    equip_sound: Id::empty(),
                    model: Id::new("minecraft", "chainmail"),
                },
            ),
        )?;
        entity.set(
            EntityComponents::LEGGINGS_ITEM,
            ItemStack::new(Items::DIAMOND_LEGGINGS).with(
                ItemComponents::EQUIPPABLE,
                EquippableComponent {
                    slot: EquipmentSlot::Leggings,
                    equip_sound: Id::empty(),
                    model: Id::new("minecraft", "chainmail"),
                },
            ),
        )?;
        entity.set(
            EntityComponents::BOOTS_ITEM,
            ItemStack::new(Items::DIAMOND_BOOTS).with(
                ItemComponents::EQUIPPABLE,
                EquippableComponent {
                    slot: EquipmentSlot::Boots,
                    equip_sound: Id::empty(),
                    model: Id::new("minecraft", "diamond"),
                },
            ),
        )?;
        entity.set(
            EntityComponents::MAINHAND_ITEM,
            ItemStack::new(id![minecraft:iron_axe]),
        )?;

        Ok(())
    });

    Ok(())
}

fn on_attack(event: Arc<PlayerAttackEntityEvent>) -> ActorResult<()> {
    let dir = event
        .attacker
        .get(PlayerComponents::DIRECTION)?
        .to_3d_direction()
        .map(|x| x / 2.0);

    event
        .victim
        .set(EntityComponents::VELOCITY, dir.with_y(0.3))?;

    event
        .attacker
        .play_sound(Sounds::ENTITY_PLAYER_ATTACK_CRIT)?;

    event.attacker.play_particle(
        event.victim.get(EntityComponents::POSITION)?,
        Particle::new(id![minecraft:crit]),
    )?;

    Ok(())
}

fn on_attack_player(event: Arc<PlayerAttackPlayerEvent>) -> ActorResult<()> {
    event
        .attacker
        .play_sound(Sounds::ENTITY_PLAYER_ATTACK_CRIT)?;

    event.victim.play_sound(Sounds::ENTITY_PLAYER_HURT)?;

    Ok(())
}
