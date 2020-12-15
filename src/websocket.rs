use iced_futures::futures;
use iced_native::Event;

// Just a little utility function
pub fn file<T: ToString>(url: T) -> iced::Subscription<Progress> {
    iced::Subscription::from_recipe(Download {
        url: url.to_string(),
    })
}

pub struct Download {
    url: String,
}

// Make sure iced can use our download stream
impl<H, I> iced_futures::subscription::Recipe<H, I> for Download
    where
        H: std::hash::Hasher,
        I: Event
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.url.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(
            futures::stream::unfold(State::Ready(self.url), |state| async move {
                match state {
                    State::Ready(url) => {
                        Some((Progress::Started, State::Counter(0.0)))
                    }

                    State::Counter(x) => {
                        use std::{thread, time};

                        let ten_millis = time::Duration::from_millis(1000);
                        let now = time::Instant::now();

                        thread::sleep(ten_millis);

                        Some((Progress::Advanced(x / 10.0), State::Counter(x + 1.0)))
                    }

                    _ => {
                        // We do not let the stream die, as it would start a
                        // new download repeatedly if the user is not careful
                        // in case of errors.
                        let _: () = iced::futures::future::pending().await;

                        None
                    }
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}

pub enum State {
    Ready(String),
    Counter(f32),
    Finished,
}