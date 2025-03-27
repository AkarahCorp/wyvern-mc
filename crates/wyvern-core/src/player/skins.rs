use std::str::FromStr;

use datafix::serialization::{
    Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder, json::JsonOps,
};
use wyvern_values::Uuid;

use crate::entities::PlayerSkinData;

use super::Player;

struct UuidToUsernameResponse {
    uuid: Uuid,
    username: String,
}

impl<O: CodecOps> DefaultCodec<O> for UuidToUsernameResponse {
    fn codec() -> impl datafix::serialization::Codec<Self, O> {
        MapCodecBuilder::new()
            .field(
                String::codec()
                    .xmap(
                        |x| Uuid::from_str(&x).unwrap_or(Uuid::nil()),
                        |x| x.to_string(),
                    )
                    .field_of("id", |x: &UuidToUsernameResponse| &x.uuid),
            )
            .field(String::codec().field_of("name", |x: &UuidToUsernameResponse| &x.username))
            .build(|uuid, username| UuidToUsernameResponse { uuid, username })
    }
}

struct ProfileResponse {
    uuid: Uuid,
    username: String,
    properties: Vec<ProfileProperty>,
    profile_actions: (),
}

impl<O: CodecOps> DefaultCodec<O> for ProfileResponse {
    fn codec() -> impl Codec<Self, O> {
        MapCodecBuilder::new()
            .field(
                String::codec()
                    .xmap(
                        |x| Uuid::from_str(&x).unwrap_or(Uuid::nil()),
                        |x| x.to_string(),
                    )
                    .field_of("id", |x: &ProfileResponse| &x.uuid),
            )
            .field(String::codec().field_of("name", |x: &ProfileResponse| &x.username))
            .field(
                ProfileProperty::codec()
                    .list_of()
                    .field_of("properties", |x: &ProfileResponse| &x.properties),
            )
            .field(
                Codecs::unit().field_of("profileActions", |x: &ProfileResponse| &x.profile_actions),
            )
            .build(
                |uuid, username, properties, profile_actions| ProfileResponse {
                    uuid,
                    username,
                    properties,
                    profile_actions,
                },
            )
    }
}

struct ProfileProperty {
    name: String,
    value: String,
    signature: String,
}

impl<O: CodecOps> DefaultCodec<O> for ProfileProperty {
    fn codec() -> impl Codec<Self, O> {
        MapCodecBuilder::new()
            .field(String::codec().field_of("name", |x: &ProfileProperty| &x.name))
            .field(String::codec().field_of("value", |x: &ProfileProperty| &x.value))
            .field(String::codec().field_of("signature", |x: &ProfileProperty| &x.signature))
            .build(|name, value, signature| ProfileProperty {
                name,
                value,
                signature,
            })
    }
}

impl Player {
    pub fn uuid_to_username(username: &str) -> Uuid {
        let mut req = ureq::get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{username}"
        ))
        .call()
        .unwrap();
        let resp = req.body_mut().read_to_string().unwrap();
        let json = json::parse(&resp).unwrap();
        log::error!("{:#?}", json);
        let value = UuidToUsernameResponse::codec()
            .decode_start(&JsonOps, &json)
            .unwrap();
        value.uuid
    }

    pub fn get_skin_for_uuid(uuid: &Uuid) -> PlayerSkinData {
        let mut req = ureq::get(format!(
            "https://sessionserver.mojang.com/session/minecraft/profile/{uuid}?unsigned=false"
        ))
        .call()
        .unwrap();
        let resp = req.body_mut().read_to_string().unwrap();
        let json = json::parse(&resp).unwrap();
        let value = ProfileResponse::codec()
            .decode_start(&JsonOps, &json)
            .unwrap();
        PlayerSkinData {
            texture: value.properties[0].value.clone(),
            signature: value.properties[0].signature.clone(),
        }
    }
}
