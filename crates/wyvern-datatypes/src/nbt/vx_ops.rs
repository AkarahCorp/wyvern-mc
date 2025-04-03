use datafix::{
    result::{DataError, DataResult},
    serialization::{CodecOps, ListView, ListViewMut, MapView, MapViewMut},
};
use voxidian_protocol::value::{NbtCompound, NbtElement};

#[derive(Debug, Clone)]
pub struct VxNbtOps;

impl CodecOps for VxNbtOps {
    type T = NbtElement;

    fn create_double(&self, value: &f64) -> NbtElement {
        NbtElement::Double(*value)
    }

    fn create_float(&self, value: &f32) -> NbtElement {
        NbtElement::Float(*value)
    }

    fn create_byte(&self, value: &i8) -> NbtElement {
        NbtElement::Byte(*value)
    }

    fn create_short(&self, value: &i16) -> NbtElement {
        NbtElement::Short(*value)
    }

    fn create_int(&self, value: &i32) -> NbtElement {
        NbtElement::Int(*value)
    }

    fn create_long(&self, value: &i64) -> NbtElement {
        NbtElement::Long(*value)
    }

    fn create_string(&self, value: &str) -> NbtElement {
        NbtElement::String(value.to_string())
    }

    fn create_boolean(&self, value: &bool) -> NbtElement {
        NbtElement::Byte(if *value { 1 } else { 0 })
    }

    fn create_list(&self, value: impl IntoIterator<Item = NbtElement>) -> NbtElement {
        let mut arr = Vec::new();
        for element in value {
            let _ = arr.push(element);
        }
        NbtElement::List(arr)
    }

    fn create_map(&self, pairs: impl IntoIterator<Item = (String, NbtElement)>) -> NbtElement {
        let mut obj = NbtCompound::new();
        for element in pairs {
            obj.insert(element.0, element.1);
        }
        NbtElement::Compound(obj)
    }

    fn create_unit(&self) -> NbtElement {
        NbtElement::Compound(NbtCompound::new())
    }

    fn get_float(&self, value: &NbtElement) -> datafix::result::DataResult<f32> {
        match value {
            NbtElement::Float(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("float")),
        }
    }

    fn get_double(&self, value: &NbtElement) -> datafix::result::DataResult<f64> {
        match value {
            NbtElement::Double(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("double")),
        }
    }

    fn get_byte(&self, value: &NbtElement) -> datafix::result::DataResult<i8> {
        match value {
            NbtElement::Byte(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("byte")),
        }
    }

    fn get_short(&self, value: &NbtElement) -> datafix::result::DataResult<i16> {
        match value {
            NbtElement::Short(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("short")),
        }
    }

    fn get_int(&self, value: &NbtElement) -> datafix::result::DataResult<i32> {
        match value {
            NbtElement::Int(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("int")),
        }
    }

    fn get_long(&self, value: &NbtElement) -> datafix::result::DataResult<i64> {
        match value {
            NbtElement::Long(value) => Ok(*value),
            _ => Err(DataError::unexpected_type("long")),
        }
    }

    fn get_string(&self, value: &NbtElement) -> datafix::result::DataResult<String> {
        match value {
            NbtElement::String(value) => Ok(value.clone()),
            _ => Err(DataError::unexpected_type("string")),
        }
    }

    fn get_boolean(&self, value: &NbtElement) -> datafix::result::DataResult<bool> {
        match value {
            NbtElement::Byte(value) => Ok(*value == 1),
            _ => Err(DataError::unexpected_type("float")),
        }
    }

    fn get_list(
        &self,
        value: &NbtElement,
    ) -> datafix::result::DataResult<impl datafix::serialization::ListView<NbtElement>> {
        if let NbtElement::List(_) = value {
            Ok(VxNbtListView { value })
        } else {
            Err(DataError::unexpected_type("list"))
        }
    }

