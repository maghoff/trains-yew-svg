#![recursion_limit = "256"]

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::SvgsvgElement;
use yew::services::ConsoleService;
use yew::{html, utils::document, Component, ComponentLink, Html, ShouldRender};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const AXIAL_DIRECTION: [(i32, i32); 6] = [(-1, 0), (0, -1), (1, -1), (1, 0), (0, 1), (-1, 1)];
const PLANAR_DIRECTION: [(f32, f32); 6] = [
    (-0.8660254037844386, -0.5),
    (0., -1.0),
    (0.8660254037844384, -0.5),
    (0.8660254037844387, 0.5),
    (0., 1.0),
    (-0.8660254037844387, 0.5),
];

#[derive(Copy, Clone)]
struct Edge {
    rail_connection: bool,
}

#[derive(Copy, Clone)]
struct Cell {
    edge: [Edge; 3],
}

#[derive(Copy, Clone)]
struct Map {
    cells: [[Cell; 7]; 7],
}

fn plus(a: (i32, i32), b: (i32, i32)) -> (i32, i32) {
    (a.0 + b.0, a.1 + b.1)
}

impl Map {
    fn new() -> Map {
        Map {
            cells: [[Cell {
                edge: [Edge {
                    rail_connection: false,
                }; 3],
            }; 7]; 7],
        }
    }

    fn cell(&self, (q, r): (i32, i32)) -> &Cell {
        assert!(q >= -3);
        assert!(r >= -3);
        &self.cells[(q + 3) as usize][(r + 3) as usize]
    }

    fn maybe_cell(&self, (q, r): (i32, i32)) -> Option<&Cell> {
        if q >= -3 && q <= 3 && r >= -3 && r <= 3 {
            Some(self.cell((q, r)))
        } else {
            None
        }
    }

    fn cell_mut(&mut self, (q, r): (i32, i32)) -> &mut Cell {
        assert!(q >= -3);
        assert!(r >= -3);
        &mut self.cells[(q + 3) as usize][(r + 3) as usize]
    }

    fn edges(&self, c: (i32, i32)) -> [Edge; 6] {
        [
            self.cell(c).edge[0],
            self.cell(c).edge[1],
            self.cell(c).edge[2],
            self.maybe_cell(plus(c, AXIAL_DIRECTION[3]))
                .map(|x| x.edge[0])
                .unwrap_or(Edge {
                    rail_connection: false,
                }),
            self.maybe_cell(plus(c, AXIAL_DIRECTION[4]))
                .map(|x| x.edge[1])
                .unwrap_or(Edge {
                    rail_connection: false,
                }),
            self.maybe_cell(plus(c, AXIAL_DIRECTION[5]))
                .map(|x| x.edge[2])
                .unwrap_or(Edge {
                    rail_connection: false,
                }),
        ]
    }

    fn maybe_edge_mut(&mut self, (q, r): (i32, i32), edge: usize) -> Option<&mut Edge> {
        if edge < 3 {
            if q >= -3 && q <= 3 && r >= -3 && r <= 3 {
                Some(&mut self.cell_mut((q, r)).edge[edge])
            } else {
                None
            }
        } else {
            self.maybe_edge_mut(plus((q, r), AXIAL_DIRECTION[edge]), edge - 3)
        }
    }
}

struct App {
    map: Map,
    highlight: Option<(i32, i32, i32)>,

    link: ComponentLink<Self>,
    console: ConsoleService,
}

enum Msg {
    MouseMove(f32, f32),
    MouseLeave,
    MouseClick(f32, f32),
}

fn stub(dir: i32, ghost: bool) -> Html {
    straight_len(dir, 10., ghost)
}

fn straight(dir: i32, ghost: bool) -> Html {
    straight_len(dir, 52., ghost)
}

fn straight_len(dir: i32, len: f32, ghost: bool) -> Html {
    let start_x = 26. * PLANAR_DIRECTION[dir as usize].0;
    let start_y = 26. * PLANAR_DIRECTION[dir as usize].1;
    let end_x = (26. - len) * PLANAR_DIRECTION[dir as usize].0;
    let end_y = (26. - len) * PLANAR_DIRECTION[dir as usize].1;

    let dx = -PLANAR_DIRECTION[dir as usize].1;
    let dy = PLANAR_DIRECTION[dir as usize].0;

    let dist = 10.;

    let class = if ghost { "rails ghost" } else { "rails" };

    html! {
        <>
            <line
                class={class}
                x1={start_x - dist * dx}
                y1={start_y - dist * dy}
                x2={end_x - dist * dx}
                y2={end_y - dist * dy}
            />
            <line
                class={class}
                x1={start_x + dist * dx}
                y1={start_y + dist * dy}
                x2={end_x + dist * dx}
                y2={end_y + dist * dy}
            />
        </>
    }
}

