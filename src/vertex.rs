use crate::line::Line;
use crate::point::Point;
use crate::polygon::Polygon;
use std::collections::BTreeSet;

type LineCollection<'a> = BTreeSet<&'a Line>;

pub struct Vertex<'a> {
    pub point: Point,
    pub start_lines: LineCollection<'a>,
    pub end_lines: LineCollection<'a>,
}

impl<'a> Vertex<'a> {
    pub fn new(point: Point) -> Vertex<'a> {
        Vertex {
            point,
            start_lines: LineCollection::new(),
            end_lines: LineCollection::new(),
        }
    }
    /*
    pub fn new(point: Point, start_lines: LineCollection<'a>, end_lines: LineCollection<'a>) -> Vertex<'a> {
        Vertex {
            point,
            start_lines,
            end_lines
        }
    }
    */
}
