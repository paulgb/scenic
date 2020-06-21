use crate::polygon::Polygon;

pub struct Scene {
    pub polys: Vec<Polygon>
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            polys: Vec::new()
        }
    }

    pub fn add_poly(&mut self, poly: Polygon) {
        self.polys.push(poly)
    }
}
