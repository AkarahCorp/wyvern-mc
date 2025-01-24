use voxidian_protocol::{
    registry::RegEntry,
    value::{
        Biome, BlockState as ProtocolState, ChunkSection as ProtocolSection, Identifier,
        PaletteFormat, PalettedContainer,
    },
};

use crate::values::Position;

use super::blocks::BlockState;

#[derive(Clone)]
pub struct ChunkSection {
    block_count: i16,
    blocks: [[[RegEntry<ProtocolState>; 16]; 16]; 16],
}

impl ChunkSection {
    pub fn empty() -> ChunkSection {
        let mut section = ChunkSection {
            block_count: 0,
            blocks: std::array::from_fn(|_| {
                std::array::from_fn(|_| {
                    std::array::from_fn(|_| unsafe { RegEntry::new_unchecked(0) })
                })
            }),
        };
        for x in 0..15 {
            for z in 0..15 {
                section.set_block_at(Position::new(x, 0, z), BlockState::from_protocol_id(1));
            }
        }

        section
    }

    pub fn set_block_at(&mut self, pos: Position<usize>, block: BlockState) {
        self.blocks[*pos.x()][*pos.y()][*pos.z()] =
            unsafe { RegEntry::new_unchecked(block.protocol_id().try_into().unwrap()) };
    }

    pub fn get_block_at(&mut self, pos: Position<usize>) -> BlockState {
        let ptc = self.blocks[*pos.x()][*pos.y()][*pos.z()];
        BlockState::from_protocol_id(ptc.id() as i32)
    }

    pub fn flatten_blocks(&self) -> [RegEntry<ProtocolState>; 4096] {
        let mut arr = [unsafe { RegEntry::new_unchecked(0) }; 4096];
        let mut idx = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    arr[idx] = self.blocks[x][y][z];
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
