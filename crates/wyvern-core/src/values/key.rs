use std::{borrow::Cow, fmt::Debug, hash::Hash};

use datafix::serialization::{CodecAdapters, CodecOps, DefaultCodec};
use voxidian_protocol::value::Identifier;

pub struct Id {
    namespace: Cow<'static, str>,
    path: Cow<'static, str>,
}
impl Id {
    pub fn new(namespace: &str, path: &str) -> Self {
        Id {
            namespace: Cow::Owned(namespace.into()),
            path: Cow::Owned(path.into()),
        }
    }

    pub const fn constant(namespace: &'static str, path: &'static str) -> Self {
        Id {
            namespace: Cow::Borrowed(namespace),
            path: Cow::Borrowed(path),
        }
    }

    pub fn empty() -> Self {
        Id {
            namespace: "".into(),
            path: "".into(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn from_string(string: &String) -> Self {
        Identifier::from(string).into()
    }

    pub fn into_string(&self) -> String {
        Identifier::from(self.clone()).to_string()
    }
}

impl From<Identifier> for Id {
    fn from(value: Identifier) -> Self {
        Id::new(&value.namespace, &value.path)
    }
}

impl From<Id> for Identifier {
    fn from(value: Id) -> Self {
        Identifier::new(value.namespace, value.path)
    }
}

impl Eq for Id {}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.path == other.path
    }
}

impl Hash for Id {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace().hash(state);
        self.path().hash(state);
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace(), self.path())
    }
}

impl Clone for Id {
    fn clone(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            path: self.path.clone(),
        }
    }
}

#[macro_export]
macro_rules! id {
    ($namespace:ident:$path:ident$(/$subpath:ident)*) => {
        $crate::values::Id::constant(
            stringify!($namespace),
            concat!(
                stringify!($path),
                $("/", stringify!($subpath), )*
            )
        )
    };
}

impl<OT: Clone, O: CodecOps<OT>> DefaultCodec<OT, O> for Id {
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        String::codec().xmap(Id::from_string, Id::into_string)
    }
}
