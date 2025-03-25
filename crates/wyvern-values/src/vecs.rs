use std::{
    f64::consts::PI,
    ops::{Add, Div, Mul, Sub},
};

use datafix::serialization::{Codec, CodecAdapters, CodecOps, DefaultCodec};
use num_traits::Float;

pub type Vec2<T> = NVec<T, 2>;
pub type Vec3<T> = NVec<T, 3>;
pub type Vec4<T> = NVec<T, 4>;

#[doc(hidden)]
pub struct Guard<const B: bool>;

#[doc(hidden)]
pub trait True {}
#[doc(hidden)]
impl True for Guard<true> {}

pub trait Vc: Copy {}
impl<T: Copy> Vc for T {}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct NVec<T: Vc, const N: usize> {
    inner: [T; N],
}

impl<T: Vc> NVec<T, 1> {
    pub fn new(x: T) -> NVec<T, 1> {
        NVec { inner: [x] }
    }
}

impl<T: Vc> NVec<T, 2> {
    pub fn new(x: T, y: T) -> NVec<T, 2> {
        NVec { inner: [x, y] }
    }
}

impl<T: Vc> NVec<T, 3> {
    pub fn new(x: T, y: T, z: T) -> NVec<T, 3> {
        NVec { inner: [x, y, z] }
    }
}

impl<T: Vc, const N: usize> NVec<T, N>
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

impl<T: Vc + Add<Output = T>, const N: usize> NVec<T, N>
where
    Guard<{ N >= 1 }>: True,
{
    pub fn shift_x(&self, x: T) -> Self {
        let mut copy = *self;
        copy.inner[0] = copy.inner[0] + x;
        copy
    }
}

impl<T: Vc, const N: usize> NVec<T, N>
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

impl<T: Vc + Add<Output = T>, const N: usize> NVec<T, N>
where
    Guard<{ N >= 2 }>: True,
{
    pub fn shift_y(&self, x: T) -> Self {
        let mut copy = *self;
        copy.inner[1] = copy.inner[1] + x;
        copy
    }
}

impl<T: Vc, const N: usize> NVec<T, N>
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

impl<T: Vc + Add<Output = T>, const N: usize> NVec<T, N>
where
    Guard<{ N >= 3 }>: True,
{
    pub fn shift_z(&self, z: T) -> Self {
        let mut copy = *self;
        copy.inner[2] = copy.inner[2] + z;
        copy
    }
}

impl<T: Vc, const N: usize> NVec<T, N>
where
    Guard<{ N >= 4 }>: True,
{
    pub fn w(&self) -> T {
        self.inner[3]
    }

    pub fn with_w(&self, w: T) -> Self {
        let mut copy = *self;
        copy.inner[3] = w;
        copy
    }
}

impl<T: Vc + Add<Output = T>, const N: usize> NVec<T, N>
where
    Guard<{ N >= 4 }>: True,
{
    pub fn shift_w(&self, w: T) -> Self {
        let mut copy = *self;
        copy.inner[3] = copy.inner[3] + w;
        copy
    }
}

impl<T: Vc + Default, const N: usize> NVec<T, N> {
    pub fn get(&self, idx: usize) -> Option<T> {
        if idx >= N {
            None
        } else {
            Some(self.inner[idx])
        }
    }

    pub fn map<U: Vc + Default>(&self, f: impl Fn(T) -> U) -> NVec<U, N> {
        let mut copy = NVec {
            inner: std::array::from_fn(|_| U::default()),
        };
        for idx in 0..N {
            copy.inner[idx] = f(self.inner[idx]);
        }
        copy
    }
}

impl<T: Vc + Default + Add<Output = T>, const N: usize> Add for NVec<T, N> {
    type Output = NVec<T, N>;

    fn add(self, other: Self) -> Self::Output {
        let mut copy = NVec {
            inner: std::array::from_fn(|_| T::default()),
        };
        for idx in 0..N {
            copy.inner[idx] = self.inner[idx] + other.inner[idx];
        }
        copy
    }
}

impl<T: Vc + Default + Sub<Output = T>, const N: usize> Sub for NVec<T, N> {
    type Output = NVec<T, N>;

    fn sub(self, other: Self) -> Self::Output {
        let mut copy = NVec {
            inner: std::array::from_fn(|_| T::default()),
        };
        for idx in 0..N {
            copy.inner[idx] = self.inner[idx] - other.inner[idx];
        }
        copy
    }
}

impl<T: Vc + Default + Mul<Output = T>, const N: usize> Mul for NVec<T, N> {
    type Output = NVec<T, N>;

    fn mul(self, other: Self) -> Self::Output {
        let mut copy = NVec {
            inner: std::array::from_fn(|_| T::default()),
        };
        for idx in 0..N {
            copy.inner[idx] = self.inner[idx] * other.inner[idx];
        }
        copy
    }
}

impl<T: Vc + Default + Div<Output = T>, const N: usize> Div for NVec<T, N> {
    type Output = NVec<T, N>;

    fn div(self, other: Self) -> Self::Output {
        let mut copy = NVec {
            inner: std::array::from_fn(|_| T::default()),
        };
        for idx in 0..N {
            copy.inner[idx] = self.inner[idx] / other.inner[idx];
        }
        copy
    }
}

impl<T: Vc + DefaultCodec<O>, const N: usize, O: CodecOps> DefaultCodec<O> for NVec<T, N> {
    fn codec() -> impl Codec<Self, O> {
        T::codec().list_of().xmap(
            |n: &Vec<T>| NVec {
                inner: n.as_slice().try_into().unwrap(),
            },
            |n: &NVec<T, N>| n.inner.to_vec(),
        )
    }
}

impl<T: Vc + Default + Float, const N: usize> NVec<T, N> {
    pub fn floor(&self) -> NVec<i32, N> {
        self.map(|x| x.floor().to_i32().unwrap_or(0))
    }

    pub fn sin(&self) -> NVec<i32, N> {
        self.map(|x| x.sin().to_i32().unwrap_or(0))
    }

    pub fn cos(&self) -> NVec<i32, N> {
        self.map(|x| x.cos().to_i32().unwrap_or(0))
    }

    pub fn tan(&self) -> NVec<i32, N> {
        self.map(|x| x.tan().to_i32().unwrap_or(0))
    }
}

impl<T: Vc + Float> NVec<T, 2> {
    pub fn to_3d_direction(&self) -> Vec3<f64> {
        let yaw = (self.inner[0].to_radians().to_f64().unwrap_or(0.0)) + (PI / 2.0);
        let pitch = self.inner[1].to_radians().to_f64().unwrap_or(0.0);

        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        Vec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw)
    }
}
