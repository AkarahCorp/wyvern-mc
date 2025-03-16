use datafix::{
    result::{DataError, DataResult},
    serialization::{CodecOps, ListView, ListViewMut, MapView, MapViewMut},
};

use super::{Nbt, NbtArray, NbtCompound};

#[derive(Debug, Clone)]
pub struct NbtOps;

impl CodecOps<Nbt> for NbtOps {
    fn create_double(&self, value: &f64) -> Nbt {
        Nbt::Double(*value)
    }

    fn create_float(&self, value: &f32) -> Nbt {
        Nbt::Float(*value)
    }

    fn create_byte(&self, value: &i8) -> Nbt {
        Nbt::Byte(*value)
    }

    fn create_short(&self, value: &i16) -> Nbt {
        Nbt::Short(*value)
    }

    fn create_int(&self, value: &i32) -> Nbt {
        Nbt::Int(*value)
    }

    fn create_long(&self, value: &i64) -> Nbt {
        Nbt::Long(*value)
    }

    fn create_string(&self, value: &str) -> Nbt {
        Nbt::String(value.to_string())
    }

    fn create_boolean(&self, value: &bool) -> Nbt {
        Nbt::Byte(if *value { 1 } else { 0 })
    }

    fn create_list(&self, value: impl IntoIterator<Item = Nbt>) -> Nbt {
        let mut arr = NbtArray::new();
        for element in value {
            let _ = arr.push(element);
        }
        Nbt::Array(arr)
    }

    fn create_map(&self, pairs: impl IntoIterator<Item = (String, Nbt)>) -> Nbt {
        let mut obj = NbtCompound::new();
        for element in pairs {
            obj.set(element.0, element.1);
        }
        Nbt::Compound(obj)
    }

    fn create_unit(&self) -> Nbt {
        Nbt::Compound(NbtCompound::new())
    }

    fn get_float(&self, value: &Nbt) -> datafix::result::DataResult<f32> {
        match value {
            Nbt::Float(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("float")),
        }
    }

    fn get_double(&self, value: &Nbt) -> datafix::result::DataResult<f64> {
        match value {
            Nbt::Double(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("double")),
        }
    }

    fn get_byte(&self, value: &Nbt) -> datafix::result::DataResult<i8> {
        match value {
            Nbt::Byte(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("byte")),
        }
    }

    fn get_short(&self, value: &Nbt) -> datafix::result::DataResult<i16> {
        match value {
            Nbt::Short(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("short")),
        }
    }

    fn get_int(&self, value: &Nbt) -> datafix::result::DataResult<i32> {
        match value {
            Nbt::Int(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("int")),
        }
    }

    fn get_long(&self, value: &Nbt) -> datafix::result::DataResult<i64> {
        match value {
            Nbt::Long(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("long")),
        }
    }

    fn get_string(&self, value: &Nbt) -> datafix::result::DataResult<String> {
        match value {
            Nbt::String(value) => Ok(value.clone()),
            _ => Err(DataError::unexpected_type("string")),
        }
    }

    fn get_boolean(&self, value: &Nbt) -> datafix::result::DataResult<bool> {
        match value {
            Nbt::Byte(value) => Ok(*value == 1),
            _ => Err(DataError::unexpected_type("float")),
        }
    }

    fn get_list(
        &self,
        value: &Nbt,
    ) -> datafix::result::DataResult<impl datafix::serialization::ListView<Nbt>> {
        if let Nbt::Array(_) = value {
            Ok(NbtListView { value })
        } else {
            Err(DataError::unexpected_type("list"))
        }
    }

    fn get_map(
        &self,
        value: &Nbt,
    ) -> datafix::result::DataResult<impl datafix::serialization::MapView<Nbt>> {
        if let Nbt::Compound(_) = value {
            Ok(NbtCompoundView { value })
        } else {
            Err(DataError::unexpected_type("compound"))
        }
    }

    fn get_unit(&self, _value: &Nbt) -> datafix::result::DataResult<()> {
        Ok(())
    }

    fn get_list_mut(&self, value: &mut Nbt) -> DataResult<impl ListViewMut<Nbt>> {
        if let Nbt::Array(_) = value {
            Ok(NbtListViewMut { value })
        } else {
            Err(DataError::unexpected_type("list"))
        }
    }

    fn get_map_mut(&self, value: &mut Nbt) -> DataResult<impl MapViewMut<Nbt>> {
        if let Nbt::Compound(_) = value {
            Ok(NbtCompoundViewMut { value })
        } else {
            Err(DataError::unexpected_type("compound"))
        }
    }
}

pub struct NbtCompoundView<'a> {
    value: &'a Nbt,
}

impl MapView<Nbt> for NbtCompoundView<'_> {
    fn get(&self, name: &str) -> DataResult<&Nbt> {
        let Nbt::Compound(compound) = self.value else {
            return Err(DataError::unexpected_type("compound"));
        };
        compound.get(name).ok_or(DataError::key_not_found(name))
    }

    fn keys(&self) -> Vec<String> {
        let Nbt::Compound(compound) = self.value else {
            return Vec::new();
        };
        compound.keys()
    }
}

pub struct NbtCompoundViewMut<'a> {
    value: &'a mut Nbt,
}

impl MapView<Nbt> for NbtCompoundViewMut<'_> {
    fn get(&self, name: &str) -> DataResult<&Nbt> {
        let Nbt::Compound(compound) = &self.value else {
            return Err(DataError::unexpected_type("compound"));
        };
        compound.get(name).ok_or(DataError::key_not_found(name))
    }

    fn keys(&self) -> Vec<String> {
        todo!()
    }
}

impl MapViewMut<Nbt> for NbtCompoundViewMut<'_> {
    fn set(&mut self, name: &str, value: Nbt) {
        let Nbt::Compound(compound) = self.value else {
            return;
        };
        compound.set(name, value);
    }

    fn remove(&mut self, _key: &str) -> datafix::result::DataResult<Nbt> {
        todo!()
    }

    fn get_mut(&mut self, name: &str) -> DataResult<&mut Nbt> {
        let Nbt::Compound(compound) = self.value else {
            return Err(DataError::unexpected_type("compound"));
        };
        compound.get_mut(name).ok_or(DataError::key_not_found(name))
    }
}

pub struct NbtListView<'a> {
    value: &'a Nbt,
}

impl ListView<Nbt> for NbtListView<'_> {
    fn get(&self, index: usize) -> DataResult<&Nbt> {
        let Nbt::Array(list) = self.value else {
            return Err(DataError::unexpected_type("list"));
        };
        let len = list.inner.len();
        list.inner
            .get(index)
            .ok_or(DataError::list_index_out_of_bounds(index, len))
    }

    fn into_iter(self) -> impl Iterator<Item = Nbt> {
        let Nbt::Array(list) = self.value else {
            return Vec::new().into_iter();
        };
        list.inner.clone().into_iter()
    }
}

pub struct NbtListViewMut<'a> {
    value: &'a mut Nbt,
}

impl ListViewMut<Nbt> for NbtListViewMut<'_> {
    fn append(&mut self, value: Nbt) {
        let Nbt::Array(list) = self.value else {
            return;
        };
        list.inner.push(value)
    }

    fn get_mut(&mut self, index: usize) -> DataResult<&mut Nbt> {
        let Nbt::Array(list) = self.value else {
            return Err(DataError::unexpected_type("list"));
        };
        let len = list.inner.len();
        list.inner
            .get_mut(index)
            .ok_or(DataError::list_index_out_of_bounds(index, len))
    }
}
