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
    player::ConnectionData,
};

impl ConnectionData {
    pub async fn login_stage(&mut self) -> ActorResult<()> {
        self.read_packets(async |packet: C2SLoginPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            match packet {
                C2SLoginPackets::CustomQueryAnswer(_packet) => todo!(),
                C2SLoginPackets::LoginAcknowledged(_packet) => {
                    log::error!("Prepi for cif 2");

                    *this.stage.lock().unwrap() = Stage::Config;
                    this.write_packet(SelectKnownPacksS2CConfigPacket {
                        known_packs: vec![KnownPack {
                            namespace: "minecraft".to_string(),
                            id: "core".to_string(),
                            version: "1.21.4".to_string(),
                        }]
                        .into(),
                    })
                    .await;
                }
                C2SLoginPackets::Key(packet) => {
                    let Ok(decrypted_verify_token) = this
                        .private_key
                        .as_ref()
                        .unwrap()
                        .decrypt(packet.verify_token.as_slice())
                    else {
                        log::error!("a");
                        return Err(ActorError::ActorDoesNotExist);
                    };
                    if decrypted_verify_token != this.verify_token.as_slice() {
                        log::error!("b");
                        return Err(ActorError::ActorDoesNotExist);
                    }

                    let Ok(secret_key) = this
                        .private_key
                        .as_ref()
                        .unwrap()
                        .decrypt(packet.secret_key.as_slice())
                    else {
                        log::error!("c");
                        return Err(ActorError::ActorDoesNotExist);
                    };

                    log::error!("d");

                    let secret_cipher = SecretCipher::from_key_bytes(&secret_key);
                    this.packet_processing.secret_cipher = secret_cipher;

                    let mojauth = match MojAuth::start(
                        None,
                        this.associated_data.username.clone(),
                        "WyvernMC".to_string(),
                        this.packet_processing.secret_cipher.key().unwrap(),
                        this.public_key.as_ref().unwrap(),
                    )
                    .await
                    {
                        Ok(mojauth) => mojauth,
                        Err(err) => {
                            return Err(match err {
                                MojAuthError::AuthServerDown => ActorError::ActorDoesNotExist,
                                MojAuthError::InvalidData => ActorError::ActorDoesNotExist,
                                MojAuthError::Unverified => ActorError::ActorDoesNotExist,
                            });
                        }
                    };
                    log::error!("e {:?}", mojauth);

                    this.associated_data.username = mojauth.name;
                    this.associated_data.uuid = mojauth.uuid;
                    this.props = mojauth.props;

                    this.write_packet(LoginFinishedS2CLoginPacket {
                        uuid: this.associated_data.uuid,
                        username: this.associated_data.username.clone(),
                        props: LengthPrefixHashMap::new(),
                    })
                    .await;

                    log::error!("Prepi for cif ");
                }
                C2SLoginPackets::Hello(packet) => {
                    this.write_packet(LoginCompressionS2CLoginPacket {
                        threshold: VarInt::from(128),
                    })
                    .await;
                    this.packet_processing.compression = CompressionMode::ZLib { threshold: 128 };

                    log::error!("0");
                    this.associated_data.username = packet.username.clone();
                    this.associated_data.uuid = packet.uuid;

                    log::error!("1");

                    let (private, public) = generate_key_pair::<1024>();
                    let verify_token =
                        std::array::from_fn::<_, 4, _>(|_| rand::random::<u8>()).to_vec();

                    log::error!("2");
                    this.private_key = Some(private);
                    this.public_key = Some(public);
                    this.verify_token = verify_token;

                    log::error!("3");
                    this.write_packet(HelloS2CLoginPacket {
                        server_id: "WyvernMC".to_string(),
                        public_key: this.public_key.as_ref().unwrap().der_bytes().into(),
                        verify_token: this.verify_token.clone().into(),
                        should_auth: true,
                    })
                    .await;
                    log::error!("4");
                }
                C2SLoginPackets::CookieResponse(_packet) => todo!(),
            }

            Ok(())
        })
        .await
    }
}
