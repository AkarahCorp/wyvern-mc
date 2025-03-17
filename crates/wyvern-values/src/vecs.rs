use std::f64::consts::PI;

use datafix::serialization::{CodecAdapters, CodecOps, DefaultCodec};

pub type Vec2<T> = NVec<T, 2>;
pub type Vec3<T> = NVec<T, 3>;
pub type Vec4<T> = NVec<T, 4>;

#[doc(hidden)]
pub struct Guard<const B: bool>;

#[doc(hidden)]
pub trait True {}
#[doc(hidden)]
impl True for Guard<true> {}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct NVec<T: Copy, const N: usize> {
    inner: [T; N],
}

impl<T: Copy> NVec<T, 1> {
    pub fn new(x: T) -> NVec<T, 1> {
        NVec { inner: [x] }
    }
}

impl<T: Copy> NVec<T, 2> {
    pub fn new(x: T, y: T) -> NVec<T, 2> {
        NVec { inner: [x, y] }
    }
}

impl<T: Copy> NVec<T, 3> {
    pub fn new(x: T, y: T, z: T) -> NVec<T, 3> {
        NVec { inner: [x, y, z] }
    }
}

impl<T: Copy, const N: usize> NVec<T, N>
where
    Guard<{ N >= 1 }>: True,
{
    pub fn x(&self) -> T {
        self.inner[0]
    }

    pub fn with_x(&self, x: T) -> Self {
        let mut copy = *self;
        copy.inner[0] = x;
        copy
    }
}

impl<T: Copy, const N: usize> NVec<T, N>
where
    Guard<{ N >= 2 }>: True,
{
    pub fn y(&self) -> T {
        self.inner[1]
    }

    pub fn with_y(&self, y: T) -> Self {
        let mut copy = *self;
        copy.inner[1] = y;
        copy
    }
}

impl<T: Copy, const N: usize> NVec<T, N>
where
    Guard<{ N >= 3 }>: True,
{
    pub fn z(&self) -> T {
        self.inner[2]
    }

    pub fn with_z(&self, z: T) -> Self {
        let mut copy = *self;
        copy.inner[2] = z;
        copy
    }
}

impl<T: Copy, const N: usize> NVec<T, N> {
    pub fn map<U: Copy + Default>(&self, f: impl Fn(T) -> U) -> NVec<U, N> {
        let mut copy = NVec {
            inner: std::array::from_fn(|_| U::default()),
        };
        for idx in 0..N {
            copy.inner[idx] = f(self.inner[idx]);
        }
        copy
    }
}

impl<T: Copy + DefaultCodec<OT, O>, const N: usize, OT: Clone, O: CodecOps<OT>> DefaultCodec<OT, O>
    for NVec<T, N>
{
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        T::codec().list_of().xmap(
            |n: &Vec<T>| NVec {
                inner: n.as_slice().try_into().unwrap(),
            },
            |n: &NVec<T, N>| n.inner.to_vec(),
        )
    }
}

impl<const N: usize> NVec<f64, N> {
    pub fn floor(&self) -> NVec<i32, N> {
        self.map(|x| x.floor() as i32)
    }
}

impl NVec<f32, 2> {
    pub fn to_3d_direction(&self) -> Vec3<f64> {
        let yaw = (self.inner[0].to_radians() as f64) + (PI / 2.0);
        let pitch = self.inner[1].to_radians() as f64;

        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        Vec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw)
    }
}
