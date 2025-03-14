use datafix::serialization::{CodecAdapters, CodecOps, DefaultCodec};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Vec2<T: Copy> {
    inner: [T; 2],
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Vec3<T: Copy> {
    inner: [T; 3],
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

impl Vec2<f32> {
    pub fn to_3d_direction(&self) -> Vec3<f64> {
        let yaw = (self.inner[0].to_radians() as f64) + 90.0;
        let pitch = self.inner[1].to_radians() as f64;

        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw)
    }
}

impl Vec2<f64> {
    pub fn magnitude(&self) -> f64 {
        f64::sqrt(self.x().powi(2) + self.y().powi(2))
    }
}

impl<T: Copy + Default + DefaultCodec<OT, O>, OT: Clone, O: CodecOps<OT>> DefaultCodec<OT, O>
    for Vec2<T>
{
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

    pub fn map<U: Copy>(&self, f: impl Fn(T) -> U) -> Vec3<U> {
        Vec3::new(f(self.x()), f(self.y()), f(self.z()))
    }
}

impl Vec3<f64> {
    pub fn floor(&self) -> Vec3<i32> {
        Vec3::new(
            self.x().floor() as i32,
            self.y().floor() as i32,
            self.z().floor() as i32,
        )
    }

    pub fn magnitude(&self) -> f64 {
        f64::sqrt(self.x().powi(2) + self.y().powi(2) + self.z().powi(2))
    }
}

impl<T: Copy + Default + DefaultCodec<OT, O>, OT: Clone, O: CodecOps<OT>> DefaultCodec<OT, O>
    for Vec3<T>
{
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
