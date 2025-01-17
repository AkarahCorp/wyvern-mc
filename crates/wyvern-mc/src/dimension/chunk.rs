use voxidian_protocol::{
    registry::RegEntry,
    value::{
        Biome, BlockState as ProtocolState, ChunkSection as ProtocolSection, Identifier,
        PaletteFormat, PalettedContainer,
    },
};

use crate::values::position::Position;

use super::blocks::BlockState;

pub struct ChunkSection {
    block_count: i16,
    blocks: [[[RegEntry<ProtocolState>; 16]; 16]; 16],
}

impl ChunkSection {
    pub fn empty() -> ChunkSection {
        ChunkSection {
            block_count: 0,
            blocks: std::array::from_fn(|_| {
                std::array::from_fn(|_| {
                    std::array::from_fn(|_| unsafe { RegEntry::new_unchecked(0) })
                })
            }),
        }
    }

    pub fn set_block_at(&mut self, pos: Position<usize>, block: BlockState) {
        self.blocks[*pos.x()][*pos.y()][*pos.z()] =
            unsafe { RegEntry::new_unchecked(block.protocol_id().try_into().unwrap()) };
    }

    pub fn flatten_blocks(&self) -> [RegEntry<ProtocolState>; 4096] {
        let mut arr = [unsafe { RegEntry::new_unchecked(0) }; 4096];
        let mut idx = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    arr[idx] = self.blocks[y][z][x];
                    idx += 1;
                }
            }
        }
        arr
    }

    pub fn into_protocol_section(&self) -> ProtocolSection {
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
                        .make_entry(&Identifier::new("minecraft", "plains"))
                        .unwrap(),
                },
            },
        }
    }
}
