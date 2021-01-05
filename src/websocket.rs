use druid::{Selector, Target};
use std::collections::HashMap;
use tungstenite::{connect, WebSocket, Message};
use reqwest::Url;
use std::thread;
use std::time::Duration;
use std::error::Error;
use tungstenite::client::AutoStream;

#[derive(Clone, Debug)]
pub enum Event {
    Previous,
    Next,
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Connecting,
    Connected { websocket_url: String, publish_url: String },
    EventReceived(Event),
}

const BACKEND_REGISTER_ENDPOINT: &str = "http://localhost:8000/register";
pub(crate) const STATE_CHANGED: Selector<StateChange> = Selector::new("set-websocket-state");


pub struct WebSocketConnection {
    event_sink: druid::ExtEventSink,
    client: reqwest::blocking::Client,
}

impl WebSocketConnection {
    pub fn new(event_sink: druid::ExtEventSink) -> Self {
        WebSocketConnection {
            event_sink,
            client: reqwest::blocking::Client::new()
        }
    }

    pub fn submit_command(&self, payload: StateChange) {
        self.event_sink.submit_command(
            STATE_CHANGED,
            payload,
            Target::Auto
        ).expect("Failed to send message to main thread");
    }

    fn register(&self) -> Result<(String, String), Box<dyn Error>> {
        let res = self.client.post(BACKEND_REGISTER_ENDPOINT)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()?;

        let content = res.json::<HashMap<String, String>>()?;
        let publish_url = content.get("publish_url")
            .ok_or("Response did not contain 'publish_url'")?
            .to_owned();
        let websocket_url = content.get("websocket_url")
            .ok_or("Response did not contain 'websocket_url'")?
            .to_owned();

        debug!("Websocket URL: {:?}", websocket_url);
        debug!("Publish URL: {:?}", publish_url);

        Ok((publish_url, websocket_url))
    }

    fn websocket_connect(&self, websocket_url: &String) -> Result<WebSocket<AutoStream>, Box<dyn Error>> {
        let url = Url::parse(websocket_url)?;
        let (socket, response) = connect(url)?;

        debug!("Connected to the server");
        debug!("Response HTTP code: {}", response.status());
        debug!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            debug!("* {}", header);
        }

        Ok(socket)
    }

    fn connect(&mut self) {
        self.submit_command(StateChange::Connecting);

        let (publish_url, websocket_url) = match self.register() {
            Err(e) => {
                error!("Failed to register new connection: {}", e);
                return;
            }
            Ok((publish_url, websocket_url)) => (publish_url, websocket_url)
        };

        let mut socket = match self.websocket_connect(&websocket_url) {
            Err(e) => {
                error!("Failed to establish websocket connection: {}", e);
                return;
            }
            Ok(socket) => socket
        };

        self.submit_command(StateChange::Connected {
            publish_url: publish_url.to_owned(),
            websocket_url: websocket_url.to_owned()
        });



        //socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
        loop {
            match socket.read_message() {
                Err(e) => {
                    debug!("Error reading from websocket {:?}", e);
                    break;
                }
                Ok(Message::Text(msg)) if msg == "next" => {
                    debug!("'Next' received!");
                    self.submit_command(StateChange::EventReceived(Event::Next));
                }

                Ok(Message::Text(msg)) if msg == "prev" => {
                    debug!("'Previous' received!");
                    self.submit_command(StateChange::EventReceived(Event::Previous));

                }

                Ok(msg) => {
                    debug!("Received unexpected message: {}", msg)
                }
            }
        }
    }


    pub fn connect_loop(&mut self) {
        loop {
            info!("Trying to establish websocket connection");
            self.connect();
            thread::sleep(Duration::from_secs(1));
        }
    }

    // fn websocket_register(event_sink: druid::ExtEventSink) -> Result<(), Box<dyn std::error::Error>> {
    //     Ok(())
    // }
}