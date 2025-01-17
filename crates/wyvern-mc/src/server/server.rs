use std::{io::{stdin, stdout}, sync::Arc};

use tokio::sync::{mpsc::Sender, oneshot::{self, Receiver}};
use voxidian_protocol::{registry::Registry, value::{Biome, DamageType, DimType, PaintingVariant, WolfVariant}};

use crate::systems::typemap::TypeMap;

use super::message::ServerMessage;

#[derive(Clone)]
pub struct Server {
    #[allow(dead_code)]
    pub(crate) sender: Sender<ServerMessage>
}

impl Server {
    pub async fn fire_systems(&self, parameters: TypeMap) {
        self.sender.send(ServerMessage::FireSystems(parameters)).await.unwrap();
    }

    pub async fn damage_types(&self) -> Arc<Registry<DamageType>> {
        let (tx, mut rx) = oneshot::channel();
        self.sender.send(ServerMessage::DamageTypeRegistry(tx)).await.unwrap();
        poll_receiver(&mut rx).await
    }

    pub async fn biomes(&self) -> Arc<Registry<Biome>> {
        let (tx, mut rx) = oneshot::channel();
        self.sender.send(ServerMessage::BiomeRegistry(tx)).await.unwrap();
        poll_receiver(&mut rx).await
    }

    pub async fn wolf_variants(&self) -> Arc<Registry<WolfVariant>> {
        let (tx, mut rx) = oneshot::channel();
        self.sender.send(ServerMessage::WolfRegistry(tx)).await.unwrap();
        poll_receiver(&mut rx).await
    }

    pub async fn dimension_types(&self) -> Arc<Registry<DimType>> {
        let (tx, mut rx) = oneshot::channel();
        self.sender.send(ServerMessage::DimTypeRegistry(tx)).await.unwrap();
        poll_receiver(&mut rx).await
    }

    pub async fn painting_variants(&self) -> Arc<Registry<PaintingVariant>> {
        let (tx, mut rx) = oneshot::channel();
        self.sender.send(ServerMessage::PaintingRegistry(tx)).await.unwrap();
        poll_receiver(&mut rx).await
    }

    
}

pub(crate) async fn poll_receiver<T>(rx: &mut Receiver<T>) -> T {
    loop {
        match rx.try_recv() {
            Ok(v) => return v,
            Err(e) => {
                tokio::task::yield_now().await;
            },
        }
    }
}