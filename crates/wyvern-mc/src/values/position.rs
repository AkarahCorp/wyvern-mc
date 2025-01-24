use std::ops::Add;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Position<T: Copy, A: Copy = ()> {
    x: T,
    y: T,
    z: T,
    pitch: A,
    yaw: A,
}

impl<T: Copy> Position<T, ()> {
    pub fn new(x: T, y: T, z: T) -> Position<T, ()> {
        Position {
            x,
            y,
            z,
            pitch: (),
            yaw: (),
        }
    }

    pub fn map<U: Copy, F: Fn(&T) -> U>(&self, f: F) -> Position<U, ()> {
        Position::new(f(&self.x), f(&self.y), f(&self.z))
    }
}

impl<T: Copy, A: Copy> Position<T, A> {
    pub fn new_angled(x: T, y: T, z: T, pitch: A, yaw: A) -> Position<T, A> {
        Position {
            x,
            y,
            z,
            pitch,
            yaw,
        }
    }

    pub fn map_coords<U: Copy, F: Fn(&T) -> U>(&self, f: F) -> Position<U, A> {
        Position::new_angled(f(&self.x), f(&self.y), f(&self.z), self.pitch, self.yaw)
    }

    pub fn map_into_coords<U: Copy, F: Fn(&T) -> U>(&self, f: F) -> Position<U, ()> {
        Position::new_angled(f(&self.x), f(&self.y), f(&self.z), (), ())
    }

    pub fn map_angled<U: Copy, B: Copy, F: Fn(&T) -> U, G: Fn(&A) -> B>(
        &self,
        f: F,
        g: G,
    ) -> Position<U, B> {
        Position::new_angled(
            f(&self.x),
            f(&self.y),
            f(&self.z),
            g(&self.pitch),
            g(&self.yaw),
        )
    }

    pub fn x(&self) -> &T {
        &self.x
    }

    pub fn y(&self) -> &T {
        &self.y
    }

    pub fn z(&self) -> &T {
        &self.z
    }

    pub fn pitch(&self) -> &A {
        &self.pitch
    }

    pub fn yaw(&self) -> &A {
        &self.yaw
    }

    pub fn with_x(self, x: T) -> Position<T, A> {
        Position { x, ..self }
    }

    pub fn with_y(self, y: T) -> Position<T, A> {
        Position { y, ..self }
    }

    pub fn with_z(self, z: T) -> Position<T, A> {
        Position { z, ..self }
    }

    pub fn with_pitch(self, pitch: A) -> Position<T, A> {
        Position { pitch, ..self }
    }

    pub fn with_yaw(self, yaw: A) -> Position<T, A> {
        Position { yaw, ..self }
    }
}

impl<T: Copy> Add<Position<T, ()>> for Position<T, ()>
where
    T: Add<T, Output = T>,
{
    type Output = Position<T, ()>;

    fn add(self, rhs: Position<T, ()>) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            pitch: (),
            yaw: (),
        }
    }
}
