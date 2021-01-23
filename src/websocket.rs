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

use anyhow::{Context, Result};
use druid::{Selector, Target};
use reqwest::Url;
use std::collections::HashMap;
use std::fmt::Debug;
use std::thread;
use std::time::Duration;
use tungstenite::client::AutoStream;
use tungstenite::{connect, Message, WebSocket};

#[derive(Clone, Debug)]
pub enum Event {
    Previous,
    Next,
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Connecting,
    Connected {
        websocket_url: String,
        publish_url: String,
    },
    EventReceived(Event),
}

const BACKEND_REGISTER_ENDPOINT: &str = "https://next-slide-please.com/register";
pub(crate) const STATE_CHANGED: Selector<StateChange> = Selector::new("set-websocket-state");

#[derive(Debug, Clone)]
struct Session {
    publish_url: String,
    websocket_url: String,
    secret: String,
}

pub struct WebSocketConnection {
    event_sink: druid::ExtEventSink,
    client: reqwest::blocking::Client,
    session: Option<Session>,
}

impl WebSocketConnection {
    pub fn new(event_sink: druid::ExtEventSink) -> Self {
        WebSocketConnection {
            event_sink,
            client: reqwest::blocking::Client::new(),
            session: None,
        }
    }

    pub fn submit_command(&self, payload: StateChange) {
        self.event_sink
            .submit_command(STATE_CHANGED, payload, Target::Auto)
            .expect("Failed to send message to main thread");
    }

    fn register(&self) -> Result<Session, anyhow::Error> {
        if self.session.is_some() {
            let session = self.session.clone().unwrap();
            debug!(
                "Register called while session already exists: {:?}",
                &session
            );
            return Ok(session);
        }

        let res = self
            .client
            .post(BACKEND_REGISTER_ENDPOINT)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()?;

        let content = res.json::<HashMap<String, String>>()?;
        let publish_url = content
            .get("publish_url")
            .context("Response did not contain 'publish_url'")?
            .to_owned();
        let websocket_url = content
            .get("websocket_url")
            .context("Response did not contain 'websocket_url'")?
            .to_owned();

        let secret = content
            .get("secret")
            .context("Response did not contain 'secret'")?
            .to_owned();

        trace!("Websocket URL: {:?}", websocket_url);
        trace!("Publish URL: {:?}", publish_url);

        Ok(Session {
            publish_url,
            websocket_url,
            secret,
        })
    }

    fn websocket_connect(
        &self,
        websocket_url: &String,
    ) -> Result<WebSocket<AutoStream>, anyhow::Error> {
        let url = Url::parse(websocket_url)?;
        let (socket, response) = connect(url)?;

        trace!("Connected to the server");
        trace!("Response HTTP code: {}", response.status());
        trace!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            trace!("* {}", header);
        }

        Ok(socket)
    }

    fn connect(&mut self) -> Result<(), anyhow::Error> {
        self.submit_command(StateChange::Connecting);

        let session = self
            .register()
            .context("Failed to register new connection")?;

        let mut socket = self
            .websocket_connect(&session.websocket_url)
            .context("Failed to establish websocket connection")?;

        self.submit_command(StateChange::Connected {
            publish_url: session.publish_url.to_owned(),
            websocket_url: session.websocket_url.to_owned(),
        });

        socket
            .write_message(Message::Text(session.secret.clone()))
            .context("Failed to send secret to backend")?;

        loop {
            let message = socket
                .read_message()
                .context("Error reading from websocket")?;

            match message {
                Message::Text(msg) if msg == "Next" => {
                    debug!("'Next' received!");
                    self.submit_command(StateChange::EventReceived(Event::Next));
                }

                Message::Text(msg) if msg == "Previous" => {
                    debug!("'Previous' received!");
                    self.submit_command(StateChange::EventReceived(Event::Previous));
                }

                Message::Ping(_) => {
                    trace!("Received ping");
                }

                msg => {
                    debug!("Received unexpected message: {}", msg)
                }
            }
        }
    }

    pub fn connect_loop(&mut self) {
        loop {
            info!("Trying to establish websocket connection");
            match self.connect() {
                Err(e) => error!("Reconnecting due to Websocket connection error: {}", e),
                Ok(_) => { /* Do nothing */ }
            }
            thread::sleep(Duration::from_secs(1));
        }
    }
}
