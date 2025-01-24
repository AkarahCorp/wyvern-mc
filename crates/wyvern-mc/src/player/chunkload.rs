use voxidian_protocol::{
    packet::s2c::play::{
        ChunkBatchFinishedS2CPlayPacket, ChunkBatchStartS2CPlayPacket,
        LevelChunkWithLightS2CPlayPacket, SetChunkCacheCenterS2CPlayPacket,
    },
    registry::RegEntry,
    value::{
        ChunkSection, ChunkSectionData, Nbt, NbtCompound, PaletteFormat, PalettedContainer, VarInt,
    },
};

use crate::values::Position;

use super::net::ConnectionData;

impl ConnectionData {
    pub async fn send_chunks(&mut self) {
        let Some(dimension) = &self.associated_data.dimension else {
            return;
        };

        let chunk_center = self
            .associated_data
            .last_position
            .map_into_coords(|x| f64::floor(x / 16.0) as i32);

        if self.associated_data.last_chunk_position == chunk_center
            && *self.associated_data.last_chunk_position.y() == 0
        {
            return;
        }

        self.associated_data.last_chunk_position = chunk_center.clone();
        self.write_packet(SetChunkCacheCenterS2CPlayPacket {
            chunk_x: chunk_center.x().clone().into(),
            chunk_z: chunk_center.z().clone().into(),
        })
        .await;

        let cx = chunk_center.x().clone();
        let cz = chunk_center.z().clone();

        let render_distance = (self.associated_data.render_distance / 2) + 2;

        *&mut self.associated_data.loaded_chunks = self
            .associated_data
            .loaded_chunks
            .iter()
            .filter(|position| {
                *position.x() > cx - render_distance
                    && *position.x() < cx + render_distance
                    && *position.z() > cz - render_distance
                    && *position.z() < cz + render_distance
            })
            .map(|x| *x)
            .collect::<Vec<_>>();

        let dim_reg = self.connected_server.dimension_types().await;
        let dim_type = dim_reg
            .get(&dimension.get_dimension_type().await.into())
            .unwrap();

        self.write_packet(ChunkBatchStartS2CPlayPacket {}).await;
        let mut chunks = 0;
        for chunk_x in (cx - render_distance)..(cx + render_distance) {
            for chunk_z in (cz - render_distance)..(cz + render_distance) {
                println!("x: {:?}, z: {:?}", chunk_x, chunk_z);
                let pos = Position::new(chunk_x, 0, chunk_z);
                if !self.associated_data.loaded_chunks.contains(&pos) {
                    let mut sections = Vec::new();
                    for y in (dim_type.min_y..dim_type.max_y).step_by(16) {
                        let pos = Position::new(chunk_x, y, chunk_z);
                        let chunk = dimension.get_chunk_at(pos).await;
                        sections.push(chunk.into_protocol_section());
                    }

                    println!("Sent chunk pos: {:?} @ {:?} sections", pos, sections.len());

                    self.write_packet(LevelChunkWithLightS2CPlayPacket {
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
                    })
                    .await;
                    chunks += 1;

                    self.associated_data.loaded_chunks.push(pos);
                }
            }
        }

        self.write_packet(ChunkBatchFinishedS2CPlayPacket {
            size: VarInt::from(chunks),
        })
        .await;
    }
}
