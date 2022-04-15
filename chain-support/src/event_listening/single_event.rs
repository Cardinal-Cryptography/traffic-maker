use std::time::Duration;

use ac_node_api::events::{EventsDecoder, Raw};
use aleph_client::Connection;
use anyhow::Result;
use hex::FromHex;
use log::error;
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

pub struct SingleEventListener<E: Event> {
    receive_event: Receiver<E>,
    cancel_listening: Sender<()>,
    listening_handle: JoinHandle<()>,
}

fn subscribe_for_events(connection: &Connection) -> Result<EventsOut> {
    let (events_in, events_out) = std::sync::mpsc::channel();
    connection
        .subscribe_events(events_in)
        .map_err(|_| ListeningError::CannotSubscribe)?;
    Ok(events_out)
}

impl<E: Event> SingleEventListener<E> {
    fn handle_new_events_batch(
        expected_event: &E,
        encoded_batch: String,
        events_decoder: &EventsDecoder,
    ) -> Result<E> {
        let encoded_batch = encoded_batch.replace("0x", "");
        let raw_events =
            events_decoder.decode_events(&mut Vec::from_hex(encoded_batch)?.as_slice())?;

        for (_, event) in raw_events.into_iter() {
            error!("Another event {:?}", event);
            if let Raw::Event(raw_event) = event {
                error!("Info {:?} {:?}", raw_event.pallet, raw_event.variant);
                if (&*raw_event.pallet, &*raw_event.variant) != expected_event.kind() {
                    continue;
                }
                if let Ok(received_event) = E::decode(&mut &raw_event.data[..]) {
                    if expected_event.matches(&received_event) {
                        return Ok(received_event);
                    }
                }
            }
        }
        Err(ListeningError::NoEventSpotted.into())
    }

    async fn listen_for_event(
        event: E,
        event_received: Sender<E>,
        mut cancel: Receiver<()>,
        events_out: EventsOut,
        events_decoder: EventsDecoder,
    ) {
        loop {
            error!("Helo");

            for event_str in events_out.try_iter() {
                error!("Another event {:?}", event_str);

                if let Ok(encountered_event) =
                    Self::handle_new_events_batch(&event, event_str, &events_decoder)
                {
                    let _ = event_received.send(encountered_event);
                    return;
                }
            }

            if cancel.try_recv().is_ok() {
                return;
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    pub async fn new(connection: &Connection, event: E) -> Result<Self> {
        let (event_tx, event_rx) = channel::<E>();
        let (cancel_tx, cancel_rx) = channel();

        let events_out = do_async!(subscribe_for_events, &connection)??;
        let decoder = EventsDecoder::new(connection.metadata.clone());
        let listening_handle = tokio::spawn(Self::listen_for_event(
            event, event_tx, cancel_rx, events_out, decoder,
        ));

        Ok(Self {
            receive_event: event_rx,
            cancel_listening: cancel_tx,
            listening_handle,
        })
    }

    async fn cancel_listening(cancel: Sender<()>, handle: JoinHandle<()>) -> Result<()> {
        // Even if sending cancellation fails, then it should be all fine anyway.
        let _ = cancel.send(());
        handle.await.map_err(|e| e.into())
    }

    pub async fn kill(self) -> Result<()> {
        Self::cancel_listening(self.cancel_listening, self.listening_handle).await
    }

    pub async fn expect_event(self, duration: Duration) -> Result<E> {
        match timeout(duration, self.receive_event).await {
            Ok(Ok(event)) => Ok(event),
            _ => {
                Self::cancel_listening(self.cancel_listening, self.listening_handle).await?;
                Err(ListeningError::NoEventSpotted.into())
            }
        }
    }
}
