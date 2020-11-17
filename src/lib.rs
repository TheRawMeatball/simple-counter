#![allow(clippy::wildcard_imports)]

use seed::{app::CmdHandle, prelude::*, *};
use std::{collections::HashMap, convert::identity};
#[allow(unused_imports)]
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);

    Model {
        current_topic: LocalStorage::get(TOPIC_KEY).unwrap_or_else(|_| String::from("Ödev")),
        solves: LocalStorage::get(&date_str()).unwrap_or_default(),
        route: Routes::from(url),
        hold_timer: None,
    }
}

struct Model {
    current_topic: String,
    solves: HashMap<String, Vec<i32>>,
    route: Routes,
    hold_timer: Option<CmdHandle>,
}

#[derive(Clone)]
enum Msg {
    UrlChanged(subs::UrlChanged),
    NewTopic(String),
    IncrementCount(i32),
    NewTest,
    HoldStart,
    HoldEnd,
    HoldCancel,
}

#[derive(Copy, Clone, PartialEq, EnumIter)]
enum Routes {
    Home,
    History(f64),
    NotFound,
}

impl From<Url> for Routes {
    fn from(mut url: Url) -> Self {
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

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.route = Routes::from(url);
        }
        Msg::NewTopic(t) => {
            model.current_topic = t;
            LocalStorage::insert(TOPIC_KEY, &model.current_topic).unwrap();
        }
        Msg::IncrementCount(n) => {
            let v = model
                .solves
                .entry(model.current_topic.clone())
                .or_insert(vec![0]);

            let base = if let Some(r) = v.last_mut() {
                r
            } else {
                v.push(0);
                v.last_mut().unwrap()
            };

            if n > 0 {
                *base += n;
            } else {
                let mut decrement = -n;
                for _ in 0..v.len() {
                    let solves = v.last_mut().unwrap();
                    let exchange = decrement.min(*solves);
                    decrement -= exchange;
                    if decrement == 0 {
                        *solves -= exchange;
                        if *solves == 0 {
                            v.pop();
                        }
                        break;
                    } else {
                        v.pop();
                    }
                }

                if v.len() == 0 {
                    model.solves.remove(&model.current_topic);
                }
            }

            LocalStorage::insert(&date_str(), &model.solves).unwrap();
        }
        Msg::NewTest => {
            if let Some(x) = model.solves.get_mut(&model.current_topic) {
                if let Some(k) = x.last() {
                    if *k > 0 {
                        x.push(0);
                        LocalStorage::insert(&date_str(), &model.solves).unwrap();
                    }
                }
            }
        }
        Msg::HoldStart => {
            log!("start");
            model.hold_timer =
                Some(orders.perform_cmd_with_handle(cmds::timeout(1000, || Msg::HoldEnd)));
        }
        Msg::HoldEnd => {
            log!("end");
            model.hold_timer = None;
            web_sys::window()
                .and_then(|w| w.prompt_with_message_and_default("", "0").ok())
                .and_then(identity)
                .and_then(|s| {
                    str::parse(&s)
                        .map_err(|_| {
                            web_sys::window().map(|w| w.alert_with_message("Bir sayı yazın."));
                        })
                        .ok()
                })
                .map(Msg::IncrementCount)
                .map(|m| orders.send_msg(m));
        }
        Msg::HoldCancel => {
            if let Some(_) = model.hold_timer {
                log!("cancel");
                model.hold_timer = None;
                orders.send_msg(Msg::IncrementCount(1));
            }
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
                    ev(Ev::PointerUp, |_| Msg::HoldCancel),
                    ev(Ev::PointerDown, |_| Msg::HoldStart),
                    ev(Ev::ContextMenu, |e| {
                        e.prevent_default();
                        e.stop_propagation();
                    }),
                    div![
                        C!["button-inner"],
                        "Aktif konu: ",
                        &model.current_topic,
                        br!(),
                        "Testte ",
                        model
                            .solves
                            .get(&model.current_topic)
                            .and_then(|v| v.last())
                            .unwrap_or(&0),
                        " soru yapıldı",
                        br!(),
                        model
                            .solves
                            .get(&model.current_topic)
                            .map(|v| v.len().saturating_sub(1))
                            .unwrap_or(0),
                        " test yapıldı."
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
            button![
                C!["button"],
                ev(Ev::Click, |_| Msg::NewTest),
                style! { St::MarginLeft => rem(0.5) },
                "Test Bitir"
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
    let start = date().set_date(date().get_date() - date().get_day() - 1 - 0 * 7);
    let (sum, nodes) = (0..7)
            .map(|i| (start + i as f64 * 1000.0 * 60.0 * 60.0 * 24.0 - (v) * 1000.0 * 60.0 * 60.0 * 24.0 * 7.0, i))
            .map(|(e, i)| (to_date_str(e), i))
            .map(|(s, i)| (LocalStorage::get(&s).unwrap_or_default(), i))
            .map(|(x, i): (HashMap<String, Vec<i32>>, _)| {
                let (sum, nodes) = x.iter().fold((0, vec![]), |(sum, mut nodes), (topic, v)| {
                    let (count, n) = v.iter().enumerate().fold((0, vec![]), |(sum, mut nodes), (i, count)| {
                        if i == 0 {
                            nodes.push(format!("{}", count));
                        } else if *count > 0 {
                            nodes.push(format!(" + {}", count));
                        }
                        (sum + count, nodes)
                    });
                    nodes.push(span![
                        topic, ": ", &n, 
                        IF!(n.len() > 1 =>
                            format!(" = {}", count)
                        ),
                        br!()
                    ]);
                    (sum + count, nodes)
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
