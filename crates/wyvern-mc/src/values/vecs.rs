use datafix::serialization::{CodecAdapters, CodecOps, DefaultCodec};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Vec1<T: Copy> {
    inner: [T; 1],
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Vec2<T: Copy> {
    inner: [T; 2],
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Vec3<T: Copy> {
    inner: [T; 3],
}

impl<T: Copy> Vec1<T> {
    pub fn new(x: T) -> Self {
        Vec1 { inner: [x] }
    }

    pub fn x(&self) -> T {
        unsafe { *self.inner.get_unchecked(0) }
    }

    pub fn with_x(&self, value: T) -> Self {
        let mut new = *self;
        unsafe { *new.inner.get_unchecked_mut(0) = value }
        new
    }
}

impl<T: Copy + Default + DefaultCodec<OT, O>, OT, O: CodecOps<OT>> DefaultCodec<OT, O> for Vec1<T> {
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        T::codec().list_of().xmap(
            |vec| Vec1::new(*vec.first().unwrap_or(&T::default())),
            |vec1| Vec::from([vec1.x()]),
        )
    }
}

impl<T: Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vec2 { inner: [x, y] }
    }

    pub fn x(&self) -> T {
        unsafe { *self.inner.get_unchecked(0) }
    }

    pub fn with_x(&self, value: T) -> Self {
        let mut new = *self;
        unsafe { *new.inner.get_unchecked_mut(0) = value }
        new
    }

    pub fn y(&self) -> T {
        unsafe { *self.inner.get_unchecked(1) }
    }

    pub fn with_y(&self, value: T) -> Self {
        let mut new = *self;
        unsafe { *new.inner.get_unchecked_mut(1) = value }
        new
    }
}

impl<T: Copy + Default + DefaultCodec<OT, O>, OT, O: CodecOps<OT>> DefaultCodec<OT, O> for Vec2<T> {
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        T::codec().list_of().xmap(
            |vec| {
                Vec2::new(
                    *vec.first().unwrap_or(&T::default()),
                    *vec.get(1).unwrap_or(&T::default()),
                )
            },
            |vec2| Vec::from([vec2.x(), vec2.y()]),
        )
    }
}

impl<T: Copy> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3 { inner: [x, y, z] }
    }

    pub fn x(&self) -> T {
        unsafe { *self.inner.get_unchecked(0) }
    }

    pub fn with_x(&self, value: T) -> Self {
        let mut new = *self;
        unsafe { *new.inner.get_unchecked_mut(0) = value }
        new
    }

    pub fn y(&self) -> T {
        unsafe { *self.inner.get_unchecked(1) }
    }

    pub fn with_y(&self, value: T) -> Self {
        let mut new = *self;
        unsafe { *new.inner.get_unchecked_mut(1) = value }
        new
    }

    pub fn z(&self) -> T {
        unsafe { *self.inner.get_unchecked(2) }
    }

    pub fn with_z(&self, value: T) -> Self {
        let mut new = *self;
        unsafe { *new.inner.get_unchecked_mut(2) = value }
        new
    }
}

impl<T: Copy + Default + DefaultCodec<OT, O>, OT, O: CodecOps<OT>> DefaultCodec<OT, O> for Vec3<T> {
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        T::codec().list_of().xmap(
            |vec| {
                Vec3::new(
                    *vec.first().unwrap_or(&T::default()),
                    *vec.get(1).unwrap_or(&T::default()),
                    *vec.get(2).unwrap_or(&T::default()),
                )
            },
            |vec3| Vec::from([vec3.x(), vec3.y(), vec3.z()]),
        )
    }
}
