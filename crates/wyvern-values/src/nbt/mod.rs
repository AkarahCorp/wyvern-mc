mod ops;
pub use ops::*;
use voxidian_protocol::packet::PacketBuf;

use std::collections::HashMap;

use voxidian_protocol::value::Nbt as PtcNbt;
use voxidian_protocol::value::NbtCompound as PtcNbtCompound;
use voxidian_protocol::value::NbtElement as PtcNbtElement;

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Byte,
    Boolean,
    Short,
    Int,
    Long,
    Float,
    Double,
    String,
    Array,
    Compound,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Nbt {
    Byte(i8),
    Boolean(bool),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Array(NbtArray),
    Compound(NbtCompound),
}

impl Nbt {
    pub fn new(value: impl Into<Nbt>) -> Nbt {
        value.into()
    }

    pub fn tag(&self) -> Tag {
        self.into()
    }
}

impl From<i8> for Nbt {
    fn from(value: i8) -> Self {
        Nbt::Byte(value)
    }
}

impl From<i16> for Nbt {
    fn from(value: i16) -> Self {
        Nbt::Short(value)
    }
}

impl From<i32> for Nbt {
    fn from(value: i32) -> Self {
        Nbt::Int(value)
    }
}

impl From<i64> for Nbt {
    fn from(value: i64) -> Self {
        Nbt::Long(value)
    }
}

impl From<f32> for Nbt {
    fn from(value: f32) -> Self {
        Nbt::Float(value)
    }
}

impl From<f64> for Nbt {
    fn from(value: f64) -> Self {
        Nbt::Double(value)
    }
}

impl From<NbtArray> for Nbt {
    fn from(value: NbtArray) -> Self {
        Nbt::Array(value)
    }
}

