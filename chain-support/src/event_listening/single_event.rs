use std::{fmt::Debug, future::Future, time::Duration};

use ac_node_api::events::{EventsDecoder, Raw};
use aleph_client::Connection;
use anyhow::Result as AnyResult;
use hex::FromHex;
use tokio::{
    sync::oneshot::{channel, Receiver, Sender},
    task::JoinHandle,
    time::{sleep, timeout},
};

use crate::{
    do_async,
    event_listening::{Event, ListeningError},
};

type EventsOut = std::sync::mpsc::Receiver<String>;

/// `SingleEventListener` lets you set up listening for a single event. It is completely
/// non-blocking and asynchronous.
pub struct SingleEventListener<E: Event> {
    /// The listening itself is performed in another thread. When the expected event is observed, it
    /// will be sent through this one-shot channel.
    receive_event: Receiver<E>,
    /// The listening process can be stopped by sending signal through this one-shot channel.
    cancel_listening: Sender<()>,
    /// The handle for the thread with listening process.
    listening_handle: JoinHandle<()>,
}

/// Makes an event subscription at the node (through `connection`) and returns a channel through
/// which all emitted (encoded) events are sent.
///
/// The channel itself is blocking, i.e. not awaitable (API requirement).
///
/// The only blocking call in this module. It is implemented as an outside function to enable
/// calling it with `do_async` macro.
fn subscribe_for_events(connection: &Connection) -> AnyResult<EventsOut> {
    let (events_in, events_out) = std::sync::mpsc::channel();
    connection
        .subscribe_events(events_in)
        .map_err(|_| ListeningError::CannotSubscribe)?;
    Ok(events_out)
}

impl<E: Event> SingleEventListener<E> {
    /// Emitted events are encoded and sent in batches. `handle_new_events_batch` inspects every
    /// single event in `encoded_batch` and checks it against `matcher`.
    ///
    /// Returns `Err(_)` if there was problem with reading `encoded_batch` or
    /// `ListeningError::NoEventSpotted` if the batch was properly decoded, but none of the events
    /// was satisfying.
    fn handle_new_events_batch<M: Fn(&E) -> bool>(
        matcher: &M,
        encoded_batch: String,
        events_decoder: &EventsDecoder,
    ) -> AnyResult<E> {
        let encoded_batch = encoded_batch.replace("0x", "");
        let raw_events =
            events_decoder.decode_events(&mut Vec::from_hex(encoded_batch)?.as_slice())?;

        for (_, event) in raw_events.into_iter() {
            if let Raw::Event(raw_event) = event {
                if (&*raw_event.pallet, &*raw_event.variant) == E::kind() {
                    if let Ok(received_event) = E::decode(&mut &raw_event.data[..]) {
                        if matcher(&received_event) {
                            return Ok(received_event);
                        }
                    }
                }
            }
        }
        Err(ListeningError::NoEventSpotted.into())
    }

