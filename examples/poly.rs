use scenic::prelude::*;
use scenic::scanlines::{LineEvent, ScanState};
use std::collections::BTreeMap;

pub fn main() {
    let p1 = Polygon::new(
        vec![
            Point::new(5., 10.),
            Point::new(9., 5.),
            Point::new(15., 10.),
            Point::new(11., 15.),
        ],
        1.,
    );
    let p2 = Polygon::new(
        vec![
            Point::new(12., 13.),
            Point::new(14., 10.),
            Point::new(15., 12.),
            Point::new(13., 13.),
        ],
        2.,
    );
    let p3 = Polygon::new(
        vec![Point::new(8., 5.), Point::new(10., 11.), Point::new(6., 8.)],
        3.,
    );
    let p4 = Polygon::new(
        vec![
            Point::new(12., 7.),
            Point::new(7., 13.),
            Point::new(8., 9.),
            Point::new(6., 5.),
        ],
        4.,
    );

    let mut scene = Scene::new();
    scene.add_poly(p1);
    scene.add_poly(p2);
    scene.add_poly(p3);
    scene.add_poly(p4);

    let mut scan_state = ScanState::new(&scene);
    let mut final_lines: Vec<Line> = Vec::new();
    let mut cur_lines: BTreeMap<&Line, Point> = BTreeMap::new();

    let mut i = 1;
    loop {
        let lines = scan_state.step();
        let pointer = scan_state.pointer.unwrap();

        let mut d = DebugDraw::new();

        for (line, line_event) in lines {
            match line_event {
                LineEvent::Begin => {
                    cur_lines.insert(line, pointer);
                }
                LineEvent::End => {
                    if let Some(from_point) = cur_lines.get(line) {
                        final_lines.push(Line::new(*from_point, pointer));
                        cur_lines.remove(line);
                    }
                }
            }
        }

        d.add_scan_state(&scan_state);

        for line in &final_lines {
            d.add_line(line).stroke("black");
        }

        d.save(&format!("step_{:0>3}.svg", i));
        i += 1;
    }
}
