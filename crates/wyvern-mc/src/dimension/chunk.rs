use std::mem::MaybeUninit;

use voxidian_protocol::{
    registry::RegEntry,
    value::{
        Biome, BlockState as ProtocolState, ChunkSection as ProtocolSection, Identifier,
        PaletteFormat, PalettedContainer,
    },
};

use crate::values::Vec3;

use super::blocks::BlockState;

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
            section.set_block_at(
                Vec3::new(pos.x() as usize, local_y as usize, pos.z() as usize),
                block,
            );
        }
    }

    pub fn get_block_at(&mut self, pos: Vec3<i32>) -> BlockState {
        let section_y = pos.y().div_euclid(16); // Use Euclidean division
        let local_y = pos.y().rem_euclid(16); // Always positive in [0, 15]

        if let Some(section) = self.section_at_mut(section_y) {
            section.get_block_at(Vec3::new(
                pos.x() as usize,
                local_y as usize,
                pos.z() as usize,
            ))
        } else {
            BlockState::from_protocol_id(0)
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ChunkSection {
    block_count: i16,
    blocks: [[[ChunkBlock; 16]; 16]; 16],
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct ChunkBlock {
    block_state: u16,
    #[allow(unused)]
    block_meta: u16,
}

impl ChunkBlock {
    pub fn air() -> ChunkBlock {
        ChunkBlock {
            block_state: 0,
            block_meta: 0,
        }
    }

    pub fn id(&self) -> u16 {
        self.block_state
    }
}

impl ChunkSection {
    pub fn empty() -> ChunkSection {
        ChunkSection {
            block_count: 0,
            blocks: std::array::from_fn(|_| {
                std::array::from_fn(|_| std::array::from_fn(|_| ChunkBlock::air()))
            }),
        }
    }

    pub fn set_block_at(&mut self, pos: Vec3<usize>, block: BlockState) {
        let old_block = self.blocks[pos.x()][pos.y()][pos.z()];

        let new_block: RegEntry<BlockState> =
            unsafe { RegEntry::new_unchecked(block.clone().protocol_id() as u32) };

        if old_block.id() == 0 && new_block.id() != 0 {
            self.block_count += 1;
        } else if old_block.id() != 0 && new_block.id() == 0 {
            self.block_count -= 1;
        }

        self.blocks[pos.x()][pos.y()][pos.z()] = ChunkBlock {
            block_state: new_block.id() as u16,
            block_meta: 0,
        };
    }

    pub fn get_block_at(&mut self, pos: Vec3<usize>) -> BlockState {
        let ptc = self.blocks[pos.x()][pos.y()][pos.z()];
        BlockState::from_protocol_id(ptc.id() as i32)
    }

    pub fn flatten_blocks(&self) -> [RegEntry<ProtocolState>; 4096] {
        // SAFETY: We are only setting values in `arr`, and we already assume underlying values aren't initialized, meaning this is safe.
        let mut arr: [MaybeUninit<RegEntry<ProtocolState>>; 4096] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let mut idx = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    // SAFETY: This is safe since the underlying memory isn't initialized and writable.
                    arr[idx] = unsafe {
                        MaybeUninit::new(RegEntry::new_unchecked(
                            self.blocks[x][y][z].block_state as u32,
                        ))
                    };
                    idx += 1;
                }
            }
        }
        // SAFETY: `arr` is never transmuted back until all elements are set,
        // allowing this to work without UB
        unsafe { std::mem::transmute(arr) }
    }

    pub fn as_protocol_section(&self) -> ProtocolSection {
        ProtocolSection {
            block_count: self.block_count,
            block_states: PalettedContainer {
                bits_per_entry: 15,
                format: PaletteFormat::Direct {
                    data: self.flatten_blocks(),
                },
            },
            biomes: PalettedContainer {
                bits_per_entry: 0,
                format: PaletteFormat::SingleValued {
                    entry: Biome::vanilla_registry()
                        .get_entry(&Identifier::new("minecraft", "plains"))
                        .unwrap(),
                },
            },
        }
    }
}
