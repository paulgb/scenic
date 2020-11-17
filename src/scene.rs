use crate::point::Point;
use crate::polygon::Polygon;
use crate::vertex::Vertex;
use std::collections::BTreeMap;

/// A container that owns multiple polygons.
pub struct Scene {
    pub polys: Vec<Polygon>,
}

impl<'a> Scene {
    pub fn new() -> Scene {
        Scene { polys: Vec::new() }
    }

    pub fn add_poly(&mut self, poly: Polygon) {
        self.polys.push(poly)
    }

    /// Return vertices associated with the polygons in this scene
    /// by iterating over the lines in each polygon.
    pub fn vertices(&'a self) -> Vec<Vertex<'a>> {
        let mut vertices: BTreeMap<Point, Vertex> = BTreeMap::new();

        for poly in &self.polys {
            for line in &poly.lines {
                // Add vertex for start point.
                vertices
                    .entry(line.start)
                    .or_insert_with(|| Vertex::new(line.start))
                    .start_lines
                    .insert(line);

                // Add vertex for end point.
                vertices
                    .entry(line.end)
                    .or_insert_with(|| Vertex::new(line.end))
                    .end_lines
                    .insert(line);
            }
        }

        vertices.into_iter().map(|(_, v)| v).collect()
    }
}

impl Default for Scene {
    fn default() -> Scene {
        Scene::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertices() {
        let p1 = Point::new(5., 5.);
        let p2 = Point::new(10., 10.);
        let p3 = Point::new(15., 5.);
        let p4 = Point::new(10., 0.);

        let poly = Polygon::new(vec![p1, p2, p3, p4], 1.);
        let mut scene = Scene::new();
        scene.add_poly(poly);
        let verts = scene.vertices();

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
