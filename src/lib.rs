#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
use std::collections::HashMap;
#[allow(unused_imports)]
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);

    Model {
        current_topic: LocalStorage::get(TOPIC_KEY).unwrap_or_else(|_| String::from("Ödev")),
        solves: LocalStorage::get(&date_str()).unwrap_or_default(),
        route: Routes::from(url),
    }
}

struct Model {
    current_topic: String,
    solves: HashMap<String, i32>,
    route: Routes,
}

#[derive(Clone)]
enum Msg {
    UrlChanged(subs::UrlChanged),
    NewTopic(String),
    IncrementCount(i32),
}

#[derive(Copy, Clone, PartialEq, EnumIter)]
enum Routes {
    Home,
    History(f64),
    NotFound,
}

impl From<Url> for Routes {
    fn from(mut url: Url) -> Self {
        log!(url);
        match url.remaining_path_parts().as_slice() {
            [] => Self::Home,
            ["history", x] if matches!(x.parse::<i32>(), Ok(_)) => {
                Self::History(x.parse().unwrap())
            }
            ["history"] => Self::History(0.0),
            _ => Self::NotFound,
        }
    }
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.route = Routes::from(url);
        }
        Msg::NewTopic(t) => {
            model.current_topic = t;
            LocalStorage::insert(TOPIC_KEY, &model.current_topic).unwrap();
        }
        Msg::IncrementCount(i) => {
            let base = model.solves.entry(model.current_topic.clone()).or_insert(0);
            *base = (*base + i).max(0);
            if *base == 0 {
                model.solves.remove(&model.current_topic);
            }
            LocalStorage::insert(&date_str(), &model.solves).unwrap();
        }
    }
}

fn main_view(model: &Model) -> Node<Msg> {
    div![
        div![
            C!["wrapper"],
            div![C!["one-by-one", "aspect-ratio"]],
            div![
                C!["content"],
                div![
                    C!["circle"],
                    ev(Ev::Click, |_| Msg::IncrementCount(1)),
                    div![
                        C!["button-inner"],
                        "Aktif konu: ",
                        &model.current_topic,
                        br!(),
                        model.solves.get(&model.current_topic).unwrap_or(&0),
                        " soru yapıldı."
                    ]
                ],
            ],
        ],
        div![
            C!["topic-wrapper"],
            input![
                C!["topic-input"],
                attrs! {
                    At::Type => "text",
                    At::Value => model.current_topic,
                },
                input_ev(Ev::Input, Msg::NewTopic)
            ],
            button![
                C!["button"],
                ev(Ev::Click, |_| Msg::IncrementCount(-1)),
                style! { St::MarginLeft => rem(0.5) },
                "Eksilt"
            ],
            a![
                C!["button"],
                attrs! { At::Href => "/history" },
                style! { St::MarginLeft => rem(0.5) },
                span!["Bitenler"]
            ],
        ]
    ]
}

fn history_view(_: &Model, v: f64) -> Node<Msg> {
    let start = date().set_date(date().get_date() - date().get_day() - 1 - 0 * 7); // - 8.0 * 60.0 * 60.0 * 1000.0;
    let (sum, nodes) = (0..7)
            .map(|i| (start + i as f64 * 1000.0 * 60.0 * 60.0 * 24.0 - (v - 1.0) * 1000.0 * 60.0 * 60.0 * 24.0 * 7.0, i))
            .inspect(|x| log!(x))
            .map(|(e, i)| (to_date_str(e), i))
            .map(|(s, i)| (LocalStorage::get(&s).unwrap_or_default(), i))
            .map(|(x, i): (HashMap<String, i32>, _)| {
                let (sum, nodes) = x.iter().fold((0, vec![]), |(sum, mut v), (topic, count)| {
                    v.push(span![topic, ": ", count, br!()]);
                    (sum + count, v)
                });
                (sum, div![
                    C!["card"],
                    h3![DAY_NAMES[i]],
                    nodes,
                    span![style!{ St::MarginTop => rem(1), St::Width => percent(100), St::Display => "block" }],
                    span!["Toplam: ", sum.to_string()],
                ])
            })
            .fold((0, vec![]), |(s, mut v), (ns, cns)| { v.push(cns); (s + ns, v) });
    div![
        C!["container"],
        h2![C!["total-sum"], "NET TOPLAM: ", sum.to_string()],
        nodes,
        div![
            a![
                C!["button"],
                attrs! { At::Href => "/" },
                style! { St::MarginBottom => rem(0.2) },
                "Geri dön"
            ],
            a![
                C!["button"],
                attrs! { At::Href => "/history" },
                style! { St::MarginBottom => rem(0.2) },
                "Bu hafta"
            ],
            a![
                C!["button"],
                attrs! { At::Href => format!("/history/{}", v + 1.0) },
                "Geçen hafta"
            ],
        ],
    ]
}

fn view(model: &Model) -> Node<Msg> {
    match model.route {
        Routes::Home => main_view(model),
        Routes::History(v) => history_view(model, v),
        Routes::NotFound => h1!["NOT FOUND"],
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

fn date_str() -> String {
    to_date_str(date().get_time())
}

fn to_date_str(n: f64) -> String {
    let mut s = n.to_string();
    s.truncate(10);
    s
}

fn date() -> js_sys::Date {
    let x = js_sys::Date::new_0().get_time() - (8 * 60 * 60 * 1000) as f64;
    let x = js_sys::Date::new(&JsValue::from_f64(x));
    js_sys::Date::new_with_year_month_day(
        x.get_full_year(),
        x.get_month() as i32,
        x.get_date() as i32,
    )
}

const DAY_NAMES: [&str; 7] = [
    "Cumartesi",
    " Pazar",
    "Pazartesi",
    "Salı",
    "Çarşamba",
    "Perşembe",
    "Cuma",
];

const TOPIC_KEY: &str = "topic";
