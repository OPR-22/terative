//! The event-bus port: how the application layer publishes domain events to
//! handlers without knowing which handlers exist.
//!
//! - [`EventHandler<E>`] — implemented once per (handler, event-type) pair. A
//!   handler only ever sees its own concrete event type.
//! - [`EventBus`] — object-safe publish side. The concrete implementation
//!   (`adapters::event_bus::InProcessEventBus`) exposes a typed `register`
//!   method; `register` can't live on the trait because it is generic over
//!   the event and handler types, which would make the trait non-object-safe.
//! - [`CommitEvents`] — the NestJS-style `aggregate.commit(bus)` ergonomic,
//!   provided as a blanket extension over every [`AggregateRoot`]. It is a
//!   convenience helper, not a port (nothing in `adapters` implements it).

use crate::domain::aggregate_root::AggregateRoot;
use crate::domain::events::DomainEvent;

/// Reacts to one concrete domain-event type. Many handlers may exist for the
/// same event — the bus fans out to all of them.
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    fn handle(&self, event: &E);
}

/// Object-safe publish side of the bus.
pub trait EventBus: Send + Sync {
    /// Fan the event out to every handler registered for its concrete type.
    /// Never fails: handlers swallow their own errors, because the audit
    /// log is a UX read-model, not a transactional invariant.
    fn dispatch(&self, event: &dyn DomainEvent);
}

/// An `EventBus` that drops every event. It is the default a use case holds
/// until production wiring injects the real in-process bus via
/// `with_events(..)`, and it keeps event-agnostic tests from having to build
/// a bus they don't care about.
pub struct NoopEventBus;

impl EventBus for NoopEventBus {
    fn dispatch(&self, _event: &dyn DomainEvent) {}
}

/// Blanket extension giving every aggregate the NestJS-style `commit(bus)`
/// call: drain buffered events and publish them. Use cases call this *after*
/// the repository write has committed, so a handler failure can never roll
/// back a successful business action.
pub trait CommitEvents: AggregateRoot {
    fn commit(&mut self, bus: &dyn EventBus) {
        for event in self.take_events() {
            bus.dispatch(event.as_ref());
        }
    }
}

impl<T: AggregateRoot> CommitEvents for T {}

#[cfg(test)]
pub mod test_support {
    use super::{DomainEvent, EventBus};
    use parking_lot::Mutex;

    /// An `EventBus` that records the `event_name` of everything dispatched.
    /// Use-case tests assert on these names to prove the use case actually
    /// drained and published its aggregate's events — payload correctness is
    /// covered by the domain and handler tests.
    #[derive(Default)]
    pub struct CollectingEventBus {
        pub names: Mutex<Vec<&'static str>>,
    }

    impl CollectingEventBus {
        /// Snapshot of the event names dispatched so far.
        pub fn names(&self) -> Vec<&'static str> {
            self.names.lock().clone()
        }
    }

    impl EventBus for CollectingEventBus {
        fn dispatch(&self, event: &dyn DomainEvent) {
            self.names.lock().push(event.event_name());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::EventBuffer;
    use chrono::{DateTime, Utc};
    use parking_lot::Mutex;

    #[derive(Debug)]
    struct Stirred;
    impl DomainEvent for Stirred {
        fn occurred_at(&self) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        }
        fn event_name(&self) -> &'static str {
            "test.stirred"
        }
    }

    #[derive(Default)]
    struct Cocktail {
        events: EventBuffer,
    }
    impl AggregateRoot for Cocktail {
        fn pending_events_mut(&mut self) -> &mut EventBuffer {
            &mut self.events
        }
    }

    /// Records the `event_name` of every event it is handed.
    #[derive(Default)]
    struct RecordingBus(Mutex<Vec<&'static str>>);
    impl EventBus for RecordingBus {
        fn dispatch(&self, event: &dyn DomainEvent) {
            self.0.lock().push(event.event_name());
        }
    }

    #[test]
    fn commit_drains_aggregate_events_into_the_bus() {
        let mut agg = Cocktail::default();
        agg.apply(Stirred);
        agg.apply(Stirred);

        let bus = RecordingBus::default();
        agg.commit(&bus);

        assert_eq!(*bus.0.lock(), ["test.stirred", "test.stirred"]);
    }

    #[test]
    fn commit_is_idempotent_once_drained() {
        let mut agg = Cocktail::default();
        agg.apply(Stirred);

        let bus = RecordingBus::default();
        agg.commit(&bus);
        agg.commit(&bus); // nothing left to publish

        assert_eq!(bus.0.lock().len(), 1);
    }
}
