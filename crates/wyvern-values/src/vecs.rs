use std::f64::consts::PI;

pub use glam::f32::*;
pub use glam::f64::*;
pub use glam::i16::*;
pub use glam::i32::*;
pub use glam::i64::*;
pub use glam::usize::*;

pub trait Vec2Ext {
    fn to_3d_direction(&self) -> DVec3;
}

impl Vec2Ext for Vec2 {
    fn to_3d_direction(&self) -> DVec3 {
        let yaw = (self[0].to_radians() as f64) + (PI / 2.0);
        let pitch = self[1].to_radians() as f64;

        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        DVec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw)
    }
}