fn bend(dir: i32, ghost: bool) -> Html {
    let x1 = 26. * PLANAR_DIRECTION[((dir + 5) % 6) as usize].0;
    let y1 = 26. * PLANAR_DIRECTION[((dir + 5) % 6) as usize].1;
    let x2 = 26. * PLANAR_DIRECTION[((dir + 1) % 6) as usize].0;
    let y2 = 26. * PLANAR_DIRECTION[((dir + 1) % 6) as usize].1;

    let dx1 = -PLANAR_DIRECTION[((dir + 5) % 6) as usize].1;
    let dy1 = PLANAR_DIRECTION[((dir + 5) % 6) as usize].0;
    let dx2 = -PLANAR_DIRECTION[((dir + 1) % 6) as usize].1;
    let dy2 = PLANAR_DIRECTION[((dir + 1) % 6) as usize].0;

    let dist = 10.;

    let d = format!(
        "M{},{} A{},{} 0 0 0 {},{} M{},{} A{},{} 0 0 0 {},{}",
        x1 - dist * dx1,
        y1 - dist * dy1,
        45. + dist,
        45. + dist,
        x2 + dist * dx2,
        y2 + dist * dy2,
        x1 + dist * dx1,
        y1 + dist * dy1,
        45. - dist,
        45. - dist,
        x2 - dist * dx2,
        y2 - dist * dy2,
    );

    let class = if ghost { "rails ghost" } else { "rails" };

    html! {
        <path class={class} d={ d } />
    }
}

fn cube_to_axial((x, _y, z): (f32, f32, f32)) -> (f32, f32) {
    (x, z)
}

fn axial_to_cube((q, r): (f32, f32)) -> (f32, f32, f32) {
    (q, -q - r, r)
}

fn cube_round((x, y, z): (f32, f32, f32)) -> (f32, f32, f32) {
    let mut rx = x.round();
    let mut ry = y.round();
    let mut rz = z.round();

    let x_diff = (rx - x).abs();
    let y_diff = (ry - y).abs();
    let z_diff = (rz - z).abs();

    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }

    (rx, ry, rz)
}

fn hex_round(hex: (f32, f32)) -> (f32, f32) {
    cube_to_axial(cube_round(axial_to_cube(hex)))
}

fn pixel_to_flat_hex((x, y): (f32, f32)) -> (i32, i32) {
    use std::f32;
    let size = 30.;
    let q = (2. / 3. * x as f32) / size;
    let r = (-1. / 3. * x as f32 + (3. as f32).sqrt() / 3. * y as f32) / size;
    let (q, r) = hex_round((q, r));
    (q as _, r as _)
}

