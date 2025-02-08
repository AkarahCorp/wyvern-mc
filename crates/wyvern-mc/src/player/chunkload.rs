use voxidian_protocol::{
    packet::s2c::play::{
        ChunkBatchFinishedS2CPlayPacket, ChunkBatchStartS2CPlayPacket,
        LevelChunkWithLightS2CPlayPacket, SetChunkCacheCenterS2CPlayPacket,
    },
    value::{ChunkSectionData, Nbt, NbtCompound, VarInt},
};

use crate::values::{Vec2, Vec3};

use super::ConnectionData;

impl ConnectionData {
    pub async fn send_chunks(&mut self) {
        let Some(dimension) = &self.associated_data.dimension else {
            return;
        };

        let chunk_center = Vec2::new(
            f64::floor(self.associated_data.last_position.x() / 16.0) as i32,
            f64::floor(self.associated_data.last_position.z() / 16.0) as i32,
        );

        self.associated_data.last_chunk_position = chunk_center.clone();

        let cx = chunk_center.x().clone();
        let cz = chunk_center.y().clone();

        let render_distance = (self.associated_data.render_distance / 2) + 2;

        *&mut self.associated_data.loaded_chunks = self
            .associated_data
            .loaded_chunks
            .iter()
            .filter(|position| {
                position.x() >= cx - render_distance
                    && position.x() <= cx + render_distance
                    && position.y() >= cz - render_distance
                    && position.y() <= cz + render_distance
            })
            .map(|x| *x)
            .collect::<Vec<_>>();

        let dim_reg = &self.connected_server.registries().await.dimension_types;

        let dim_type = dim_reg
            .get(&dimension.get_dimension_type().await.into())
            .unwrap();

        let mut chunks = Vec::new();
        for chunk_x in (cx - render_distance)..(cx + render_distance) {
            for chunk_z in (cz - render_distance)..(cz + render_distance) {
                let pos = Vec2::new(chunk_x, chunk_z);
                if !self.associated_data.loaded_chunks.contains(&pos) {
                    chunks.push(pos);
                }
            }
        }

        chunks.sort_by(|lhs, rhs| {
            let lhs_dist = i32::isqrt(i32::pow(lhs.x() - cx, 2) + i32::pow(lhs.y() - cz, 2));
            let rhs_dist = i32::isqrt(i32::pow(rhs.x() - cx, 2) + i32::pow(rhs.y() - cz, 2));
            lhs_dist.cmp(&rhs_dist)
        });

        if let Some(pos) = chunks.first() {
            let chunk_x = pos.x();
            let chunk_z = pos.y();

            let mut sections = Vec::new();
            for y in (dim_type.min_y..dim_type.max_y).step_by(16) {
                let pos = Vec3::new(chunk_x, y, chunk_z);
                let chunk = dimension.get_chunk_section(pos).await;
                sections.push(chunk.into_protocol_section());
            }

            let packet = LevelChunkWithLightS2CPlayPacket {
                chunk_x,
                chunk_z,
                heightmaps: Nbt {
                    name: "".to_string(),
                    root: NbtCompound::new(),
                },
                data: ChunkSectionData { sections },
                block_entities: vec![].into(),
                sky_light_mask: vec![0].into(),
                block_light_mask: vec![0].into(),
                empty_sky_light_mask: vec![0].into(),
                empty_block_light_mask: vec![0].into(),
                sky_light_array: vec![].into(),
                block_light_array: vec![].into(),
            };

            self.associated_data.loaded_chunks.push(pos.clone());

            self.write_packet(SetChunkCacheCenterS2CPlayPacket {
                chunk_x: chunk_center.x().clone().into(),
                chunk_z: chunk_center.y().clone().into(),
            })
            .await;
            self.write_packet(ChunkBatchStartS2CPlayPacket {}).await;
            self.write_packet(packet).await;
            self.write_packet(ChunkBatchFinishedS2CPlayPacket {
                size: VarInt::from(1),
            })
            .await;
        }
    }
}
