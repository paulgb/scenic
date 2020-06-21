use svg::node::element;
use svg::node::element::path::Data;
use svg::node::Text;
use svg::Node;
use svg::Document;

use crate::line::Line;
use crate::point::Point;
use crate::polygon::Polygon;

const MARGIN: f64 = 0.1;
const STROKE_WIDTH: usize = 2;
const STROKE: &str = "black";
const POLY_FILL: &str = "none";
const POINT_FILL: &str = "red";

#[derive(Clone)]
struct Bounds {
    // Top = min Y and Bottom = max Y because SVG uses screen coords.
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
}

pub struct DebugGroupBuilder<'a, T> where T: Node {
    debug_draw: &'a mut DebugDraw,
    element: Option<T>,
}

impl<'a, T> DebugGroupBuilder<'a, T> where T: Node {
    pub fn new(debug_draw: &'a mut DebugDraw, element: T) -> DebugGroupBuilder<'a, T> {
        DebugGroupBuilder {
            debug_draw,
            element: Some(element)
        }
    }

    pub fn note(&mut self, note: &str) -> &mut Self {
        let title = element::Title::new().add(Text::new(note));
        let mut element = self.element.take().unwrap();
        element.append(title);
        self.element = Some(element);
        self
    }

    pub fn stroke(&mut self, color: &str) -> &mut Self {
        let mut element = self.element.take().unwrap();
        element.assign("stroke", color);
        self.element = Some(element);
        self
    }
}

impl<'a, T> Drop for DebugGroupBuilder<'a, T> where T: Node {
    fn drop(&mut self) {
        let mut doc = self.debug_draw.doc.take().unwrap();
        doc.append(self.element.take().unwrap());
        self.debug_draw.doc = Some(doc);
    }
}

#[derive(Clone)]
pub struct DebugDraw {
    doc: Option<Document>,
    bounds: Option<Bounds>,
}

impl Drop for DebugDraw {
    fn drop(&mut self) {
        let bounds = self.bounds.take().expect("No bounds, empty DebugDraw?");

        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        let view_box = format!(
            "{} {} {} {}",
            bounds.left - width * MARGIN,
            bounds.top - height * MARGIN,
            width * (1. + 2. * MARGIN),
            height * (1. + 2. * MARGIN)
        );
        let mut doc = self.doc.take().unwrap();
        doc = doc.set("viewBox", view_box);
        svg::save("debug.svg", &doc).expect("Error writing.");
    }
}

impl DebugDraw {
    pub fn new() -> DebugDraw {
        DebugDraw {
            doc: Some(Document::new()),
            bounds: None,
        }
    }

    pub fn update_bounds(&mut self, p: Point) {
        self.bounds = Some(match self.bounds {
            None => Bounds {
                top: p.y,
                bottom: p.y,
                left: p.x,
                right: p.x,
            },
            Some(Bounds {
                top,
                bottom,
                left,
                right,
            }) => Bounds {
                top: top.min(p.y),
                bottom: bottom.max(p.y),
                left: left.min(p.x),
                right: right.max(p.x),
            },
        })
    }

    pub fn add_point<'a>(&'a mut self, point: Point) -> DebugGroupBuilder<'a, element::Circle> {
        let c = element::Circle::new()
            .set("cx", point.x)
            .set("cy", point.y)
            .set("r", 0.1)
            .set("fill", POINT_FILL);
        self.update_bounds(point);

        DebugGroupBuilder::new(self, c)
    }

    pub fn add_line<'a, T: std::fmt::Debug>(&'a mut self, line: &Line<T>) -> DebugGroupBuilder<'a, element::Line> {
        let svg_line = element::Line::new()
            .set("x1", line.start.x)
            .set("y1", line.start.y)
            .set("x2", line.end.x)
            .set("y2", line.end.y)
            .set("vector-effect", "non-scaling-stroke")
            .set("stroke-width", STROKE_WIDTH)
            .set("stroke", STROKE);
        self.update_bounds(line.start);
        self.update_bounds(line.end);

        DebugGroupBuilder::new(self, svg_line)
    }

    pub fn add_poly<'a>(&'a mut self, poly: &Polygon) -> DebugGroupBuilder<'a, element::Path> {
        let mut data = Data::new();

        data = data.move_to(poly.points[0].coords());
        self.update_bounds(poly.points[0]);
        for point in &poly.points[1..] {
            data = data.line_to(point.coords());
            self.update_bounds(*point);
        }
        data = data.close();

        let path = element::Path::new()
            .set("d", data)
            .set("vector-effect", "non-scaling-stroke")
            .set("stroke-width", STROKE_WIDTH)
            .set("fill", POLY_FILL)
            .set("stroke", STROKE);
        
        DebugGroupBuilder::new(self, path)
        //self.doc = Some(self.doc.take().unwrap().add(path));
    }
}
