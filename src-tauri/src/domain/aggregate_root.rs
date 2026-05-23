//! The [`AggregateRoot`] trait — the DDD base shared by `Invoice`, `Client`
//! and `Payment`. Mirrors the `apply` ergonomics of NestJS's `@nestjs/cqrs`
//! `AggregateRoot`.
//!
//! Deliberate layering split: `AggregateRoot` lives in the domain and only
//! *buffers* events (`apply`) and *drains* them (`take_events`). Publishing
//! to the bus is the application layer's job — see `CommitEvents` in
//! `application::ports::event_bus`, which adds the NestJS-style `commit(bus)`
//! call. This preserves the domain's "no dependencies" rule: the domain never
//! names the `EventBus` port.

use crate::domain::events::{DomainEvent, EventBuffer};
use crate::domain::field_change::FieldChange;

pub trait AggregateRoot {
    /// Mutable access to the aggregate's event buffer. The only method an
    /// implementor must provide — everything else is a default.
    fn pending_events_mut(&mut self) -> &mut EventBuffer;

    /// Buffer a domain event. Mirrors NestJS `this.apply(event)`.
    fn apply<E: DomainEvent>(&mut self, event: E) {
        self.pending_events_mut().push(event);
    }

    /// Drain every buffered event, leaving the aggregate's buffer empty. The
    /// application layer calls this (via `CommitEvents::commit`) after the
    /// repository write has committed.
    fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        self.pending_events_mut().take()
    }

    /// Field-level diff between this aggregate and a prior snapshot. Use cases
    /// clone the aggregate before mutating, then call this to build the
    /// `changes` payload of an `Updated` domain event.
    ///
    /// Default returns empty so aggregates without an `Updated` event don't
    /// need to opt in. Override per concrete type using the
    /// [`FieldChange::scalar`] / [`FieldChange::opt`] / [`FieldChange::collection`]
    /// helpers — each returns `Option<FieldChange>` so the body is just a
    /// flat list flattened into a `Vec`.
    fn diff_against(&self, _before: &Self) -> Vec<FieldChange> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[derive(Debug)]
    struct Pinged;
    impl DomainEvent for Pinged {
        fn occurred_at(&self) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        }
        fn event_name(&self) -> &'static str {
            "test.pinged"
        }
    }

    #[derive(Default)]
    struct Gadget {
        events: EventBuffer,
    }
    impl AggregateRoot for Gadget {
        fn pending_events_mut(&mut self) -> &mut EventBuffer {
            &mut self.events
        }
    }

    #[test]
    fn apply_buffers_event_and_take_events_drains_it() {
        let mut g = Gadget::default();
        g.apply(Pinged);
        g.apply(Pinged);

        let drained = g.take_events();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].event_name(), "test.pinged");
        assert!(g.take_events().is_empty(), "second drain yields nothing");
    }
}
