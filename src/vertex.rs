use crate::line::Line;
use crate::point::Point;
use std::collections::BTreeSet;

type LineCollection<'a> = BTreeSet<&'a Line>;

/// Represents a point in space at which at least one line starts
/// or ends. Multiple lines can start and end at the same vertex.
#[derive(PartialEq, PartialOrd, Ord, Eq)]
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
}
