use svg::node::element;
use svg::node::element::path::Data;
use svg::node::Text;
use svg::Document;
use svg::Node;

use crate::line::Line;
use crate::point::Point;
use crate::polygon::Polygon;
use crate::scanlines::{LineEvent, ScanState, SceneEvent};
use crate::scene::Scene;

const MARGIN: f64 = 0.1;
const STROKE_WIDTH: usize = 2;
const STROKE: &str = "#ddd";
const POLY_FILL: &str = "none";
const POINT_FILL: &str = "red";
const POINT_RADIUS: f64 = 0.15;

// For SceneEvent renderer.
const VERTEX_EVENT_FILL: &str = "red";
const POINTER_FILL: &str = "blue";
const INTERSECTION_START_EVENT_FILL: &str = "purple";
const INTERSECTION_ENG_EVENT_FILL: &str = "orange";

#[derive(Clone)]
struct Bounds {
    // Top = min Y and Bottom = max Y because SVG uses screen coords.
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
}

pub struct DebugGroupBuilder<'a, T>
where
    T: Node,
{
    debug_draw: &'a mut DebugDraw,
    element: Option<T>,
}

impl<'a, T> DebugGroupBuilder<'a, T>
where
    T: Node,
{
    pub fn new(debug_draw: &'a mut DebugDraw, element: T) -> DebugGroupBuilder<'a, T> {
        DebugGroupBuilder {
            debug_draw,
            element: Some(element),
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

impl<'a, T> Drop for DebugGroupBuilder<'a, T>
where
    T: Node,
{
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

    fn line(&mut self, line: &Line) -> element::Line {
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
        svg_line
    }

    pub fn add_line<'a>(&'a mut self, line: &Line) -> DebugGroupBuilder<'a, element::Line> {
        let svg_line = self.line(line);
        DebugGroupBuilder::new(self, svg_line)
    }

    fn polygon_to_path(&mut self, poly: &Polygon) -> element::Path {
        let mut data = Data::new();

        data = data.move_to(poly.points[0].coords());
        self.update_bounds(poly.points[0]);
        for point in &poly.points[1..] {
            data = data.line_to(point.coords());
            self.update_bounds(*point);
        }
        data = data.close();

        element::Path::new()
            .set("d", data)
            .set("vector-effect", "non-scaling-stroke")
            .set("stroke-width", STROKE_WIDTH)
            .set("fill", POLY_FILL)
            .set("stroke", STROKE)
    }

    pub fn add_poly<'a>(&'a mut self, poly: &Polygon) -> DebugGroupBuilder<'a, element::Path> {
        let path = self.polygon_to_path(poly);

        DebugGroupBuilder::new(self, path)
    }

    pub fn add_scene<'a>(&'a mut self, scene: &Scene) -> DebugGroupBuilder<'a, element::Group> {
        let mut group = element::Group::new();

        for poly in &scene.polys {
            let path = self.polygon_to_path(&poly);
            group = group.add(path);
        }

        DebugGroupBuilder::new(self, group)
    }

    fn point_circle(&mut self, point: Point, fill: &str) -> element::Circle {
        self.update_bounds(point);
        element::Circle::new()
            .set("cx", point.x)
            .set("cy", point.y)
            .set("fill", fill)
            .set("r", POINT_RADIUS)
    }

    pub fn add_scan_state<'a>(
        &'a mut self,
        state: &ScanState,
    ) -> DebugGroupBuilder<'a, element::Group> {
        let mut group: element::Group = element::Group::new();

        let mut queue_group: element::Group = element::Group::new();

        for event in &state.events {
            let g = match event {
                SceneEvent::VertexEvent(v) => {
                    let mut g = element::Group::new();

                    if !v.start_lines.is_empty() {
                        let mut start_group = element::Group::new().set("class", "starting");
                        for line in &v.start_lines {
                            start_group = start_group.add(self.line(line));
                        }
                        g = g.add(start_group);
                    }

                    if !v.end_lines.is_empty() {
                        let mut end_group = element::Group::new().set("class", "ending");
                        for line in &v.end_lines {
                            end_group = end_group.add(self.line(line));
                        }
                        g = g.add(end_group);
                    }

                    g = g.add(self.point_circle(v.point, VERTEX_EVENT_FILL));

                    g
                }
                SceneEvent::IntersectionEvent(p, line, line_event) => {
                    let g = element::Group::new();
                    // g
                    unimplemented!()
                }
            };

            queue_group = queue_group.add(g);
        }

        if let Some(p) = state.pointer {
            queue_group = queue_group.add(self.point_circle(p, POINTER_FILL))
        }

        group = group.add(queue_group);

        DebugGroupBuilder::new(self, group)
    }

    pub fn save(&mut self, filename: &str) {
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
        svg::save(filename, &doc).expect("Error writing.");
    }
}
