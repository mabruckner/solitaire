use std::ops::{
    Add,
    Sub,
};


/// GridValues are for indicating positions with greater semantic content.
/// 
/// The first value is the major location with each increment equal to one-half the size of the card along the axis in
/// question. The second value is the minor location, each increment is one-half the size of a
/// card's identifying mark - an offset of two minor locations is enough to fully reveal the suit
/// and rank of the card below.
///
/// It is assumed that positive Y is down, positive X is right, and high sort values occlude low ones
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GridValue(pub i32, pub i32);

impl GridValue {
    pub fn new(major: i32, minor: i32) -> GridValue {
        GridValue(major, minor)
    }
    pub fn to_float(&self, major: f32, minor: f32) -> f32 {
        self.0 as f32*major + self.1 as f32*minor
    }
}

impl Add for GridValue {
    type Output = GridValue;
    fn add(self, other: GridValue) -> GridValue {
        GridValue(self.0+other.0, self.1+other.1)
    }
}

impl Sub for GridValue {
    type Output = GridValue;
    fn sub(self, other: GridValue) -> GridValue {
        GridValue(self.0-other.0, self.1-other.1)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GridLocation {
    pub x: GridValue,
    pub y: GridValue,
    pub sort: i32
}

impl GridLocation {
    pub fn new(x:GridValue, y:GridValue, sort: i32) -> Self {
        GridLocation {
            x: x,
            y: y,
            sort: sort
        }
    }
}

impl Add for GridLocation {
    type Output = GridLocation;
    fn add(self, other: GridLocation) -> GridLocation {
        GridLocation {
            x: self.x + other.x,
            y: self.y + other.y,
            sort: self.sort + other.sort
        }
    }
}

impl Sub for GridLocation {
    type Output = GridLocation;
    fn sub(self, other: GridLocation) -> GridLocation {
        GridLocation {
            x: self.x - other.x,
            y: self.y - other.y,
            sort: self.sort - other.sort
        }
    }
}