    /// Asynchronously receives new events through `events_out` and checks them against `matcher`.
    /// Occasionally checks whether cancellation signal has been sent through `cancel`. When there
    /// is nothing to do yields the control for some time.
    ///
    /// A spotted event that satisfies `matcher` will be sent through `event_received` and the whole
    /// process will end.
    ///
    /// Since events from `events_out` are encoded to `String`, we expect here also `events_decoder`
    /// which is able to decode them.
    async fn listen_for_event<M: Fn(&E) -> bool>(
        matcher: M,
        event_received: Sender<E>,
        mut cancel: Receiver<()>,
        events_out: EventsOut,
        events_decoder: EventsDecoder,
    ) {
        // Listen in a loop, until either satisfying event comes or the cancel signal has been sent.
        loop {
            // Check in a non-blocking manner whether there are some events ready for processing.
            for event_str in events_out.try_iter() {
                if let Ok(encountered_event) =
                    Self::handle_new_events_batch(&matcher, event_str, &events_decoder)
                {
                    let _ = event_received.send(encountered_event);
                    return;
                }
            }

            // Check in a non-blocking manner whether we should give up.
            if cancel.try_recv().is_ok() {
                return;
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Constructs new `SingleEventListener` and starts listening for an event matching `matcher`
    /// in another thread.
    ///
    /// Can fail (returns `ListeningError::CannotSubscribe`) only if subscribing to a node was
    /// unsuccessful.
    pub async fn new<M: Fn(&E) -> bool + Send + 'static>(
        connection: &Connection,
        matcher: M,
    ) -> AnyResult<Self> {
        let (event_tx, event_rx) = channel::<E>();
        let (cancel_tx, cancel_rx) = channel();

        let events_out = do_async!(subscribe_for_events, &connection)??;
        let decoder = EventsDecoder::new(connection.metadata.clone());
        let listening_handle = tokio::spawn(Self::listen_for_event(
            matcher, event_tx, cancel_rx, events_out, decoder,
        ));

        Ok(Self {
            receive_event: event_rx,
            cancel_listening: cancel_tx,
            listening_handle,
        })
    }

    /// Cancels listening by sending signal through `cancel`.
    ///
    /// Returns `Err(_)` only if there have been problems with joining the auxiliary thread
    /// (through `handle`).
    async fn cancel_listening(cancel: Sender<()>, handle: JoinHandle<()>) -> AnyResult<()> {
        // Even if sending cancellation fails, then it should be all fine: the process must have
        // finished anyway, since the channel is closed.
        let _ = cancel.send(());
        handle.await.map_err(|e| e.into())
    }

    /// Semantically identical to `Self::cancel_listening`.
    pub async fn kill(self) -> AnyResult<()> {
        Self::cancel_listening(self.cancel_listening, self.listening_handle).await
    }

    /// For at most `duration` wait (no blocking) for the event to be observed.
    ///
    /// Returns `Ok(event)` if an event matching `matcher` has been emitted and observed.
    /// Otherwise, returns `ListeningError::NoEventSpotted`. In any case, the auxiliary thread will
    /// be shut down.
    pub async fn expect_event(self, duration: Duration) -> AnyResult<E> {
        match timeout(duration, self.receive_event).await {
            Ok(Ok(event)) => Ok(event),
            _ => {
                Self::cancel_listening(self.cancel_listening, self.listening_handle).await?;
                Err(ListeningError::NoEventSpotted.into())
            }
        }
    }
}

/// Like [with_event_matching] but looks for an event matching a specific struct instead of using a
/// closure to perform the match.
pub async fn with_event_listening<E: Event, R: Debug, F: Future<Output = AnyResult<R>>>(
    connection: &Connection,
    expected_event: E,
    event_timeout: Duration,
    action: F,
) -> AnyResult<(R, E)> {
    with_event_matching(
        connection,
        move |e| expected_event.matches(e),
        event_timeout,
        action,
    )
    .await
}

/// Wrapper for a waiting-flow depending on the success of `action`.
///
/// Performs three steps:
/// - creates a `SingleEventListener` instance for `event_matcher` (using `connection`)
/// - awaits for `action`
/// - depending on whether `action` returned:
///     - `Ok(result)`: waits for an event matching `event_matcher` for at most `event_timeout` and
///        returns either `(result, received_event)` if listening succeeded or `Err(_)` otherwise.
///     - `Err(e)`: cancels listening and returns `Err(e)`.
pub async fn with_event_matching<
    E: Event,
    R: Debug,
    F: Future<Output = AnyResult<R>>,
    M: Fn(&E) -> bool + Send + 'static,
>(
    connection: &Connection,
    event_matcher: M,
    event_timeout: Duration,
    action: F,
) -> AnyResult<(R, E)> {
    let sel = SingleEventListener::new(connection, event_matcher).await?;
    match action.await {
        Ok(result) => match sel.expect_event(event_timeout).await {
            Ok(event) => Ok((result, event)),
            Err(e) => Err(e),
        },
        Err(e) => {
            let _ = sel.kill().await;
            Err(e)
        }
    }
}
