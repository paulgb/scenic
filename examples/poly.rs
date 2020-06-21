use scenic::prelude::*;

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

    let mut d = DebugDraw::new();
    d.add_scene(&scene);
}
