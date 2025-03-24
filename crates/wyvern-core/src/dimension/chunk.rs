use std::collections::HashMap;

use voxidian_protocol::{
    registry::RegEntry,
    value::{ChunkSection as ProtocolSection, PaletteFormat, PalettedContainer, RawDataArray},
};
use wyvern_components::DataComponentHolder;
use wyvern_datatypes::nbt::Nbt;

use crate::{
    blocks::BlockComponents,
    server::{Server, registries::RegistryKeys},
};

use wyvern_values::{Id, Vec3};

use crate::blocks::BlockState;

#[derive(Clone, Debug)]
pub struct Chunk {
    min_sections: i32,
    _max_sections: i32,
    sections: Vec<ChunkSection>,
}

impl Chunk {
    pub fn new(min_sections: i32, max_sections: i32) -> Chunk {
        let total_sections = max_sections + -min_sections;
        let mut vec = Vec::with_capacity(total_sections as usize);
        for _ in 0..total_sections {
            vec.push(ChunkSection::empty());
        }
        Chunk {
            min_sections,
            _max_sections: max_sections,
            sections: vec,
        }
    }

    pub(crate) fn section_at_mut(&mut self, section: i32) -> Option<&mut ChunkSection> {
        self.sections
            .get_mut((section + -self.min_sections) as usize)
    }

    pub fn set_block_at(&mut self, pos: Vec3<i32>, block: BlockState) {
        let section_y = pos.y().div_euclid(16);
        let local_y = pos.y().rem_euclid(16);
        if let Some(section) = self.section_at_mut(section_y) {
            section.set_block_at(pos.map(|x| x as usize).with_y(local_y as usize), block);
        }
    }

    pub fn get_block_at(&mut self, pos: Vec3<i32>) -> BlockState {
        let section_y = pos.y().div_euclid(16); // Use Euclidean division
        let local_y = pos.y().rem_euclid(16); // Always positive in [0, 15]

        if let Some(section) = self.section_at_mut(section_y) {
            section.get_block_at(pos.map(|x| x as usize).with_y(local_y as usize))
        } else {
            BlockState::from_protocol_id(0)
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ChunkSection {
    block_count: i16,
    blocks: RawDataArray,
    block_meta: HashMap<Vec3<usize>, Nbt>,
}

impl ChunkSection {
    pub fn index_from_pos(pos: Vec3<usize>) -> usize {
        pos.y() * 256 + pos.z() * 16 + pos.x()
    }

    pub fn empty() -> ChunkSection {
        ChunkSection {
            block_count: 0,
            blocks: {
                let mut arr = RawDataArray::new(15);
                for _ in 0..4096 {
                    arr.push(0);
                }
                arr
            },
            block_meta: HashMap::new(),
        }
    }

    pub fn set_block_at(&mut self, pos: Vec3<usize>, block: BlockState) {
        let idx = Self::index_from_pos(pos);
        let old_block = self.blocks.get(idx).unwrap();

        let new_block =
            unsafe { RegEntry::<BlockState>::new_unchecked(block.clone().protocol_id() as u32) }
                .id();

        if old_block == 0 && new_block != 0 {
            self.block_count += 1;
        } else if old_block != 0 && new_block == 0 {
            self.block_count -= 1;
        }

        self.blocks.set(idx, new_block as u64);
        if let Ok(data) = block.get(BlockComponents::CUSTOM_DATA) {
            self.block_meta.insert(pos, data);
        }
    }

    pub fn get_block_at(&mut self, pos: Vec3<usize>) -> BlockState {
        let ptc = self.blocks.get(Self::index_from_pos(pos)).unwrap();
        let mut state = BlockState::from_protocol_id(ptc as i32);

        if let Some(cdata) = self.block_meta.get(&pos) {
            state.set(BlockComponents::CUSTOM_DATA, cdata.clone());
        }

        state
    }

    pub fn as_protocol_section(&self) -> ProtocolSection {
        ProtocolSection {
            block_count: self.block_count,
            block_states: PalettedContainer {
                bits_per_entry: 15,
                format: PaletteFormat::RawDirect {
                    data: self.blocks.clone(),
                },
            },
            biomes: PalettedContainer {
                bits_per_entry: 0,
                format: PaletteFormat::SingleValued {
                    entry: Server::get()
                        .unwrap()
                        .registries()
                        .unwrap()
                        .get(RegistryKeys::BIOME)
                        .get_entry(Id::new("minecraft", "plains"))
                        .unwrap(),
                },
            },
        }
    }
}