impl From<NbtCompound> for Nbt {
    fn from(value: NbtCompound) -> Self {
        Nbt::Compound(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NbtArray {
    inner: Vec<Nbt>,
}

impl From<&Nbt> for Tag {
    fn from(value: &Nbt) -> Self {
        match value {
            Nbt::Byte(_) => Tag::Byte,
            Nbt::Boolean(_) => Tag::Boolean,
            Nbt::Short(_) => Tag::Short,
            Nbt::Int(_) => Tag::Int,
            Nbt::Long(_) => Tag::Long,
            Nbt::Float(_) => Tag::Float,
            Nbt::Double(_) => Tag::Double,
            Nbt::String(_) => Tag::String,
            Nbt::Array(_) => Tag::Array,
            Nbt::Compound(_) => Tag::Compound,
        }
    }
}

impl From<Nbt> for PtcNbtElement {
    fn from(value: Nbt) -> Self {
        match value {
            Nbt::Byte(value) => PtcNbtElement::Byte(value),
            Nbt::Boolean(value) => PtcNbtElement::Byte(value as i8),
            Nbt::Short(value) => PtcNbtElement::Short(value),
            Nbt::Int(value) => PtcNbtElement::Int(value),
            Nbt::Long(value) => PtcNbtElement::Long(value),
            Nbt::Float(value) => PtcNbtElement::Float(value),
            Nbt::Double(value) => PtcNbtElement::Double(value),
            Nbt::String(value) => PtcNbtElement::String(value),
            Nbt::Array(nbt_array) => {
                PtcNbtElement::List(nbt_array.inner.into_iter().map(|x| x.into()).collect())
            }
            Nbt::Compound(nbt_compound) => PtcNbtElement::Compound({
                let mut map = PtcNbtCompound::new();
                for (key, value) in nbt_compound.inner.into_iter() {
                    let element: PtcNbtElement = value.into();
                    map.insert(key, element);
                }
                map
            }),
        }
    }
}

impl From<PtcNbtElement> for Nbt {
    fn from(value: PtcNbtElement) -> Self {
        match value {
            PtcNbtElement::Byte(v) => Nbt::Byte(v),
            PtcNbtElement::Short(v) => Nbt::Short(v),
            PtcNbtElement::Int(v) => Nbt::Int(v),
            PtcNbtElement::Long(v) => Nbt::Long(v),
            PtcNbtElement::Float(v) => Nbt::Float(v),
            PtcNbtElement::Double(v) => Nbt::Double(v),
            PtcNbtElement::BArray(_items) => todo!(),
            PtcNbtElement::String(v) => Nbt::String(v),
            PtcNbtElement::List(nbt_elements) => {
                let mut array = NbtArray::new();
                for element in nbt_elements {
                    array.push(element).unwrap();
                }
                Nbt::Array(array)
            }
            PtcNbtElement::Compound(nbt_compound) => {
                let mut compound = NbtCompound::new();
                for entry in nbt_compound.entries() {
                    compound.set(entry.0, Nbt::from(entry.1.clone()));
                }
                Nbt::Compound(compound)
            }
            PtcNbtElement::IArray(_items) => todo!(),
            PtcNbtElement::LArray(_items) => todo!(),
        }
    }
}

impl NbtArray {
    pub fn new() -> NbtArray {
        NbtArray { inner: Vec::new() }
    }

    pub fn push(&mut self, value: impl Into<Nbt>) -> Result<(), Nbt> {
        let value = value.into();
        let Some(first) = self.inner.first() else {
            self.inner.push(value);
            return Ok(());
        };

        if first.tag() == value.tag() {
            self.inner.push(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Nbt> {
        self.inner.iter().cloned()
    }
}

impl Default for NbtArray {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NbtCompound {
    inner: HashMap<String, Nbt>,
}

impl From<PtcNbtCompound> for NbtCompound {
    fn from(value: PtcNbtCompound) -> Self {
        let mut c = NbtCompound::new();
        for entry in value.entries() {
            let entry = (entry.0.clone(), entry.1.clone());
            c.set(entry.0, entry.1.into());
        }
        c
    }
}

impl NbtCompound {
    pub fn new() -> NbtCompound {
        NbtCompound {
            inner: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, key: impl Into<String>) -> Option<&Nbt> {
        self.inner.get(&key.into())
    }

    pub fn get_mut(&mut self, key: impl Into<String>) -> Option<&mut Nbt> {
        self.inner.get_mut(&key.into())
    }

    pub fn set(&mut self, key: impl Into<String>, value: Nbt) {
        self.inner.insert(key.into(), value);
    }

    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }
}

impl Default for NbtCompound {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NbtDecodeError {
    BadNbt,
    NotGzipped,
}

impl TryFrom<Vec<u8>> for NbtCompound {
    type Error = NbtDecodeError;

    fn try_from(value: Vec<u8>) -> Result<Self, NbtDecodeError> {
        let mut buf = PacketBuf::from(value)
            .inflate_gzip()
            .map_err(|_| NbtDecodeError::NotGzipped)?;
        let nbt = PtcNbt::read_named(&mut buf)
            .map_err(|_| NbtDecodeError::BadNbt)?
            .root;
        Ok(nbt.into())
    }
}

#[macro_export]
macro_rules! compound {
    (
        $($key:ident: $value:expr),*
    ) => {{
        let mut compound = $crate::nbt::NbtCompound::new();

        $(
            {
                compound.set(stringify!($key), $crate::nbt::Nbt::new($value));
            }
        )*
        compound
    }};
}

#[macro_export]
macro_rules! list {
    (
        $($value:expr),*
    ) => {{
        let mut value = $crate::nbt::NbtArray::new();
        $(
            {
                let _ = value.push($crate::nbt::Nbt::new($value));
            }
        )*
        value
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn compound_creation() {
        let nbt = compound! {
            x: 10,
            y: 20,
            z: list![
                compound! {
                    x: 10
                },
                compound! {
                    y: 20
                },
                compound! {
                    z: 60
                }
            ]
        };

        eprintln!("{:?}", nbt);
    }
}
