use datafix::serialization::{
    Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder,
};
use voxidian_protocol::registry::RegEntry;
use wyvern_actors::ActorResult;
use wyvern_values::IVec3;

use crate::dimension::Dimension;

use super::BlockState;

pub struct Structure {
    size: IVec3,
    blocks: Vec<StructureBlock>,
    palette: Vec<BlockState>,
    entities: (),
    data_version: i32,
}

impl<O: CodecOps> DefaultCodec<O> for Structure {
    fn codec() -> impl Codec<Self, O> {
        MapCodecBuilder::new()
            .field(ivec3_codec().field_of("size", |s: &Structure| &s.size))
            .field(
                StructureBlock::codec()
                    .list_of()
                    .field_of("blocks", |s: &Structure| &s.blocks),
            )
            .field(
                BlockState::codec()
                    .list_of()
                    .field_of("palette", |s: &Structure| &s.palette),
            )
            .field(Codecs::unit().default_field_of("entities", |s: &Structure| &s.entities, || ()))
            .field(i32::codec().field_of("DataVersion", |s: &Structure| &s.data_version))
            .build(|size, blocks, palette, entities, data_version| Structure {
                size,
                blocks,
                palette,
                entities,
                data_version,
            })
    }
}
struct StructureBlock {
    pos: IVec3,
    state: i32,
    nbt: (),
}

impl<O: CodecOps> DefaultCodec<O> for StructureBlock {
    fn codec() -> impl datafix::serialization::Codec<Self, O> {
        MapCodecBuilder::new()
            .field(ivec3_codec().field_of("pos", |s: &StructureBlock| &s.pos))
            .field(i32::codec().field_of("state", |s: &StructureBlock| &s.state))
            .field(Codecs::unit().default_field_of("nbt", |s: &StructureBlock| &s.nbt, || ()))
            .build(|pos, state, nbt| StructureBlock { pos, state, nbt })
    }
}

impl Structure {
    pub fn place(&self, dim: Dimension, base_position: IVec3) -> ActorResult<()> {
        for block in &self.blocks {
            let new_pos = base_position + block.pos;
            let block_state = &self.palette[block.state as usize];
            dim.set_block(new_pos, block_state.clone())?;
        }
        Ok(())
    }

    pub fn place_loading(&self, dim: Dimension, base_position: IVec3) -> ActorResult<()> {
        for block in &self.blocks {
            let new_pos = base_position + block.pos;
            let block_state = &self.palette[block.state as usize];
            let block_state_id =
                unsafe { RegEntry::<BlockState>::new_unchecked(block_state.protocol_id() as u32) }
                    .id();
            dim.set_block_loading(new_pos, block_state_id)?;
        }
        Ok(())
    }
}

fn ivec3_codec<O: CodecOps>() -> impl Codec<IVec3, O> {
    i32::codec().list_of().xmap(
        |vec| IVec3::new(vec[0], vec[1], vec[2]),
        |ivec| Vec::from(ivec.to_array()),
    )
}
