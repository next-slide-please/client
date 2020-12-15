use std::time::Instant;
use std::thread;
use std::time::Duration;

extern crate reqwest;

use druid::widget::prelude::*;
use druid::{AppLauncher, Color, Selector, Target, Data, WidgetExt, Lens, WindowDesc, FontDescriptor, UnitPoint, LocalizedString, FontFamily};
use druid::widget::{Label, TextBox, Flex, Controller, Align};
use tungstenite::{connect, Message};
use url::Url;
use std::collections::HashMap;

#[macro_use]
extern crate log;

#[derive(Clone, Data, Lens)]
struct AppState {
    name: String,
    backend_url: String,
}

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Next slide please!");

// If you want to submit commands to an event sink you have to give it some kind
// of ID. The selector is that, it also assures the accompanying data-type is correct.
// look at the docs for `Selector` for more detail.
const SET_FOO: Selector<u32> = Selector::new("event-example.set-color");

pub fn main() {
    env_logger::init();
    info!("starting up");

    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    // create the initial app state
    let initial_state = AppState {
        name: "World".into(),
        backend_url: "http://127.0.0.1:8000".into(),
    };

    // start the application
    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();
    thread::spawn(move || websocket_thread(event_sink));

    launcher
       // .use_simple_logger()
        .launch(initial_state)
        .expect("Failed to launch application");
}


struct AppController;

impl Controller<AppState, Align<AppState>> for AppController {
    fn event(
        &mut self,
        child: &mut Align<AppState>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(SET_FOO)=> {
                let foo = cmd.get_unchecked(SET_FOO);
                data.name = format!("foo {}", foo);

                //child.event(ctx, event, data, env)
            }

            _ => child.event(ctx, event, data, env),
        }
    }
}

fn build_root_widget() -> impl Widget<AppState> {
    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &AppState, _env: &Env| {
        if data.name.is_empty() {
            "Hello anybody!?".to_string()
        } else {
            format!("Hello {}!", data.name)
        }
    })
        .with_font(FontDescriptor::new(FontFamily::SERIF).with_size(32.0))
        .align_horizontal(UnitPoint::CENTER);

    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .with_text_size(18.0)
        .fix_width(TEXT_BOX_WIDTH)
        .align_horizontal(UnitPoint::CENTER)
        .lens(AppState::name);

    // arrange the two widgets vertically, with some padding
    Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox)
        .align_vertical(UnitPoint::CENTER)
        .controller(AppController)
}

fn websocket_thread(event_sink: druid::ExtEventSink) {
    let client = reqwest::blocking::Client::new();
    let mut res = client.post("http://127.0.0.1:8000/register")
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .unwrap();
    let content = res.json::<HashMap<String, String>>().unwrap();
    let url = content.get("url").unwrap();
    println!("{:?}", url);

    let (mut socket, response) =
        connect(Url::parse(url).unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    //socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }

    loop {
        // if event_sink
        //         .submit_command(SET_FOO, 5, Target::Auto)
        //         .is_err()
        //     {
        //         break;
        //     }
        thread::sleep(Duration::from_millis(1000));
    }
}