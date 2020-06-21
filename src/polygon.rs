use crate::line::Line;
use crate::point::Point;
use crate::vertex::Vertex;
use std::collections::{BTreeMap, BTreeSet};

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

    pub fn vertices(&'a self) -> Vec<Vertex<'a>> {
        let mut vertices: BTreeMap<Point, Vertex> = BTreeMap::new();

        for line in &self.lines {
            vertices
                .entry(line.start)
                .or_insert_with(|| Vertex::new(line.start))
                .start_lines
                .insert(line);

            vertices
                .entry(line.end)
                .or_insert_with(|| Vertex::new(line.end))
                .end_lines
                .insert(line);
        }

        //vertices.values().collect()
        vertices.into_iter().map(|(_, v)| v).collect()
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
        assert_eq!(LineOrientation::Bottom, lines[0].orientation);

        assert_eq!(p1, lines[1].start);
        assert_eq!(p2, lines[1].end);
        assert_eq!(LineOrientation::Top, lines[1].orientation);

        assert_eq!(p2, lines[2].start);
        assert_eq!(p3, lines[2].end);
        assert_eq!(LineOrientation::Top, lines[2].orientation);

        assert_eq!(p4, lines[3].start);
        assert_eq!(p3, lines[3].end);
        assert_eq!(LineOrientation::Bottom, lines[3].orientation);
    }

    #[test]
    fn test_vertices() {
        let p1 = Point::new(5., 5.);
        let p2 = Point::new(10., 10.);
        let p3 = Point::new(15., 5.);
        let p4 = Point::new(10., 0.);

        let poly = Polygon::new(vec![p1, p2, p3, p4], 1.);
        let verts = poly.vertices();

        assert_eq!(p1, verts[0].point);
        assert_eq!(2, verts[0].start_lines.len());
        assert_eq!(0, verts[0].end_lines.len());
        assert_eq!(p4, verts[1].point);
        assert_eq!(1, verts[1].start_lines.len());
        assert_eq!(1, verts[1].end_lines.len());
        assert_eq!(p2, verts[2].point);
        assert_eq!(1, verts[2].start_lines.len());
        assert_eq!(1, verts[2].end_lines.len());
        assert_eq!(p3, verts[3].point);
        assert_eq!(0, verts[3].start_lines.len());
        assert_eq!(2, verts[3].end_lines.len());
    }
}
