//! In-process implementation of the [`EventBus`] port: a `TypeId`-keyed
//! registry of typed handlers.
//!
//! Registration is typed (`register::<InvoiceFinalized, _>(handler)`) so the
//! compiler checks the handler matches the event. Dispatch erases back down
//! to `&dyn DomainEvent`, looks the event's concrete `TypeId` up in the
//! registry, and fans out to every handler under that key. Sync and
//! single-threaded — at single-user / single-org scale there is nothing to
//! gain from a queue or background worker.

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::application::ports::{EventBus, EventHandler};
use crate::domain::events::DomainEvent;

/// Erases `EventHandler<E>` so handlers for different event types can share
/// one `Vec`. `handle` re-checks the concrete type via downcast — the miss
/// branch is unreachable in practice because the bus only ever calls a
/// handler that was registered under that exact `TypeId`.
trait ErasedHandler: Send + Sync {
    fn handle(&self, event: &dyn DomainEvent);
}

struct Erased<E: DomainEvent, H: EventHandler<E>> {
    inner: H,
    _event: PhantomData<fn(E)>,
}

impl<E: DomainEvent, H: EventHandler<E>> ErasedHandler for Erased<E, H> {
    fn handle(&self, event: &dyn DomainEvent) {
        if let Some(typed) = event.downcast_ref::<E>() {
            self.inner.handle(typed);
        }
    }
}

#[derive(Default)]
pub struct InProcessEventBus {
    handlers: HashMap<TypeId, Vec<Arc<dyn ErasedHandler>>>,
}

impl InProcessEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler for one concrete event type. Call once per
    /// (handler, event) pair; multiple handlers for the same event are
    /// fanned out in registration order.
    pub fn register<E, H>(&mut self, handler: H)
    where
        E: DomainEvent,
        H: EventHandler<E> + 'static,
    {
        self.handlers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Arc::new(Erased {
                inner: handler,
                _event: PhantomData,
            }));
    }
}

impl EventBus for InProcessEventBus {
    fn dispatch(&self, event: &dyn DomainEvent) {
        // `Downcast::as_any` resolves to the concrete event's `TypeId`, not
        // the trait object's — that is what makes the registry lookup hit.
        if let Some(handlers) = self.handlers.get(&event.as_any().type_id()) {
            for handler in handlers {
                handler.handle(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use parking_lot::Mutex;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[derive(Debug)]
    struct Opened {
        label: &'static str,
    }
    impl DomainEvent for Opened {
        fn occurred_at(&self) -> DateTime<Utc> {
            at()
        }
        fn event_name(&self) -> &'static str {
            "test.opened"
        }
    }

    #[derive(Debug)]
    struct Closed;
    impl DomainEvent for Closed {
        fn occurred_at(&self) -> DateTime<Utc> {
            at()
        }
        fn event_name(&self) -> &'static str {
            "test.closed"
        }
    }

    /// Pushes the label of every `Opened` it receives into a shared sink.
    struct OpenedRecorder {
        tag: &'static str,
        sink: Arc<Mutex<Vec<String>>>,
    }
    impl EventHandler<Opened> for OpenedRecorder {
        fn handle(&self, event: &Opened) {
            self.sink.lock().push(format!("{}:{}", self.tag, event.label));
        }
    }

    /// Records that it ran at all — used to prove `Closed` events never
    /// reach an `Opened` handler.
    struct ClosedRecorder {
        sink: Arc<Mutex<Vec<String>>>,
    }
    impl EventHandler<Closed> for ClosedRecorder {
        fn handle(&self, _event: &Closed) {
            self.sink.lock().push("closed".into());
        }
    }

    #[test]
    fn register_then_dispatch_invokes_typed_handler() {
        let sink = Arc::new(Mutex::new(Vec::new()));
        let mut bus = InProcessEventBus::new();
        bus.register::<Opened, _>(OpenedRecorder {
            tag: "h",
            sink: sink.clone(),
        });

        bus.dispatch(&Opened { label: "front-door" });

        assert_eq!(*sink.lock(), ["h:front-door"]);
    }

    #[test]
    fn multiple_handlers_per_event_all_run() {
        let sink = Arc::new(Mutex::new(Vec::new()));
        let mut bus = InProcessEventBus::new();
        bus.register::<Opened, _>(OpenedRecorder {
            tag: "first",
            sink: sink.clone(),
        });
        bus.register::<Opened, _>(OpenedRecorder {
            tag: "second",
            sink: sink.clone(),
        });

        bus.dispatch(&Opened { label: "gate" });

        assert_eq!(*sink.lock(), ["first:gate", "second:gate"]);
    }

    #[test]
    fn dispatch_with_no_registered_handler_is_silent_noop() {
        let bus = InProcessEventBus::new();
        // Must not panic: an event nobody subscribed to simply goes nowhere.
        bus.dispatch(&Opened { label: "unheard" });
    }

    #[test]
    fn dispatch_does_not_route_event_to_handlers_of_other_types() {
        let sink = Arc::new(Mutex::new(Vec::new()));
        let mut bus = InProcessEventBus::new();
        bus.register::<Opened, _>(OpenedRecorder {
            tag: "h",
            sink: sink.clone(),
        });
        bus.register::<Closed, _>(ClosedRecorder { sink: sink.clone() });

        bus.dispatch(&Closed);

        assert_eq!(*sink.lock(), ["closed"], "Opened handler must not see Closed");
    }
}
