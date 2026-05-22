//! Field-level diff describing what changed on an aggregate during an update.
//!
//! Produced by [`AggregateRoot::diff_against`](crate::domain::aggregate_root::AggregateRoot::diff_against)
//! and serialised into the audit row's `metadata_json` so the UI can render
//! "name: Old → New" without re-fetching the entity.
//!
//! Two flavours: [`FieldChange::Scalar`] for plain fields (name, percentage,
//! dates, money, …) and [`FieldChange::Collection`] for `Vec`-typed fields
//! (line items, addresses, allocations). Collection diffs are count-only in
//! v1 — full element-level diffing would be its own project.

use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::{Number, Value};

use crate::domain::money::Money;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldChange {
    /// String / bool / null scalar. Use [`FieldChange::scalar`] /
    /// [`FieldChange::opt`].
    Scalar {
        field: &'static str,
        from: Value,
        to: Value,
    },
    /// Numeric scalar (decimal or integer). `from` / `to` are
    /// `Value::Number` (or `Value::Null` for empty optionals). Use
    /// [`FieldChange::number`].
    Number {
        field: &'static str,
        from: Value,
        to: Value,
    },
    /// Monetary scalar — structured with currency code and decimal amount in
    /// major units, so the frontend never has to parse a pre-formatted
    /// string. Use [`FieldChange::money`] / [`FieldChange::money_opt`].
    Money {
        field: &'static str,
        from: Option<MoneyValue>,
        to: Option<MoneyValue>,
    },
    /// `Vec` field whose contents differ. Count delta only — element-level
    /// Vec diffing is not in scope for collections without a stable element
    /// identity (line items, generic addresses, …).
    Collection {
        field: &'static str,
        from_count: usize,
        to_count: usize,
    },
    /// Element-level diff for a `Vec` whose elements have a stable identity
    /// (e.g. catalog item `prices` keyed by currency, future "tax ids per
    /// country" keyed by country code). Reports `added` / `removed` /
    /// `changed` slices; unchanged elements are not enumerated. Use
    /// [`FieldChange::indexed_collection`].
    IndexedCollection {
        field: &'static str,
        added: Vec<IndexedDelta>,
        removed: Vec<IndexedDelta>,
        changed: Vec<IndexedDelta>,
    },
}

/// One element-level entry inside [`FieldChange::IndexedCollection`].
/// `from` is set on `removed` + `changed`; `to` is set on `added` + `changed`.
/// `key` is the stable identity (e.g. the ISO currency code for a price).
/// `label` is an optional human-friendly rendering of `key` — populated by
/// audit handlers that resolve IDs to user-facing strings (e.g. an invoice
/// UUID → `"#1001"`). Left `None` for keys that are already user-readable
/// (e.g. currency codes).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndexedDelta {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Full element payload as JSON. Populated for `added` (when only `to`
    /// is set) and `removed` (when only `from` is set). For `changed`
    /// entries produced by [`FieldChange::diffable_collection`], this is
    /// left `None` — the per-field sub-diff lives in `changes` instead.
    /// Closure-based [`FieldChange::indexed_collection`] still uses
    /// from+to here for `changed` entries (catalog prices, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Value>,
    /// Recursive per-field sub-diff. Populated on `changed` entries
    /// emitted by [`FieldChange::diffable_collection`]; each entry's
    /// inner [`FieldChange`]s may themselves include nested
    /// `IndexedCollection`s for arbitrarily deep value-object trees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<Vec<FieldChange>>,
}

/// Money rendered for the audit payload: ISO currency code plus the amount
/// expressed in **major units** as a decimal string ("123.45", not "12345"
/// minor units). The string preserves precision exactly across the IPC
/// boundary; the frontend parses it for display or arithmetic.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MoneyValue {
    pub currency: String,
    pub amount: String,
}

