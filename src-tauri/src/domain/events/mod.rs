//! Domain events — the DDD/CQRS eventing seam.
//!
//! Each event is its own struct implementing [`DomainEvent`], so a handler
//! can subscribe to exactly one concrete event type. Aggregates buffer events
//! as they mutate (via [`AggregateRoot::apply`](crate::domain::aggregate_root::AggregateRoot::apply));
//! the use case drains and publishes them after the repository write commits.
//!
//! Note: events emitted by aggregates (`Invoice`, `Client`, `Payment`) live
//! here in the domain. `BackupCreated` is *not* here — there is no backup
//! aggregate, it is emitted by an application use case, so it lives in
//! `application::data_usecases` and implements [`DomainEvent`] from there.

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use downcast_rs::{impl_downcast, Downcast};

pub mod catalog_item_events;
pub mod client_events;
pub mod invoice_events;
pub mod payment_events;
pub mod tax_events;

/// A fact that something happened in the domain. Implemented by one struct
/// per event type so handlers can subscribe to a single concrete event.
///
/// The `Downcast` supertrait (from `downcast-rs`) lets the event bus recover
/// the concrete type for typed dispatch without any hand-written `as_any`
/// boilerplate on each event.
pub trait DomainEvent: Downcast + Debug + Send + Sync {
    /// When the change happened. Carried on the event so projection is
    /// deterministic in tests rather than relying on `Utc::now()` at dispatch
    /// time.
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Stable, dotted identifier written verbatim into the `activities` row's
    /// `event_type` column, e.g. `"invoice.finalized"`.
    fn event_name(&self) -> &'static str;
}
impl_downcast!(DomainEvent);

/// Holds the events an aggregate has buffered but not yet published.
///
/// Wrapped in a newtype so the aggregates that embed it (`Invoice`, `Client`,
/// `Payment`) keep their `#[derive(Clone, PartialEq, Eq)]` — `Box<dyn
/// DomainEvent>` implements none of those. Pending events are transient
/// bookkeeping, not part of an aggregate's identity or value, so:
///
/// - `Clone` yields an **empty** buffer: a cloned aggregate has not itself
///   recorded anything. In practice aggregates are never cloned between
///   `apply` and the use case's drain, so nothing is lost.
/// - `PartialEq` is **always equal**: two aggregates with the same persisted
///   state compare equal regardless of what either has buffered.
#[derive(Default)]
pub struct EventBuffer(Vec<Box<dyn DomainEvent>>);

impl EventBuffer {
    /// Buffer one event.
    pub fn push<E: DomainEvent>(&mut self, event: E) {
        self.0.push(Box::new(event));
    }

    /// Drain every buffered event, leaving the buffer empty.
    pub fn take(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Clone for EventBuffer {
    /// A clone has not itself recorded anything — see the type-level docs.
    fn clone(&self) -> Self {
        Self(Vec::new())
    }
}

impl PartialEq for EventBuffer {
    /// Pending events are not part of an aggregate's value — see the
    /// type-level docs.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for EventBuffer {}

impl Debug for EventBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventBuffer({} pending)", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Tick {
        at: DateTime<Utc>,
    }
    impl DomainEvent for Tick {
        fn occurred_at(&self) -> DateTime<Utc> {
            self.at
        }
        fn event_name(&self) -> &'static str {
            "test.tick"
        }
    }

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn push_then_take_drains_buffered_events() {
        let mut buf = EventBuffer::default();
        buf.push(Tick { at: at() });
        buf.push(Tick { at: at() });
        assert_eq!(buf.len(), 2);

        let taken = buf.take();
        assert_eq!(taken.len(), 2);
        assert_eq!(taken[0].event_name(), "test.tick");
        assert!(buf.is_empty(), "take must leave the buffer empty");
    }

    #[test]
    fn clone_yields_empty_buffer() {
        let mut buf = EventBuffer::default();
        buf.push(Tick { at: at() });

        let cloned = buf.clone();
        assert!(cloned.is_empty(), "a clone has not itself recorded anything");
        assert_eq!(buf.len(), 1, "cloning must not drain the original");
    }

    #[test]
    fn partial_eq_ignores_buffered_events() {
        let mut a = EventBuffer::default();
        let b = EventBuffer::default();
        a.push(Tick { at: at() });
        assert_eq!(a, b, "pending events are not part of aggregate value");
    }

    #[test]
    fn downcast_recovers_concrete_event_type() {
        let mut buf = EventBuffer::default();
        buf.push(Tick { at: at() });
        let taken = buf.take();
        let ev: &dyn DomainEvent = taken[0].as_ref();
        assert!(ev.downcast_ref::<Tick>().is_some());
    }
}
