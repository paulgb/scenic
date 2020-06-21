use crate::point::Point;
use crate::polygon::Polygon;

#[derive(Debug, PartialEq)]
pub enum LineOrientation {
    Top,
    Bottom,
}

#[derive(Debug, PartialEq)]
enum LineSlope {
    FiniteSlope(f64),
    InfiniteSlope,
}

impl LineSlope {
    pub fn unwrap(&self) -> f64 {
        match &self {
            LineSlope::FiniteSlope(f) => *f,
            _ => panic!("Unwrapped InfiniteSlope."),
        }
    }
}

#[derive(Debug)]
pub struct Line {
    pub start: Point,
    pub end: Point,
    pub orientation: LineOrientation,
    pub polygon: *const Polygon,
}

impl PartialEq for Line {
    fn eq(&self, other: &Line) -> bool {
        self.cmp_repr().eq(&other.cmp_repr())
    }
}

impl Eq for Line {
}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).expect("Invalid ordering of Lines.")
    }
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Line) -> Option<std::cmp::Ordering> {
        self.cmp_repr().partial_cmp(&other.cmp_repr())
    }
}

impl Line {
    fn cmp_repr(&self) -> (Point, Point, *const Polygon) {
        (self.start, self.end, self.polygon)
    }

    pub fn new(start: Point, end: Point) -> Line {
        Line::new_with_poly(start, end, std::ptr::null())
    }

    pub fn new_with_poly(start: Point, end: Point, polygon: *const Polygon) -> Line {
        if start < end {
            Line {
                start,
                end,
                polygon,
                orientation: LineOrientation::Top,
            }
        } else {
            Line {
                start: end,
                end: start,
                polygon,
                orientation: LineOrientation::Bottom,
            }
        }
    }

    fn slope(&self) -> LineSlope {
        let rise = self.end.y - self.start.y;
        let run = self.end.x - self.start.x;

        if run == 0. {
            LineSlope::InfiniteSlope
        } else {
            LineSlope::FiniteSlope(rise / run)
        }
    }

    pub fn y_at(&self, x: f64) -> Option<f64> {
        let denom = self.end.x - self.start.x;
        if denom == 0. {
            None
        } else {
            let frac = (x - self.start.x) / denom;
            Some(frac * self.end.y + (1. - frac) * self.start.y)
        }
    }

    pub fn intersect(&self, other: &Line) -> Option<Point> {
        let self_slope = self.slope().unwrap();
        let other_slope = other.slope().unwrap();
        let net_slope = -self_slope + other_slope;
        let y_delta = self.start.y
            - other
                .y_at(self.start.x)
                .expect("Unhandled vertical line (1).");
        let x_int = self.start.x + (y_delta / net_slope);
        if (self.start.x <= x_int)
            && (x_int <= self.end.x)
            && (other.start.x <= x_int)
            && (x_int <= other.end.x)
        {
            Some(Point::new(
                x_int,
                self.y_at(x_int).expect("Unhandled vertical line (2)."),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation() {
        // Lines are always oriented so that start is left of end.

        let p1 = Point::new(4., 5.);
        let p2 = Point::new(6., 9.);

        let l1 = Line::new(p1, p2);
        assert_eq!(LineOrientation::Top, l1.orientation);
        assert_eq!(p1, l1.start);
        assert_eq!(p2, l1.end);

        let l2 = Line::new(p2, p1);
        assert_eq!(LineOrientation::Bottom, l2.orientation);
        assert_eq!(p1, l2.start);
        assert_eq!(p2, l2.end);
    }

    #[test]
    fn test_finite_slope() {
        let p1 = Point::new(2., 2.);
        let p2 = Point::new(4., 3.);

        let l1 = Line::new(p1, p2);
        assert_eq!(LineSlope::FiniteSlope(0.5), l1.slope());
        let l2 = Line::new(p2, p1);
        assert_eq!(LineSlope::FiniteSlope(0.5), l2.slope());
    }

    #[test]
    fn test_infinite_slope() {
        let p1 = Point::new(5., 6.);
        let p2 = Point::new(5., 3.);

        let l1 = Line::new(p1, p2);
        assert_eq!(LineSlope::InfiniteSlope, l1.slope());
        let l2 = Line::new(p2, p1);
        assert_eq!(LineSlope::InfiniteSlope, l2.slope());
    }

    #[test]
    fn test_y_at() {
        let l1 = Line::new(Point::new(4., 10.), Point::new(14., 20.));

        // Points at endpoints return their values.
        assert_eq!(Some(10.), l1.y_at(4.));
        assert_eq!(Some(20.), l1.y_at(14.));

        // Points between the values are interpolated.
        assert_eq!(Some(11.), l1.y_at(5.));

        // Values outside of the line are interpolated.
        assert_eq!(Some(9.), l1.y_at(3.));
        assert_eq!(Some(21.), l1.y_at(15.));
    }

    #[test]
    fn test_y_at_inf() {
        let p1 = Point::new(5., 6.);
        let p2 = Point::new(5., 3.);

        let l1 = Line::new(p1, p2);
        assert_eq!(None, l1.y_at(4.));
        assert_eq!(None, l1.y_at(5.)); // Behavior TBD.
        assert_eq!(None, l1.y_at(6.));
    }

    #[test]
    fn test_intersect() {
        let l1 = Line::new(Point::new(4., 10.), Point::new(14., 20.));
        let l2 = Line::new(Point::new(0., 20.), Point::new(20., 0.));

        assert_eq!(Some(Point::new(7., 13.)), l1.intersect(&l2));
        assert_eq!(Some(Point::new(7., 13.)), l2.intersect(&l1));
    }
}
