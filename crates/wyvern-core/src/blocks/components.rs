use std::collections::BTreeMap;
use wyvern_components::{DataComponentMap, DataComponentType};
use wyvern_datatypes::nbt::Nbt;
use wyvern_values::id;

use super::{Axis, BlockDirection, ChestType, Half, StairShape};

macro_rules! generate_block_components {
    (
        $(
            pub const $name:ident: $type:ty = $id:expr, $string:expr ;
        )*
    ) => {
        pub struct BlockComponents;

        impl BlockComponents {
            $(
                pub const $name: DataComponentType<$type> = DataComponentType::new($id);
            )*
        }

        #[allow(unused)]
        pub fn components_to_array(components: &DataComponentMap) -> BTreeMap<String, String> {
            let mut map = BTreeMap::new();
            $(
                if let Ok(value) = components.get(BlockComponents::$name) {
                    map.insert($string.into(), value.to_string());
                }
            )*

            map
        }

        #[allow(unused)]
        pub fn array_to_components(array: &BTreeMap<String, String>) -> DataComponentMap {
            let mut map = DataComponentMap::new();
            for element in array {
                match element.0.as_str() {
                    $(
                        $string => {
                            if let Ok(value) = element.1.parse::<$type>() {
                                map.set(BlockComponents::$name, value)
                            }
                        }
                    )*
                    _ => {}
                }
            }
            map
        }
    };
}

generate_block_components! {
    pub const WATERLOGGED: bool = id![minecraft:waterlogged], "waterlogged";
    pub const POWERED: bool = id![minecraft:powered], "powered";
    pub const OPEN: bool = id![minecraft:open], "open";

    pub const AXIS: Axis = id![minecraft:axis], "axis";
    pub const FACING: BlockDirection = id![minecraft:facing], "facing";
    pub const HALF: Half = id![minecraft:half], "half";
    pub const STAIR_SHAPE: StairShape = id![minecraft:stair/shape], "shape";

    pub const AGE: i32 = id![minecraft:age], "age";
    pub const SNOWY: bool = id![minecraft:snowy], "snowy";

    pub const BANNER_ROTATION: i32 = id![minecraft:banner/rotation], "rotation";
    pub const TNT_UNSTABLE: bool = id![minecraft:tnt/unstable], "unstable";
    pub const CHEST_TYPE: ChestType = id![minecraft:chest/type], "type";
    pub const WATER_LEVEL: i32 = id![minecraft:water/level], "level";

    pub const FACING_NORTH: bool = id![minecraft:north], "north";
    pub const FACING_SOUTH: bool = id![minecraft:south], "south";
    pub const FACING_EAST: bool = id![minecraft:east], "east";
    pub const FACING_WEST: bool = id![minecraft:west], "west";
}

impl BlockComponents {
    pub const CUSTOM_DATA: DataComponentType<Nbt> =
        DataComponentType::new(id![minecraft:custom_data]);
}
