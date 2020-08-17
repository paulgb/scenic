use std::cmp::PartialOrd;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    pub fn coords(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<std::cmp::Ordering> {
        (self.x, self.y).partial_cmp(&(other.x, other.y))
    }
}

impl Eq for Point {}

impl std::cmp::Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("Invalid values encountered in point comparison.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lt() {
        assert_eq!(true, Point::new(4., 5.) < Point::new(5., 5.));
        assert_eq!(true, Point::new(4., 4.) < Point::new(4., 5.));
        assert_eq!(false, Point::new(5., 5.) < Point::new(5., 5.));
        assert_eq!(false, Point::new(4., 4.) < Point::new(4., 3.));
        assert_eq!(false, Point::new(4., 4.) < Point::new(3., 6.));
    }
}
