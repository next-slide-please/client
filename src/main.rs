// NextSlidePlease.com GUI client
// Copyright (C) 2021  Lucas Jenß
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![deny(warnings)]

#[macro_use]
extern crate log;
extern crate keybd_event;
extern crate reqwest;

use keybd_event::KeyBondingInstance;
use keybd_event::KeyboardKey::{KeyLEFT, KeyRIGHT};

use dotenv::dotenv;
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Controller, Either, Flex, Label, LineBreaking};
use druid::{
    AppDelegate, AppLauncher, Application, Color, Data, DelegateCtx, FontDescriptor, FontFamily,
    Lens, LocalizedString, UnitPoint, WidgetExt, WindowDesc, WindowId,
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
    return trusted;
}

#[cfg(not(target_os = "macos"))]
fn has_accessibility_permissions() -> bool {
    true
}

fn main() {
    dotenv().ok();
    env_logger::init();
    info!("starting up");

    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 200.0));

    #[cfg(target_os = "macos")]
    macos_app_nap::prevent();

    let initial_state = AppState {
        websocket_url: None,
        publish_url: None,
        status: "Initializing".into(),
        has_accessibility_permissions: has_accessibility_permissions(),
    };

    let launcher = AppLauncher::with_window(main_window).delegate(Delegate {});
    let event_sink = launcher.get_external_handle();
    thread::spawn(move || websocket::WebSocketConnection::new(event_sink).connect_loop());

    launcher
        .launch(initial_state)
        .expect("Failed to launch application");

    debug!("end of main");
}

struct AppController;

impl<C: Widget<AppState>> Controller<AppState, C> for AppController {
    fn event(
        &mut self,
        child: &mut C,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::WindowConnected => {
                debug!("Widget WindowConnected");
            }

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

    let version_label = Label::<AppState>::new(|_data: &AppState, _env: &Env| {
        format!(
            "v{}+{}",
            env!("CARGO_PKG_VERSION"),
            env!("VERGEN_SHA_SHORT")
        )
    })
    .with_text_color(Color::WHITE.with_alpha(0.5));

    let open_publish_button =
        Button::<AppState>::new("Open control website").on_click(|_ctx, data, _env| {
            if let Some(url) = &data.publish_url {
                webbrowser::open(&url).expect("Failed to open browser");
            }
        });

    let no_accessibility_options =
        Label::<AppState>::new("Please grant the application the ability to control they keyboard, so that it can advance the slides when your co-presenters press the Previous/Next button.\n\nRestart the application afterwards.")
            .with_line_break_mode(LineBreaking::WordWrap);

    let yes = Flex::column()
        .with_child(heading())
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(status_label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(open_publish_button)
        .align_vertical(UnitPoint::CENTER);

    let version_row = Flex::row().with_flex_spacer(1.0).with_child(version_label);

    let no = Flex::column()
        .with_child(heading())
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(no_accessibility_options)
        .align_vertical(UnitPoint::CENTER)
        .padding(10.0);

    let yes_no = Either::<AppState>::new(|state, _| state.has_accessibility_permissions, yes, no)
        .controller(AppController);

    Flex::column()
        .with_flex_child(yes_no, 1.0)
        .with_child(version_row)







}
