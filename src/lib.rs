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
}

struct App {
    map: Map,
    highlight: Option<(i32, i32)>,

    link: ComponentLink<Self>,
    console: ConsoleService,
    value: i64,
}

enum Msg {
    Increment,
    MouseMove(f32, f32),
    MouseLeave,
}

fn circle(dir: i32) -> Html {
    let cx = 20. * PLANAR_DIRECTION[dir as usize].0;
    let cy = 20. * PLANAR_DIRECTION[dir as usize].1;

    html! {
        <circle cx={cx} cy={cy} r=5 />
    }
}

fn straight(dir: i32) -> Html {
    let start_x = 26. * PLANAR_DIRECTION[dir as usize].0;
    let start_y = 26. * PLANAR_DIRECTION[dir as usize].1;
    let end_x = -start_x;
    let end_y = -start_y;

    html! {
        <line class="rails" x1={start_x} y1={start_y} x2={end_x} y2={end_y} />
    }
}

fn bend(dir: i32) -> Html {
    let x1 = 26. * PLANAR_DIRECTION[((dir + 5) % 6) as usize].0;
    let y1 = 26. * PLANAR_DIRECTION[((dir + 5) % 6) as usize].1;
    let x2 = 26. * PLANAR_DIRECTION[((dir + 1) % 6) as usize].0;
    let y2 = 26. * PLANAR_DIRECTION[((dir + 1) % 6) as usize].1;

    let d = format!("M{},{} A45,45 0 0 0 {},{} ", x1, y1, x2, y2);

    html! {
        <path class="rails" d={ d } />
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

fn pixel_to_flat_hex((x, y): (f32, f32)) -> (f32, f32) {
    use std::f32;
    let size = 30.;
    let q = (2. / 3. * x as f32) / size;
    let r = (-1. / 3. * x as f32 + (3. as f32).sqrt() / 3. * y as f32) / size;
    hex_round((q, r))
}

impl App {
    fn hex(&self, q: i32, r: i32, highlight: bool) -> Html {
        let edges = self.map.edges((q, r));

        let x = q * 45;
        let y = q * 26 + r * 52;

        let dots = (0..6)
            .into_iter()
            .filter(|dir| {
                (edges[*dir as usize].rail_connection
                    && !edges[((dir + 2) % 6) as usize].rail_connection
                    && !edges[((dir + 3) % 6) as usize].rail_connection
                    && !edges[((dir + 4) % 6) as usize].rail_connection)
            })
            .map(|dir| circle(dir))
            .collect::<Html>();

        let straights = (0..3)
            .into_iter()
            .filter(|dir| {
                (edges[*dir as usize].rail_connection
                    && edges[((dir + 3) % 6) as usize].rail_connection)
            })
            .map(|dir| straight(dir))
            .collect::<Html>();

        let bends = (0..6)
            .into_iter()
            .filter(|dir| {
                (edges[((dir + 5) % 6) as usize].rail_connection
                    && edges[((dir + 1) % 6) as usize].rail_connection)
            })
            .map(|dir| bend(dir))
            .collect::<Html>();

        let class = if highlight { "hex highlight" } else { "hex" };

        html! {
            <g transform={ format!("translate({},{})", 500 + x, 500 + y) }>
                <path
                    class={ class }
                    d="m-15,-26 l30,0 l15,26 l-15,26 l-30,0 l-15,-26 z"
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
        let mut map = Map::new();
        map.cell_mut((0, 1)).edge[1].rail_connection = true;
        map.cell_mut((0, 0)).edge[2].rail_connection = true;
        map.cell_mut((2, -1)).edge[0].rail_connection = true;
        map.cell_mut((2, 0)).edge[1].rail_connection = true;
        map.cell_mut((1, 1)).edge[0].rail_connection = true;
        map.cell_mut((1, 1)).edge[2].rail_connection = true;

        map.cell_mut((0, 0)).edge[1].rail_connection = true;
        map.cell_mut((0, 2)).edge[1].rail_connection = true;
        map.cell_mut((0, 2)).edge[2].rail_connection = true;

        App {
            map,
            highlight: None,
            link,
            console: ConsoleService::new(),
            value: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Increment => {
                self.value = self.value + 1;
                self.console.log("plus one");
                true
            }
            Msg::MouseMove(x, y) => {
                let (q, r) = pixel_to_flat_hex((x, y));
                let (q, r) = (q as i32, r as i32);
                self.highlight = Some((q, r));
                self.console
                    .log(&format!("mousemove({}, {}) => {}, {}", x, y, q, r));
                true
            }
            Msg::MouseLeave => {
                self.console.log("leave");
                self.highlight = None;
                true
            }
        }
    }

    fn view(&self) -> Html {
        let button_text = self.value.to_string();

        use std::cmp::{max, min};

        let coords = (-3..4).into_iter().flat_map(|q| {
            (max(-3, -3 - q)..min(4, 4 - q))
                .into_iter()
                .map(move |r| (q, r))
        });

        html! {
            <>
                <svg
                    viewBox="0 0 1000 1000"
                    style="width: 1000px; height: 1000px"
                    onmousemove=self.link.callback(|ev: yew::MouseEvent| {
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
                        Msg::MouseMove(tr.x() - 500., tr.y() - 500.)
                    })
                    onmouseleave=self.link.callback(|_| Msg::MouseLeave)
                >
                    { coords.into_iter().map(|(q, r)| {
                        let highlight = self.highlight
                            .map(|(q1, r1)| q1 == q && r1 == r)
                            .unwrap_or(false);
                        self.hex(q, r, highlight)
                    }).collect::<Html>() }
                </svg>
                <button onmousemove=self.link.callback(|_| Msg::Increment)>{ button_text }</button>
            </>
        }
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<App>();

    Ok(())
}
