use std::ops::Add;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Position<T> {
    x: T,
    y: T,
    z: T,
}

impl<T> Position<T> {
    pub fn new(x: T, y: T, z: T) -> Position<T> {
        Position { x, y, z }
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
}

impl<T> Add<Position<T>> for Position<T>
where
    T: Add<T, Output = T>,
{
    type Output = Position<T>;

    fn add(self, rhs: Position<T>) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
