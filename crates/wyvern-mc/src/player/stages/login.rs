use voxidian_protocol::{
    mojang::auth_verify::{MojAuth, MojAuthError},
    packet::{
        Stage,
        c2s::login::C2SLoginPackets,
        processing::{CompressionMode, SecretCipher, generate_key_pair},
        s2c::{
            config::{KnownPack, SelectKnownPacksS2CConfigPacket},
            login::{
                HelloS2CLoginPacket, LoginCompressionS2CLoginPacket, LoginFinishedS2CLoginPacket,
            },
        },
    },
    value::{LengthPrefixHashMap, VarInt},
};

use crate::{
    actors::{ActorError, ActorResult},
    player::{ConnectionData, MojauthData},
    server::Server,
};

impl ConnectionData {
    pub fn login_stage(&mut self) -> ActorResult<()> {
        self.read_packets(|packet: C2SLoginPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            match packet {
                C2SLoginPackets::CustomQueryAnswer(_packet) => todo!(),
                C2SLoginPackets::LoginAcknowledged(_packet) => {
                    *this.stage.lock().unwrap() = Stage::Config;
                    this.write_packet(SelectKnownPacksS2CConfigPacket {
                        known_packs: vec![KnownPack {
                            namespace: "minecraft".to_string(),
                            id: "core".to_string(),
                            version: "1.21.4".to_string(),
                        }]
                        .into(),
                    });
                }
                C2SLoginPackets::Key(packet) => {
                    let Ok(decrypted_verify_token) = this
                        .mojauth
                        .as_ref()
                        .ok_or(ActorError::ActorIsNotLoaded)?
                        .private_key
                        .as_ref()
                        .unwrap()
                        .decrypt(packet.verify_token.as_slice())
                    else {
                        return Err(ActorError::ActorDoesNotExist);
                    };
                    if decrypted_verify_token
                        != this
                            .mojauth
                            .as_ref()
                            .ok_or(ActorError::ActorIsNotLoaded)?
                            .verify_token
                            .as_slice()
                    {
                        return Err(ActorError::ActorDoesNotExist);
                    }

                    let Ok(secret_key) = this
                        .mojauth
                        .as_ref()
                        .ok_or(ActorError::ActorIsNotLoaded)?
                        .private_key
                        .as_ref()
                        .unwrap()
                        .decrypt(packet.secret_key.as_slice())
                    else {
                        return Err(ActorError::ActorDoesNotExist);
                    };

                    let secret_cipher = SecretCipher::from_key_bytes(&secret_key);
                    this.packet_processing.secret_cipher = secret_cipher;

                    let mojauth = match MojAuth::start_blocking(
                        None,
                        this.associated_data.username.clone(),
                        "WyvernMC",
                        this.packet_processing.secret_cipher.key().unwrap(),
                        this.mojauth
                            .as_ref()
                            .ok_or(ActorError::ActorIsNotLoaded)?
                            .public_key
                            .as_ref()
                            .unwrap(),
                    ) {
                        Ok(mojauth) => mojauth,
                        Err(err) => {
                            return Err(match err {
                                MojAuthError::AuthServerDown => ActorError::ActorDoesNotExist,
                                MojAuthError::InvalidData => ActorError::ActorDoesNotExist,
                                MojAuthError::Unverified => ActorError::ActorDoesNotExist,
                            });
                        }
                    };

                    this.associated_data.username = mojauth.name;
                    this.associated_data.uuid = mojauth.uuid;
                    this.mojauth
                        .as_mut()
                        .ok_or(ActorError::ActorIsNotLoaded)?
                        .props = mojauth.props;

                    this.write_packet(LoginFinishedS2CLoginPacket {
                        uuid: this.associated_data.uuid,
                        username: this.associated_data.username.clone(),
                        props: LengthPrefixHashMap::new(),
                    });
                }
                C2SLoginPackets::Hello(packet) => {
                    this.write_packet(LoginCompressionS2CLoginPacket {
                        threshold: VarInt::from(128),
                    });
                    this.packet_processing.compression = CompressionMode::ZLib { threshold: 128 };

                    this.associated_data.username = packet.username.clone();
                    this.associated_data.uuid = packet.uuid;

                    if Server::get()?.mojauth_enabled()? {
                        this.mojauth = Some(MojauthData {
                            private_key: None,
                            verify_token: Vec::new(),
                            public_key: None,
                            props: Vec::new(),
                        });

                        let (private, public) = generate_key_pair::<1024>();
                        let verify_token =
                            std::array::from_fn::<_, 4, _>(|_| rand::random::<u8>()).to_vec();

                        this.mojauth
                            .as_mut()
                            .ok_or(ActorError::ActorIsNotLoaded)?
                            .private_key = Some(private);
                        this.mojauth
                            .as_mut()
                            .ok_or(ActorError::ActorIsNotLoaded)?
                            .public_key = Some(public);
                        this.mojauth
                            .as_mut()
                            .ok_or(ActorError::ActorIsNotLoaded)?
                            .verify_token = verify_token;

                        this.write_packet(HelloS2CLoginPacket {
                            server_id: "WyvernMC".to_string(),
                            public_key: this
                                .mojauth
                                .as_ref()
                                .ok_or(ActorError::ActorIsNotLoaded)?
                                .public_key
                                .as_ref()
                                .unwrap()
                                .der_bytes()
                                .into(),
                            verify_token: this
                                .mojauth
                                .as_ref()
                                .ok_or(ActorError::ActorIsNotLoaded)?
                                .verify_token
                                .clone()
                                .into(),
                            should_auth: true,
                        });
                    } else {
                        this.write_packet(LoginFinishedS2CLoginPacket {
                            uuid: this.associated_data.uuid,
                            username: this.associated_data.username.clone(),
                            props: LengthPrefixHashMap::new(),
                        });
                    }
                }
                C2SLoginPackets::CookieResponse(_packet) => todo!(),
            }

            Ok(())
        })
    }
}