impl MoneyValue {
    fn from_money(m: &Money) -> Self {
        let scale = m.currency().minor_unit_scale();
        let amount = if scale == 1 {
            // Zero-fraction currencies (JPY, KRW): no decimal point.
            m.minor_units().to_string()
        } else {
            // Build "123.45" from the minor-unit i64 by hand to avoid any
            // float intermediate. Negative values keep their sign on the
            // whole part.
            let minor = m.minor_units();
            let sign = if minor < 0 { "-" } else { "" };
            let abs = minor.unsigned_abs() as i64;
            let whole = abs / scale;
            let frac = abs % scale;
            let width = m.currency().fraction_digits() as usize;
            format!("{sign}{whole}.{frac:0width$}", width = width)
        };
        Self {
            currency: m.currency().code().to_string(),
            amount,
        }
    }
}

impl FieldChange {
    /// The `field` discriminant on every variant — handy for tests and for
    /// frontend filtering without an exhaustive match.
    pub fn field(&self) -> &'static str {
        match self {
            Self::Scalar { field, .. }
            | Self::Number { field, .. }
            | Self::Money { field, .. }
            | Self::Collection { field, .. }
            | Self::IndexedCollection { field, .. } => field,
        }
    }

    /// Scalar diff between two values. Returns `None` when unchanged so call
    /// sites can chain `.into_iter().flatten()` without per-field branching.
    /// Every scalar in the domain implements `ToString` (numbers via
    /// `Display`, dates as ISO, `Money` via `Money::format`, currencies via
    /// `Currency::code`, strings as themselves), so this is the universal
    /// helper.
    pub fn scalar<T>(field: &'static str, from: &T, to: &T) -> Option<Self>
    where
        T: PartialEq + ToString + ?Sized,
    {
        if from == to {
            None
        } else {
            Some(Self::Scalar {
                field,
                from: Value::String(from.to_string()),
                to: Value::String(to.to_string()),
            })
        }
    }

    /// Optional scalar — `None` is rendered as JSON `null`. Use this for
    /// `Option<T>` fields like `tax_id_number`, `notes`, `due_date`.
    pub fn opt<T>(field: &'static str, from: &Option<T>, to: &Option<T>) -> Option<Self>
    where
        T: PartialEq + ToString,
    {
        if from == to {
            None
        } else {
            Some(Self::Scalar {
                field,
                from: opt_to_value(from),
                to: opt_to_value(to),
            })
        }
    }

    /// Collection diff. Considered changed when the slices differ by
    /// `PartialEq` (counts may match but elements may have moved or been
    /// edited). Reports the from/to counts either way so the UI can say
    /// "1 → 2 addresses" or "addresses changed".
    pub fn collection<T>(field: &'static str, from: &[T], to: &[T]) -> Option<Self>
    where
        T: PartialEq,
    {
        if from == to {
            None
        } else {
            Some(Self::Collection {
                field,
                from_count: from.len(),
                to_count: to.len(),
            })
        }
    }

    /// Numeric scalar (`Decimal`). Emits a JSON number when the value fits;
    /// falls back to a string for the edge case of values too large for
    /// f64-backed `serde_json::Number`. Use for percentages, quantities,
    /// counts, etc.
    pub fn number(field: &'static str, from: &Decimal, to: &Decimal) -> Option<Self> {
        if from == to {
            None
        } else {
            Some(Self::Number {
                field,
                from: decimal_to_value(from),
                to: decimal_to_value(to),
            })
        }
    }

    /// Monetary scalar. Both sides carry the currency code plus the amount
    /// in major units as a decimal string.
    pub fn money(field: &'static str, from: &Money, to: &Money) -> Option<Self> {
        if from == to {
            None
        } else {
            Some(Self::Money {
                field,
                from: Some(MoneyValue::from_money(from)),
                to: Some(MoneyValue::from_money(to)),
            })
        }
    }

    /// Optional monetary scalar — `None` is rendered as JSON `null` on its
    /// side of the diff (so the FE can render "∅ → €100.00" naturally).
    pub fn money_opt(
        field: &'static str,
        from: &Option<Money>,
        to: &Option<Money>,
    ) -> Option<Self> {
        if from == to {
            None
        } else {
            Some(Self::Money {
                field,
                from: from.as_ref().map(MoneyValue::from_money),
                to: to.as_ref().map(MoneyValue::from_money),
            })
        }
    }

    /// Element-level diff for a `Vec<T>` whose elements have a stable
    /// identity. `key_of` extracts the identity (e.g. `|m: &Money|
    /// m.currency().code()`). `value_of` projects each element to JSON
    /// (e.g. `|m| money_to_value(m)`).
    ///
    /// Returns `None` when nothing was added, removed, or changed. Linear
    /// scan — fine for the small collections this targets (≤ tens of
    /// elements). Duplicate keys: first-match-wins, no error.
    pub fn indexed_collection<T, K>(
        field: &'static str,
        from: &[T],
        to: &[T],
        key_of: impl Fn(&T) -> K,
        value_of: impl Fn(&T) -> Value,
    ) -> Option<Self>
    where
        K: PartialEq + ToString,
        T: PartialEq,
    {
        let mut added: Vec<IndexedDelta> = Vec::new();
        let mut removed: Vec<IndexedDelta> = Vec::new();
        let mut changed: Vec<IndexedDelta> = Vec::new();

        for t in to {
            let key = key_of(t);
            match from.iter().find(|f| key_of(f) == key) {
                Some(prev) if prev != t => changed.push(IndexedDelta {
                    key: key.to_string(),
                    label: None,
                    from: Some(value_of(prev)),
                    to: Some(value_of(t)),
                    changes: None,
                }),
                Some(_) => {} // unchanged — omit
                None => added.push(IndexedDelta {
                    key: key.to_string(),
                    label: None,
                    from: None,
                    to: Some(value_of(t)),
                    changes: None,
                }),
            }
        }
        for f in from {
            let key = key_of(f);
            if !to.iter().any(|t| key_of(t) == key) {
                removed.push(IndexedDelta {
                    key: key.to_string(),
                    label: None,
                    from: Some(value_of(f)),
                    to: None,
                    changes: None,
                });
            }
        }

        if added.is_empty() && removed.is_empty() && changed.is_empty() {
            None
        } else {
            Some(Self::IndexedCollection {
                field,
                added,
                removed,
                changed,
            })
        }
    }
}

/// Project a `Money` to JSON for use as the `value_of` closure of
/// [`FieldChange::indexed_collection`]. Mirrors what [`FieldChange::money`]
/// emits inside its variant.
pub fn money_to_value(m: &Money) -> Value {
    serde_json::to_value(MoneyValue::from_money(m)).unwrap_or(Value::Null)
}

/// Implemented by value-object collection elements (line items, contact
/// entries, addresses, allocations, …) so they can be diffed recursively
/// by [`FieldChange::diffable_collection`]. Each element supplies:
///
/// - `audit_key` — stable identity for matching elements across before/after
///   (typically the entity's UUID for entities with one, or a stable content
///   key like ISO currency code for value-keyed lists).
/// - `audit_label` — optional human-readable name surfaced in the audit row
///   (e.g. the email value, an address city, the line item description).
/// - `to_audit_json` — full element payload for `added` / `removed` entries.
///   Not used for `changed` (the per-field sub-diff in `changes` is richer).
/// - `diff_against` — per-field [`FieldChange`] list comparing self vs. an
///   older copy of the same element. May itself emit nested
///   `IndexedCollection`s, enabling arbitrarily deep recursion.
pub trait DiffableValue: PartialEq {
    fn audit_key(&self) -> String;
    fn audit_label(&self) -> Option<String> {
        None
    }
    fn to_audit_json(&self) -> Value;
    fn diff_against(&self, before: &Self) -> Vec<FieldChange>;
}

impl FieldChange {
    /// Element-level recursive diff for a `Vec<T>` of value objects. Each
    /// `changed` entry carries its own [`FieldChange`] list (the element's
    /// `diff_against` result), so the frontend can render per-sub-field
    /// changes instead of dumping the whole row. Use this for line items,
    /// contact entries, addresses, allocations — anything that has stable
    /// identity AND meaningful per-field deltas.
    ///
    /// Returns `None` when nothing was added, removed, or changed.
    pub fn diffable_collection<T: DiffableValue>(
        field: &'static str,
        from: &[T],
        to: &[T],
    ) -> Option<Self> {
        let mut added: Vec<IndexedDelta> = Vec::new();
        let mut removed: Vec<IndexedDelta> = Vec::new();
        let mut changed: Vec<IndexedDelta> = Vec::new();

        for t in to {
            let key = t.audit_key();
            match from.iter().find(|f| f.audit_key() == key) {
                Some(prev) if prev != t => {
                    let sub = t.diff_against(prev);
                    if !sub.is_empty() {
                        changed.push(IndexedDelta {
                            key,
                            label: t.audit_label(),
                            from: None,
                            to: None,
                            changes: Some(sub),
                        });
                    }
                }
                Some(_) => {} // unchanged — omit
                None => added.push(IndexedDelta {
                    key,
                    label: t.audit_label(),
                    from: None,
                    to: Some(t.to_audit_json()),
                    changes: None,
                }),
            }
        }
        for f in from {
            let key = f.audit_key();
            if !to.iter().any(|t| t.audit_key() == key) {
                removed.push(IndexedDelta {
                    key,
                    label: f.audit_label(),
                    from: Some(f.to_audit_json()),
                    to: None,
                    changes: None,
                });
            }
        }

        if added.is_empty() && removed.is_empty() && changed.is_empty() {
            None
        } else {
            Some(Self::IndexedCollection {
                field,
                added,
                removed,
                changed,
            })
        }
    }

    /// Element-level diff for a `Vec<String>`. Each element is its own key
    /// (the string itself), so there is no `changed` bucket — only added
    /// and removed strings. Use for tag-style lists. Order-insensitive.
    pub fn string_collection(
        field: &'static str,
        from: &[String],
        to: &[String],
    ) -> Option<Self> {
        let mut added: Vec<IndexedDelta> = Vec::new();
        let mut removed: Vec<IndexedDelta> = Vec::new();

        for s in to {
            if !from.iter().any(|f| f == s) {
                added.push(IndexedDelta {
                    key: s.clone(),
                    label: None,
                    from: None,
                    to: Some(Value::String(s.clone())),
                    changes: None,
                });
            }
        }
        for s in from {
            if !to.iter().any(|t| t == s) {
                removed.push(IndexedDelta {
                    key: s.clone(),
                    label: None,
                    from: Some(Value::String(s.clone())),
                    to: None,
                    changes: None,
                });
            }
        }

        if added.is_empty() && removed.is_empty() {
            None
        } else {
            Some(Self::IndexedCollection {
                field,
                added,
                removed,
                changed: vec![],
            })
        }
    }
}

fn opt_to_value<T: ToString>(opt: &Option<T>) -> Value {
    match opt {
        Some(v) => Value::String(v.to_string()),
        None => Value::Null,
    }
}

fn decimal_to_value(d: &Decimal) -> Value {
    // Try a JSON number for FE convenience; if precision would be lost
    // (very rare for percentages/quantities) fall back to a string.
    let s = d.to_string();
    if let Ok(n) = s.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = Number::from_f64(f) {
            return Value::Number(n);
        }
    }
    Value::String(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_returns_none_when_unchanged() {
        assert!(FieldChange::scalar("name", "Acme", "Acme").is_none());
    }

    #[test]
    fn scalar_returns_change_with_string_values() {
        let c = FieldChange::scalar("name", &"Old", &"New").unwrap();
        match c {
            FieldChange::Scalar { field, from, to } => {
                assert_eq!(field, "name");
                assert_eq!(from, Value::String("Old".into()));
                assert_eq!(to, Value::String("New".into()));
            }
            _ => panic!("expected Scalar"),
        }
    }

    #[test]
    fn opt_none_to_some_renders_null_then_value() {
        let from: Option<String> = None;
        let to = Some("BE0123".to_string());
        let c = FieldChange::opt("tax_id", &from, &to).unwrap();
        match c {
            FieldChange::Scalar { from, to, .. } => {
                assert_eq!(from, Value::Null);
                assert_eq!(to, Value::String("BE0123".into()));
            }
            _ => panic!("expected Scalar"),
        }
    }

    #[test]
    fn opt_unchanged_returns_none_for_both_some_and_none() {
        let none_a: Option<String> = None;
        let none_b: Option<String> = None;
        assert!(FieldChange::opt("notes", &none_a, &none_b).is_none());

        let same = Some("x".to_string());
        let same2 = Some("x".to_string());
        assert!(FieldChange::opt("notes", &same, &same2).is_none());
    }

    #[test]
    fn collection_equal_returns_none() {
        assert!(FieldChange::collection("nums", &[1, 2, 3], &[1, 2, 3]).is_none());
    }

    #[test]
    fn collection_count_delta_reported() {
        let c = FieldChange::collection("nums", &[1, 2], &[1, 2, 3]).unwrap();
        match c {
            FieldChange::Collection {
                field,
                from_count,
                to_count,
            } => {
                assert_eq!(field, "nums");
                assert_eq!(from_count, 2);
                assert_eq!(to_count, 3);
            }
            _ => panic!("expected Collection"),
        }
    }

    #[test]
    fn collection_same_count_but_different_contents_still_changed() {
        let c = FieldChange::collection("nums", &[1, 2, 3], &[1, 2, 4]).unwrap();
        match c {
            FieldChange::Collection {
                from_count, to_count, ..
            } => {
                assert_eq!(from_count, 3);
                assert_eq!(to_count, 3);
            }
            _ => panic!("expected Collection"),
        }
    }

    #[test]
    fn money_emits_currency_and_decimal_amount() {
        use crate::domain::money::{Currency, Money};
        let from = Money::from_minor(10_000, Currency::Eur); // €100.00
        let to = Money::from_minor(12_345, Currency::Eur); // €123.45
        let c = FieldChange::money("price", &from, &to).unwrap();
        match c {
            FieldChange::Money { from, to, .. } => {
                let from = from.unwrap();
                let to = to.unwrap();
                assert_eq!(from.currency, "EUR");
                assert_eq!(from.amount, "100.00");
                assert_eq!(to.currency, "EUR");
                assert_eq!(to.amount, "123.45");
            }
            _ => panic!("expected Money"),
        }
    }

    #[test]
    fn money_zero_fraction_currency_has_no_decimal_point() {
        use crate::domain::money::{Currency, Money};
        let from = Money::from_minor(100, Currency::Jpy);
        let to = Money::from_minor(250, Currency::Jpy);
        let c = FieldChange::money("price", &from, &to).unwrap();
        match c {
            FieldChange::Money { from, to, .. } => {
                assert_eq!(from.unwrap().amount, "100");
                assert_eq!(to.unwrap().amount, "250");
            }
            _ => panic!("expected Money"),
        }
    }

    #[test]
    fn money_opt_renders_null_for_none() {
        use crate::domain::money::{Currency, Money};
        let from: Option<Money> = None;
        let to = Some(Money::from_minor(500, Currency::Eur));
        let c = FieldChange::money_opt("price", &from, &to).unwrap();
        match c {
            FieldChange::Money { from, to, .. } => {
                assert!(from.is_none());
                let to = to.unwrap();
                assert_eq!(to.amount, "5.00");
            }
            _ => panic!("expected Money"),
        }
    }

    #[test]
    fn money_negative_keeps_sign_on_whole_part() {
        use crate::domain::money::{Currency, Money};
        let from = Money::from_minor(0, Currency::Eur);
        let to = Money::from_minor(-12_345, Currency::Eur);
        let c = FieldChange::money("balance", &from, &to).unwrap();
        match c {
            FieldChange::Money { to, .. } => {
                assert_eq!(to.unwrap().amount, "-123.45");
            }
            _ => panic!("expected Money"),
        }
    }

    #[test]
    fn number_emits_json_number_for_decimals() {
        use rust_decimal_macros::dec;
        let c = FieldChange::number("percentage", &dec!(21), &dec!(20)).unwrap();
        match c {
            FieldChange::Number { from, to, .. } => {
                assert!(from.is_number());
                assert!(to.is_number());
            }
            _ => panic!("expected Number"),
        }
    }

    #[test]
    fn indexed_collection_returns_none_when_unchanged() {
        use crate::domain::money::{Currency, Money};
        let prices = vec![Money::from_minor(10_000, Currency::Eur)];
        let same = prices.clone();
        let result = FieldChange::indexed_collection(
            "prices",
            &prices,
            &same,
            |m: &Money| m.currency().code(),
            money_to_value,
        );
        assert!(result.is_none());
    }

    #[test]
    fn indexed_collection_classifies_added_removed_and_changed() {
        use crate::domain::money::{Currency, Money};
        let from = vec![
            Money::from_minor(10_000, Currency::Eur), // will change
            Money::from_minor(5_000, Currency::Jpy),  // will be removed
        ];
        let to = vec![
            Money::from_minor(12_000, Currency::Eur), // changed
            Money::from_minor(8_000, Currency::Usd),  // added
        ];
        let c = FieldChange::indexed_collection(
            "prices",
            &from,
            &to,
            |m: &Money| m.currency().code(),
            money_to_value,
        )
        .unwrap();
        match c {
            FieldChange::IndexedCollection {
                field,
                added,
                removed,
                changed,
            } => {
                assert_eq!(field, "prices");
                assert_eq!(added.len(), 1);
                assert_eq!(added[0].key, "USD");
                // `to` is a Money JSON object; `from` is None for an addition.
                assert!(added[0].from.is_none());
                assert_eq!(added[0].to.as_ref().unwrap()["currency"], "USD");
                assert_eq!(added[0].to.as_ref().unwrap()["amount"], "80.00");

                assert_eq!(removed.len(), 1);
                assert_eq!(removed[0].key, "JPY");
                assert!(removed[0].to.is_none());

                assert_eq!(changed.len(), 1);
                assert_eq!(changed[0].key, "EUR");
                assert_eq!(changed[0].from.as_ref().unwrap()["amount"], "100.00");
                assert_eq!(changed[0].to.as_ref().unwrap()["amount"], "120.00");
            }
            _ => panic!("expected IndexedCollection"),
        }
    }

    #[test]
    fn indexed_collection_serializes_with_added_removed_changed_keys() {
        use crate::domain::money::{Currency, Money};
        let c = FieldChange::indexed_collection(
            "prices",
            &[Money::from_minor(10_000, Currency::Eur)],
            &[Money::from_minor(12_000, Currency::Eur)],
            |m: &Money| m.currency().code(),
            money_to_value,
        )
        .unwrap();
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["kind"], "indexed_collection");
        assert_eq!(json["field"], "prices");
        assert_eq!(json["added"].as_array().unwrap().len(), 0);
        assert_eq!(json["removed"].as_array().unwrap().len(), 0);
        assert_eq!(json["changed"][0]["key"], "EUR");
        assert_eq!(json["changed"][0]["from"]["amount"], "100.00");
    }

    #[test]
    fn diffable_collection_emits_recursive_sub_diff_on_changed() {
        #[derive(Debug, Clone, PartialEq)]
        struct Row {
            id: u32,
            name: String,
            count: rust_decimal::Decimal,
        }
        impl DiffableValue for Row {
            fn audit_key(&self) -> String {
                self.id.to_string()
            }
            fn audit_label(&self) -> Option<String> {
                Some(self.name.clone())
            }
            fn to_audit_json(&self) -> Value {
                serde_json::json!({ "name": self.name, "count": self.count.to_string() })
            }
            fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
                [
                    FieldChange::scalar("name", &before.name, &self.name),
                    FieldChange::number("count", &before.count, &self.count),
                ]
                .into_iter()
                .flatten()
                .collect()
            }
        }

        use rust_decimal_macros::dec;
        let from = vec![
            Row { id: 1, name: "Alpha".into(), count: dec!(2) },
            Row { id: 2, name: "Bravo".into(), count: dec!(5) },
            Row { id: 3, name: "Charlie".into(), count: dec!(1) },
        ];
        let to = vec![
            Row { id: 1, name: "Alpha".into(), count: dec!(2) }, // unchanged
            Row { id: 2, name: "Bravo".into(), count: dec!(7) }, // count changed
            Row { id: 4, name: "Delta".into(), count: dec!(3) }, // added (3 removed)
        ];

        let diff = FieldChange::diffable_collection("rows", &from, &to).unwrap();
        match diff {
            FieldChange::IndexedCollection { added, removed, changed, .. } => {
                assert_eq!(added.len(), 1);
                assert_eq!(added[0].key, "4");
                assert_eq!(added[0].label.as_deref(), Some("Delta"));
                assert!(added[0].changes.is_none());
                assert_eq!(added[0].to.as_ref().unwrap()["name"], "Delta");

                assert_eq!(removed.len(), 1);
                assert_eq!(removed[0].key, "3");
                assert!(removed[0].changes.is_none());

                assert_eq!(changed.len(), 1);
                assert_eq!(changed[0].key, "2");
                assert_eq!(changed[0].label.as_deref(), Some("Bravo"));
                assert!(changed[0].from.is_none(), "changed entries skip from/to");
                let sub = changed[0].changes.as_ref().unwrap();
                assert_eq!(sub.len(), 1, "only count changed, not name");
                assert_eq!(sub[0].field(), "count");
            }
            _ => panic!("expected IndexedCollection"),
        }
    }

    #[test]
    fn diffable_collection_unchanged_returns_none() {
        #[derive(Debug, Clone, PartialEq)]
        struct Row { id: u32, value: String }
        impl DiffableValue for Row {
            fn audit_key(&self) -> String { self.id.to_string() }
            fn to_audit_json(&self) -> Value { serde_json::json!({ "value": self.value }) }
            fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
                FieldChange::scalar("value", &before.value, &self.value).into_iter().collect()
            }
        }
        let rows = vec![Row { id: 1, value: "a".into() }];
        assert!(FieldChange::diffable_collection("rows", &rows, &rows).is_none());
    }

    #[test]
    fn string_collection_reports_added_and_removed_only() {
        let from: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let to: Vec<String> = vec!["a".into(), "c".into(), "d".into()];
        let diff = FieldChange::string_collection("tags", &from, &to).unwrap();
        match diff {
            FieldChange::IndexedCollection { added, removed, changed, .. } => {
                assert_eq!(added.len(), 1);
                assert_eq!(added[0].key, "d");
                assert_eq!(added[0].to.as_ref().unwrap(), &Value::String("d".into()));
                assert_eq!(removed.len(), 1);
                assert_eq!(removed[0].key, "b");
                assert!(changed.is_empty(), "strings cannot change in place");
            }
            _ => panic!("expected IndexedCollection"),
        }
    }

    #[test]
    fn string_collection_unchanged_returns_none() {
        let xs: Vec<String> = vec!["a".into(), "b".into()];
        assert!(FieldChange::string_collection("tags", &xs, &xs).is_none());
        // Order-insensitive:
        let reordered = vec!["b".into(), "a".into()];
        assert!(FieldChange::string_collection("tags", &xs, &reordered).is_none());
    }

    #[test]
    fn serializes_with_kind_discriminator() {
        let c = FieldChange::scalar("name", &"a", &"b").unwrap();
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["kind"], "scalar");
        assert_eq!(json["field"], "name");
        assert_eq!(json["from"], "a");
        assert_eq!(json["to"], "b");

        let coll = FieldChange::collection("xs", &[1], &[1, 2]).unwrap();
        let json = serde_json::to_value(&coll).unwrap();
        assert_eq!(json["kind"], "collection");
        assert_eq!(json["field"], "xs");
        assert_eq!(json["from_count"], 1);
        assert_eq!(json["to_count"], 2);
    }
}