impl App {
    fn hex(&self, q: i32, r: i32) -> Html {
        let edges = self.map.edges((q, r));

        let x = q * 45;
        let y = q * 26 + r * 52;

        let h = if let Some(highlight) = self.highlight {
            let highlight2 = (
                highlight.0 + AXIAL_DIRECTION[highlight.2 as usize].0,
                highlight.1 + AXIAL_DIRECTION[highlight.2 as usize].1,
                ((highlight.2 + 3) % 6) as _,
            );

            if q == highlight.0 && r == highlight.1 {
                Some(highlight.2)
            } else if q == highlight2.0 && r == highlight2.1 {
                Some(highlight2.2)
            } else {
                None
            }
        } else {
            None
        };

        let mut connections = [0; 6];
        for i in 0..6 {
            connections[i] = if edges[i].rail_connection { 1 } else { 0 };
        }

        if let Some(h) = h {
            connections[h as usize] |= 2;
        }

        let dots = (0..6)
            .into_iter()
            .filter(|dir| {
                (connections[*dir as usize] != 0)
                    && (connections[((dir + 2) % 6) as usize] == 0)
                    && (connections[((dir + 3) % 6) as usize] == 0)
                    && (connections[((dir + 4) % 6) as usize] == 0)
            })
            .map(|dir| (dir, connections[dir as usize] & 1 == 0))
            .map(|(dir, ghost)| stub(dir, ghost))
            .collect::<Html>();

        let straights = (0..3)
            .into_iter()
            .filter(|dir| {
                (connections[*dir as usize] != 0) && (connections[((dir + 3) % 6) as usize] != 0)
            })
            .map(|dir| {
                (
                    dir,
                    (connections[dir as usize] & connections[((dir + 3) % 6) as usize]) & 1 == 0,
                )
            })
            .map(|(dir, ghost)| straight(dir, ghost))
            .collect::<Html>();

        let bends = (0..6)
            .into_iter()
            .filter(|dir| {
                (connections[((dir + 5) % 6) as usize] != 0)
                    && (connections[((dir + 1) % 6) as usize] != 0)
            })
            .map(|dir| {
                (
                    dir,
                    (connections[((dir + 5) % 6) as usize] & connections[((dir + 1) % 6) as usize])
                        & 1
                        == 0,
                )
            })
            .map(|(dir, ghost)| bend(dir, ghost))
            .collect::<Html>();

        html! {
            <g transform={ format!("translate({},{})", 500 + x, 500 + y) }>
                <polygon
                    class="hex-background"
                    points="-15,-26 15,-26 30,0 15,26 -15,26 -30,0"
                />
                <polygon
                    class={ if Some(0) == h { "hex-edge highlight" } else { "hex-edge" } }
                    points="-18,0 -30,0 -15,-26 -9,-15.6"
                />
                <polygon
                    class={ if Some(1) == h { "hex-edge highlight" } else { "hex-edge" } }
                    points="-9,-15.6 -15,-26 15,-26 9,-15.6"
                />
                <polygon
                    class={ if Some(2) == h { "hex-edge highlight" } else { "hex-edge" } }
                    points="9,-15.6 15,-26 30,0 18,0"
                />
                <polygon
                    class={ if Some(3) == h { "hex-edge highlight" } else { "hex-edge" } }
                    points="18,0 30,0 15,26 9,15.6"
                />
                <polygon
                    class={ if Some(4) == h { "hex-edge highlight" } else { "hex-edge" } }
                    points="9,15.6 15,26 -15,26 -9,15.6"
                />
                <polygon
                    class={ if Some(5) == h { "hex-edge highlight" } else { "hex-edge" } }
                    points="-9,15.6 -15,26 -30,0 -18,0"
                />
                <polygon
                    class="hex-foreground"
                    points="-15,-26 15,-26 30,0 15,26 -15,26 -30,0"
                />
                { dots }
                { straights }
                { bends }
            </g>
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            map: Map::new(),
            highlight: None,
            link,
            console: ConsoleService::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        fn edge_from_coord(x: f32, y: f32) -> Option<(i32, i32, i32)> {
            let (q, r) = pixel_to_flat_hex((x, y));

            let rx = x - (q * 45) as f32;
            let ry = y - (q * 26 + r * 52) as f32;

            let a = (PLANAR_DIRECTION[0].0 * rx + PLANAR_DIRECTION[0].1 * ry) / 26.;
            let b = (PLANAR_DIRECTION[1].0 * rx + PLANAR_DIRECTION[1].1 * ry) / 26.;
            let c = (PLANAR_DIRECTION[2].0 * rx + PLANAR_DIRECTION[2].1 * ry) / 26.;
            let edge_proximities = [a, b, c, -a, -b, -c];

            let (index, _) = edge_proximities
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap();

            if edge_proximities[index] > 0.6 {
                Some((q, r, index as _))
            } else {
                None
            }
        }

        match msg {
            Msg::MouseMove(x, y) => {
                self.highlight = edge_from_coord(x, y);
                true
            }
            Msg::MouseLeave => {
                self.highlight = None;
                true
            }
            Msg::MouseClick(x, y) => edge_from_coord(x, y)
                .and_then(|(q, r, edge)| self.map.maybe_edge_mut((q, r), edge as _))
                .map(|edge| {
                    edge.rail_connection = !edge.rail_connection;
                    true
                })
                .unwrap_or(false),
        }
    }

    fn view(&self) -> Html {
        use std::cmp::{max, min};

        let coords = (-3..4).into_iter().flat_map(|q| {
            (max(-3, -3 - q)..min(4, 4 - q))
                .into_iter()
                .map(move |r| (q, r))
        });

        fn coord_from_ev(ev: &yew::MouseEvent) -> (f32, f32) {
            let svg: SvgsvgElement = document()
                .query_selector("svg")
                .unwrap()
                .expect("Should find the svg element")
                .dyn_into()
                .expect("Should have type svg");
            let pt = svg.create_svg_point();
            pt.set_x(ev.client_x() as _);
            pt.set_y(ev.client_y() as _);
            let transform_point = svg.get_screen_ctm().unwrap().inverse().unwrap();
            let tr = pt.matrix_transform(&transform_point);
            (tr.x(), tr.y())
        }

        html! {
            <>
                <svg
                    viewBox="0 0 1000 1000"
                    style="width: 1000px; height: 1000px"
                    onmousemove=self.link.callback(|ev: yew::MouseEvent| {
                        let coord = coord_from_ev(&ev);
                        Msg::MouseMove(coord.0 - 500., coord.1 - 500.)
                    })
                    onmouseleave=self.link.callback(|_| Msg::MouseLeave)
                    onclick=self.link.callback(|ev: yew::MouseEvent| {
                        let coord = coord_from_ev(&ev);
                        Msg::MouseClick(coord.0 - 500., coord.1 - 500.)
                    })
                >
                    { coords.into_iter().map(|(q, r)| {
                        self.hex(q, r)
                    }).collect::<Html>() }
                </svg>
            </>
        }
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<App>();

    Ok(())
}