    fn get_map(
        &self,
        value: &NbtElement,
    ) -> datafix::result::DataResult<impl datafix::serialization::MapView<NbtElement>> {
        if let NbtElement::Compound(_) = value {
            Ok(VxNbtCompoundView { value })
        } else {
            Err(DataError::unexpected_type("compound"))
        }
    }

    fn get_unit(&self, _value: &NbtElement) -> datafix::result::DataResult<()> {
        Ok(())
    }

    fn get_list_mut(&self, value: &mut NbtElement) -> DataResult<impl ListViewMut<NbtElement>> {
        if let NbtElement::List(_) = value {
            Ok(VxNbtListViewMut { value })
        } else {
            Err(DataError::unexpected_type("list"))
        }
    }

    fn get_map_mut(&self, value: &mut NbtElement) -> DataResult<impl MapViewMut<NbtElement>> {
        if let NbtElement::Compound(_) = value {
            Ok(VxNbtCompoundViewMut { value })
        } else {
            Err(DataError::unexpected_type("compound"))
        }
    }
}

pub struct VxNbtCompoundView<'a> {
    value: &'a NbtElement,
}

impl MapView<NbtElement> for VxNbtCompoundView<'_> {
    fn get(&self, name: &str) -> DataResult<&NbtElement> {
        let NbtElement::Compound(compound) = self.value else {
            return Err(DataError::unexpected_type("compound"));
        };
        compound.get(name).ok_or(DataError::key_not_found(name))
    }

    fn keys(&self) -> Vec<String> {
        let NbtElement::Compound(compound) = self.value else {
            return Vec::new();
        };
        compound.entries().map(|x| x.0.clone()).collect()
    }
}

pub struct VxNbtCompoundViewMut<'a> {
    value: &'a mut NbtElement,
}

impl MapView<NbtElement> for VxNbtCompoundViewMut<'_> {
    fn get(&self, name: &str) -> DataResult<&NbtElement> {
        let NbtElement::Compound(compound) = &self.value else {
            return Err(DataError::unexpected_type("compound"));
        };
        compound.get(name).ok_or(DataError::key_not_found(name))
    }

    fn keys(&self) -> Vec<String> {
        todo!()
    }
}

impl MapViewMut<NbtElement> for VxNbtCompoundViewMut<'_> {
    fn set(&mut self, name: &str, value: NbtElement) {
        let NbtElement::Compound(compound) = self.value else {
            return;
        };
        compound.insert(name, value);
    }

    fn remove(&mut self, _key: &str) -> datafix::result::DataResult<NbtElement> {
        todo!()
    }

    fn get_mut(&mut self, name: &str) -> DataResult<&mut NbtElement> {
        let NbtElement::Compound(compound) = self.value else {
            return Err(DataError::unexpected_type("compound"));
        };
        compound.get_mut(name).ok_or(DataError::key_not_found(name))
    }
}

pub struct VxNbtListView<'a> {
    value: &'a NbtElement,
}

impl ListView<NbtElement> for VxNbtListView<'_> {
    fn get(&self, index: usize) -> DataResult<&NbtElement> {
        let NbtElement::List(list) = self.value else {
            return Err(DataError::unexpected_type("list"));
        };
        let len = list.len();
        list.get(index)
            .ok_or(DataError::list_index_out_of_bounds(index, len))
    }

    fn into_iter(self) -> impl Iterator<Item = NbtElement> {
        let NbtElement::List(list) = self.value else {
            return Vec::new().into_iter();
        };
        list.clone().into_iter()
    }
}

pub struct VxNbtListViewMut<'a> {
    value: &'a mut NbtElement,
}

impl ListViewMut<NbtElement> for VxNbtListViewMut<'_> {
    fn append(&mut self, value: NbtElement) {
        let NbtElement::List(list) = self.value else {
            return;
        };
        list.push(value)
    }

    fn get_mut(&mut self, index: usize) -> DataResult<&mut NbtElement> {
        let NbtElement::List(list) = self.value else {
            return Err(DataError::unexpected_type("list"));
        };
        let len = list.len();
        list.get_mut(index)
            .ok_or(DataError::list_index_out_of_bounds(index, len))
    }
}
