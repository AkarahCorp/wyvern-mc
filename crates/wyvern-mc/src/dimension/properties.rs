use std::{marker::PhantomData, str::FromStr};

use crate::components::{ComponentKind, ComponentRegistry};

use super::blocks::BlockState;

#[allow(dead_code)]
pub struct StateProperty<T: FromStr + ToString> {
    pub(crate) name: &'static str,
    pub(crate) condition: fn(&T) -> bool,
    _phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl<T: FromStr + ToString> StateProperty<T> {
    pub const fn new(name: &'static str) -> StateProperty<T> {
        StateProperty {
            name,
            condition: |_| true,
            _phantom: PhantomData,
        }
    }

    pub const fn new_restrict(name: &'static str, condition: fn(&T) -> bool) -> StateProperty<T> {
        StateProperty {
            name,
            condition,
            _phantom: PhantomData,
        }
    }
}

impl<T: FromStr + ToString> ComponentKind<BlockState, BlockComponents, T> for StateProperty<T> {
    fn insert_component(&self, holder: &mut BlockState, value: T) {
        holder.insert_raw_property(self.name, &value.to_string());
    }

    fn get_component(&self, holder: &BlockState) -> Option<T> {
        T::from_str(&holder.state.iter().find(|x| x.0 == self.name)?.1).ok()
    }
}

pub struct BlockComponents;

impl BlockComponents {
    pub const AGE: StateProperty<u8> = StateProperty::new_restrict("age", |x| *x <= 15);
    pub const SNOWY: StateProperty<bool> = StateProperty::new("snowy");
    pub const FACING: StateProperty<BlockDirection> = StateProperty::new("facing");
    pub const BANNER_ROTATION: StateProperty<u8> =
        StateProperty::new_restrict("rotation", |x| *x <= 15);
    pub const OPEN: StateProperty<bool> = StateProperty::new("open");
    pub const WATERLOGGED: StateProperty<bool> = StateProperty::new("waterlogged");
    pub const AXIS: StateProperty<Axis> = StateProperty::new("axis");
    pub const BED_OCCUPIED: StateProperty<bool> = StateProperty::new("occupied");
    pub const BED_PART: StateProperty<BedPart> = StateProperty::new("part");
    pub const BEEHIVE_HONEY_LEVEL: StateProperty<u8> =
        StateProperty::new_restrict("honey_level", |x| *x <= 5);
    pub const POWERED: StateProperty<bool> = StateProperty::new("powered");
    pub const FURNACE_LIT: StateProperty<bool> = StateProperty::new("bool");
}

impl ComponentRegistry<BlockState> for BlockComponents {}

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

        #[allow(clippy::to_string_trait_impl)]
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
