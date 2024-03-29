use crate::line::Line;
use crate::point::Point;

#[derive(Debug)]
pub struct Polygon {
    pub points: Vec<Point>,
    pub z: f64,
    pub lines: Vec<Line>,
}

impl<'a> Polygon {
    pub fn new(points: Vec<Point>, z: f64) -> Polygon {
        let mut poly = Polygon {
            points,
            z,
            lines: Vec::new(),
        };

        let mut last_point = poly
            .points
            .last()
            .expect("Tried to build lines from empty polygon.");

        for point in poly.points.iter() {
            poly.lines
                .push(Line::new_with_poly(*last_point, *point, &poly));
            last_point = point;
        }

        poly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line::LineOrientation;

    #[test]
    fn test_lines() {
        let p1 = Point::new(5., 5.);
        let p2 = Point::new(10., 10.);
        let p3 = Point::new(15., 5.);
        let p4 = Point::new(10., 0.);

        let poly = Polygon::new(vec![p1, p2, p3, p4], 1.);

        let lines = poly.lines;
        assert_eq!(p1, lines[0].start);
        assert_eq!(p4, lines[0].end);
        assert_eq!(LineOrientation::RightToLeft, lines[0].orientation);

        assert_eq!(p1, lines[1].start);
        assert_eq!(p2, lines[1].end);
        assert_eq!(LineOrientation::LeftToRight, lines[1].orientation);

        assert_eq!(p2, lines[2].start);
        assert_eq!(p3, lines[2].end);
        assert_eq!(LineOrientation::LeftToRight, lines[2].orientation);

        assert_eq!(p4, lines[3].start);
        assert_eq!(p3, lines[3].end);
        assert_eq!(LineOrientation::RightToLeft, lines[3].orientation);
    }
}
