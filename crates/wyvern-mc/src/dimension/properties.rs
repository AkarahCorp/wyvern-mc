use std::{marker::PhantomData, str::FromStr};

#[allow(dead_code)]
pub struct BlockProperty<T: FromStr + ToString> {
    pub(crate) name: &'static str,
    pub(crate) condition: fn(&T) -> bool,
    _phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl<T: FromStr + ToString> BlockProperty<T> {
    pub const fn new(name: &'static str) -> BlockProperty<T> {
        BlockProperty {
            name,
            condition: |_| true,
            _phantom: PhantomData,
        }
    }

    pub const fn new_restrict(name: &'static str, condition: fn(&T) -> bool) -> BlockProperty<T> {
        BlockProperty {
            name,
            condition,
            _phantom: PhantomData,
        }
    }

    
}

pub struct Properties;

impl Properties {
    pub const AGE: BlockProperty<u8> = BlockProperty::new_restrict("age", |x| *x <= 15);
    pub const SNOWY: BlockProperty<bool> = BlockProperty::new("snowy");
    pub const FACING: BlockProperty<BlockDirection> = BlockProperty::new("facing");
    pub const BANNER_ROTATION: BlockProperty<u8> = BlockProperty::new_restrict("rotation", |x| *x <= 15);
    pub const OPEN: BlockProperty<bool> = BlockProperty::new("open");
    pub const WATERLOGGED: BlockProperty<bool> = BlockProperty::new("waterlogged");
    pub const AXIS: BlockProperty<Axis> = BlockProperty::new("axis");
    pub const BED_OCCUPIED: BlockProperty<bool> = BlockProperty::new("occupied");
    pub const BED_PART: BlockProperty<BedPart> = BlockProperty::new("part");
    pub const BEEHIVE_HONEY_LEVEL: BlockProperty<u8> = BlockProperty::new_restrict("honey_level", |x| *x <= 5);
    pub const POWERED: BlockProperty<bool> = BlockProperty::new("powered");
    pub const FURNACE_LIT: BlockProperty<bool> = BlockProperty::new("bool");
}

macro_rules! make_enum {
    (
        $name:ident =>
            $(
                $key:ident as $value:expr,
            )*
    ) => {
        pub enum $name {
            $($key),*
        }

        impl FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($value => Ok($name::$key),)*
                    _ => Err(())
                }
            }
        }

        impl ToString for $name {
            fn to_string(&self) -> String {
                (match self {
                    $($name::$key => $value,)*
                }).to_string()
            }
        }
    };
}

make_enum! {
    BlockDirection =>
        Up as "up",
        Down as "down",
        North as "north",
        South as "south",
        East as "east",
        West as "west",
}

make_enum! {
    Axis =>
        X as "x",
        Y as "y",
        Z as "z",
}

make_enum! {
    BedPart =>
        Head as "head",
        Foot as "foot",
}