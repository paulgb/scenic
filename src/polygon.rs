use crate::point::Point;

pub struct Polygon {
    pub points: Vec<Point>,
    pub z: f64,
}

impl Polygon {
    pub fn new(points: Vec<Point>, z: f64) -> Polygon {
        Polygon { points, z }
    }
}
