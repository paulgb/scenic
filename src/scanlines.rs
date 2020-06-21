use crate::line::Line;
use crate::point::Point;
use crate::scene::Scene;
use crate::vertex::Vertex;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(PartialEq, PartialOrd, Ord, Eq)]
pub enum LineEvent {
    Begin,
    End,
}

#[derive(PartialEq, Ord, Eq)]
pub enum SceneEvent<'a> {
    VertexEvent(Vertex<'a>),
    IntersectionEvent(Point, &'a Line, LineEvent),
}

impl<'a> PartialOrd for SceneEvent<'a> {
    // Ordering is inverted because PriorityQueue is a max queue and we want a
    // min queue.
    fn partial_cmp(&self, other: &SceneEvent) -> Option<Ordering> {
        match other.point().cmp(&self.point()) {
            Ordering::Equal => match self {
                SceneEvent::VertexEvent(vs) => match other {
                    SceneEvent::VertexEvent(vo) => vo.partial_cmp(vs),
                    SceneEvent::IntersectionEvent(_, _, _) => Some(Ordering::Less),
                },
                SceneEvent::IntersectionEvent(_, ls, es) => match other {
                    SceneEvent::VertexEvent(_) => Some(Ordering::Greater),
                    SceneEvent::IntersectionEvent(_, lo, eo) => (lo, eo).partial_cmp(&(ls, es)),
                },
            },
            ord => Some(ord),
        }
    }
}

impl<'a> SceneEvent<'a> {
    pub fn point(&self) -> Point {
        match &self {
            SceneEvent::VertexEvent(v) => v.point,
            SceneEvent::IntersectionEvent(p, _, _) => *p,
        }
    }
}

pub struct ScanState<'a> {
    pub pointer: Option<Point>,
    pub events: BinaryHeap<SceneEvent<'a>>,
}

type StepResult<'a> = Vec<(&'a Line, LineEvent)>;

impl<'a> ScanState<'a> {
    pub fn step(&mut self) -> StepResult<'a> {
        let event = self.events.pop();
        if let Some(e) = event {
            self.pointer = Some(e.point());

            match e {
                SceneEvent::VertexEvent(v) => {
                    let mut vs: StepResult =
                        Vec::with_capacity(v.start_lines.len() + v.end_lines.len());

                    for &line in &v.start_lines {
                        vs.push((line, LineEvent::Begin));
                    }
                    for &line in &v.end_lines {
                        vs.push((line, LineEvent::End));
                    }

                    vs
                }
                SceneEvent::IntersectionEvent(_, line, line_event) => vec![(line, line_event)],
            }
        } else {
            self.pointer = None;
            Vec::new()
        }
    }

    pub fn done(&self) -> bool {
        self.events.is_empty()
    }

    pub fn new(scene: &Scene) -> ScanState {
        let vertices = scene.vertices();
        let mut events = BinaryHeap::with_capacity(vertices.len());

        for vertex in vertices {
            events.push(SceneEvent::VertexEvent(vertex));
        }

        ScanState {
            pointer: None,
            events,
        }
    }
}
