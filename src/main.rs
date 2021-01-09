#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![deny(warnings)]

#[macro_use]
extern crate log;
extern crate keybd_event;
extern crate reqwest;

use keybd_event::KeyBondingInstance;
use keybd_event::KeyboardKey::{KeyLEFT, KeyRIGHT};

use dotenv::dotenv;
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Controller, Flex, Label, Either};
use druid::{
    AppDelegate, AppLauncher, Application, Data, DelegateCtx, FontDescriptor, FontFamily, Lens,
    LocalizedString, UnitPoint, WidgetExt, WindowDesc, WindowId,
};
use std::thread;

mod websocket;

#[derive(Clone, Data, Lens, Debug)]
struct AppState {
    websocket_url: Option<String>,
    publish_url: Option<String>,
    status: String,
    has_accessibility_permissions: bool,
}

struct Delegate {}
impl AppDelegate<AppState> for Delegate {
    fn window_removed(
        &mut self,
        _id: WindowId,
        _data: &mut AppState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        Application::global().quit();
    }
}

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Next slide please?");

#[cfg(target_os = "macos")]
fn has_accessibility_permissions() -> bool {
    let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    if !trusted {
        warn!("application isn't trusted");
    }
    return trusted
}

#[cfg(not(target_os = "macos"))]
fn has_accessibility_permissions() -> bool {
    true
}

pub fn main() {
    dotenv().ok();
    env_logger::init();
    info!("starting up");

    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    // create the initial app state
    let initial_state = AppState {
        websocket_url: None,
        publish_url: None,
        status: "Initializing".into(),
        has_accessibility_permissions: has_accessibility_permissions()
    };

    // Spawn the thread that manages the websocket connection to the backend
    let launcher = AppLauncher::with_window(main_window).delegate(Delegate {});
    let event_sink = launcher.get_external_handle();
    thread::spawn(move || websocket::WebSocketConnection::new(event_sink).connect_loop());

    // start the application
    launcher
        .launch(initial_state)
        .expect("Failed to launch application");

    debug!("end of main");
}

struct AppController;

impl<C : Widget<AppState>> Controller<AppState, C> for AppController {
    fn event(
        &mut self,
        child: &mut C,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(websocket::STATE_CHANGED) => {
                match cmd.get_unchecked(websocket::STATE_CHANGED) {
                    websocket::StateChange::Connecting => {
                        data.status = "Connecting...".into();
                        data.websocket_url = None;
                        data.publish_url = None;
                    }

                    websocket::StateChange::Connected {
                        websocket_url,
                        publish_url,
                    } => {
                        data.status = "Connected!".into();
                        data.websocket_url = Some(websocket_url.to_owned());
                        data.publish_url = Some(publish_url.to_owned());
                    }

                    websocket::StateChange::EventReceived(event) => {
                        let mut kb = KeyBondingInstance::new().unwrap();
                        match event {
                            websocket::Event::Previous => kb.add_keys(&[KeyLEFT]),
                            websocket::Event::Next => kb.add_keys(&[KeyRIGHT]),
                        }
                        kb.launching();
                        debug!("event received {:?}", event);
                    }
                }
            }

            // Forward other events to widget children
            _ => child.event(ctx, event, data, env),
        }
    }
}

fn heading() -> Align<AppState> {
    Label::new(|_data: &AppState, _env: &Env| "Next slide please?".to_string())
        .with_font(FontDescriptor::new(FontFamily::SERIF).with_size(32.0))
        .align_horizontal(UnitPoint::CENTER)
}

fn build_root_widget() -> impl Widget<AppState> {
    let status_label =
        Label::<AppState>::new(|data: &AppState, _env: &Env| format!("Status: {}", data.status));

    let open_publish_button =
        Button::<AppState>::new("Open control website").on_click(|_ctx, data, _env| {
            if let Some(url) = &data.publish_url {
                webbrowser::open(&url).expect("Failed to open browser");
            }
        });

    let no_accessibility_options = Label::<AppState>::new("E_NOACCESSIBILITY");

    let yes = Flex::column()
        .with_child(heading())
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(status_label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(open_publish_button)
        .align_vertical(UnitPoint::CENTER);

    let no = Flex::column()
        .with_child(heading())
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(no_accessibility_options)
        .align_vertical(UnitPoint::CENTER);

    Either::<AppState>::new(
        |state, _| state.has_accessibility_permissions,
        yes,
        no
    ).controller(AppController)
}
