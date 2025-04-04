use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use datafix::serialization::{
    Codec, CodecAdapters, CodecOps, Codecs, DefaultCodec, MapCodecBuilder,
};
use voxidian_protocol::{
    packet::PacketBuf,
    registry::RegEntry,
    value::{Nbt as PtcNbt, NbtElement},
};
use wyvern_actors::ActorResult;
use wyvern_datatypes::nbt::VxNbtOps;
use wyvern_values::IVec3;

use crate::{dimension::Dimension, runtime::Runtime};

use super::BlockState;

#[derive(Debug, Clone)]
pub struct Structure {
    pub size: IVec3,
    pub blocks: Vec<StructureBlock>,
    pub palette: Vec<BlockState>,
    pub entities: (),
    pub data_version: i32,
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

#[derive(Clone, Debug)]
pub struct StructureBlock {
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

pub struct StructureSplitter;

impl StructureSplitter {
    pub fn split_structure(structure: Structure, output_dir: &str, piece_size: IVec3) {
        let mut map = HashMap::<IVec3, Structure>::new();
        for block in structure.blocks {
            let offset_pos = block.pos.div_euclid(piece_size);
            map.entry(offset_pos)
                .or_insert_with(|| Structure {
                    size: piece_size,
                    blocks: Vec::new(),
                    palette: structure.palette.clone(),
                    entities: (),
                    data_version: 0,
                })
                .blocks
                .push(block);
        }

        for entry in map {
            let encoded = Structure::codec()
                .encode_start(&VxNbtOps, &entry.1)
                .unwrap();
            let NbtElement::Compound(encoded) = encoded else {
                continue;
            };

            let mut buf = PacketBuf::new();
            PtcNbt {
                name: String::new(),
                root: encoded,
            }
            .write_named(&mut buf)
            .unwrap();

            std::fs::write(
                format!("{output_dir}/{}.{}.{}.nbt", entry.0.x, entry.0.y, entry.0.z),
                buf.into_inner(),
            )
            .unwrap();
        }
    }

    pub fn place_split_structure(origin: IVec3, dimension: &Dimension, directory: &str) {
        let files = std::fs::read_dir(directory).unwrap();
        let idx = Arc::new(AtomicU32::new(0));
        for file in files {
            if let Ok(file) = file {
                let ft = file.file_type().unwrap();
                if ft.is_file() {
                    let dim = dimension.clone();
                    let origin = origin.clone();
                    let idx2 = idx.clone();
                    idx.fetch_add(1, Ordering::AcqRel);

                    Runtime::spawn_task(async move {
                        let bytes = std::fs::read(file.path()).unwrap();
                        let mut buf = PacketBuf::from(bytes);
                        let nbt = PtcNbt::read_named(&mut buf).unwrap();
                        let structure = Structure::codec()
                            .decode_start(&VxNbtOps, &NbtElement::Compound(nbt.root))
                            .unwrap();
                        structure.place_loading(dim, origin)?;
                        idx2.fetch_sub(1, Ordering::AcqRel);
                        Ok(())
                    });
                }
            }
        }

        Runtime::spawn_actor(
            move || {
                loop {
                    let v = idx.load(Ordering::SeqCst);
                    println!("{:#?}", v);
                    if v == 0 {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            },
            "Logger",
        );
    }
}
