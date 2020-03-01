use wasm_bindgen::prelude::*;
use yew::{html, Callback, ClickEvent, Component, ComponentLink, Html, ShouldRender};

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
    clicked: bool,
    onclick: Callback<ClickEvent>,
    map: Map,
}

enum Msg {
    Click,
}

impl App {
    fn hex(&self, q: i32, r: i32) -> Html {
        let edges = self.map.edges((q, r));

        let x = q * 45;
        let y = q * 26 + r * 52;

        fn circle(dir: i32) -> Html {
            let cx = 20. * PLANAR_DIRECTION[dir as usize].0;
            let cy = 20. * PLANAR_DIRECTION[dir as usize].1;

            html! {
                <circle data-dir={dir} cx={cx} cy={cy} r=5 />
            }
        }

        let dots = (0..6)
            .into_iter()
            .filter(|dir| edges[*dir as usize].rail_connection)
            .map(|dir| circle(dir))
            .collect::<Html>();

        html! {
            <g transform={ format!("translate({},{})", 500 + x, 500 + y) }>
                <path
                    class="hex"
                    d="m-15,-26 l30,0 l15,26 l-15,26 l-30,0 l-15,-26 z"
                />
                { dots }
            </g>
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut map = Map::new();
        map.cell_mut((0, 0)).edge[1].rail_connection = true;
        map.cell_mut((0, 1)).edge[1].rail_connection = true;

        App {
            clicked: false,
            onclick: link.callback(|_| Msg::Click),
            map,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                self.clicked = true;
                true // Indicate that the Component should re-render
            }
        }
    }

    fn view(&self) -> Html {
        let button_text = if self.clicked {
            "Clicked!"
        } else {
            "Click me!"
        };

        use std::cmp::{max, min};

        let coords = (-3..4).into_iter().flat_map(|q| {
            (max(-3, -3 - q)..min(4, 4 - q))
                .into_iter()
                .map(move |r| (q, r))
        });

        html! {
            <>
                <svg viewBox="0 0 1000 1000" style="width: 1000px; height: 1000px">
                    { coords.into_iter().map(|(q, r)| self.hex(q, r)).collect::<Html>() }
                </svg>
                <button onclick=&self.onclick>{ button_text }</button>
            </>
        }
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<App>();

    Ok(())
}
