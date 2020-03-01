use wasm_bindgen::prelude::*;
use yew::{html, Callback, ClickEvent, Component, ComponentLink, Html, ShouldRender};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct App {
    clicked: bool,
    onclick: Callback<ClickEvent>,
}

enum Msg {
    Click,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            clicked: false,
            onclick: link.callback(|_| Msg::Click),
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

        fn hex(q: i32, r: i32) -> Html {
            let x = q * 45;
            let y = q * 26 + r * 52;

            html! {
                <path
                    class="hex"
                    d={
                        format!("M500,500 m-15,-26 m{},{} l30,0 l15,26 l-15,26 l-30,0 l-15,-26 z", x, y)
                    }
                />
            }
        }

        use std::cmp::{max, min};

        let coords = (-3..4).into_iter().flat_map(|q| {
            (max(-3, -3 - q)..min(4, 4 - q))
                .into_iter()
                .map(move |r| (q, r))
        });

        html! {
            <>
                <svg viewBox="0 0 1000 1000" style="width: 1000px; height: 1000px">
                    { coords.into_iter().map(|(q, r)| hex(q, r)).collect::<Html>() }
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
