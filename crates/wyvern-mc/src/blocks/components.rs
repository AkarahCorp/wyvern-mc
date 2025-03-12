use crate::{
    components::{DataComponentMap, DataComponentType},
    id,
    values::nbt::Nbt,
};
use std::collections::BTreeMap;

use super::{Axis, BlockDirection};

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
            let mut arr = Vec::new();
            $(
                if let Ok(value) = components.get(BlockComponents::$name) {
                    arr.push(($string, value.to_string()));
                }
            )*

            arr.into_iter().map(|x| (x.0.to_string(), x.1)).collect()
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

// The block components *must* be in alphabetical order of the string key. Failure to do so will result in panics.
generate_block_components! {
    pub const AGE: i32 = id![minecraft:age], "age";
    pub const FACING: BlockDirection = id![minecraft:facing], "facing";
    pub const BANNER_ROTATION: i32 = id![minecraft:banner_rotation], "rotation";
    pub const AXIS: Axis = id![minecraft:axis], "axis";
    pub const SNOWY: bool = id![minecraft:snowy], "snowy";
}

impl BlockComponents {
    pub const CUSTOM_DATA: DataComponentType<Nbt> =
        DataComponentType::new(id![minecraft:custom_data]);
}
