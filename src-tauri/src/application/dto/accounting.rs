use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::MoneyDto;
use super::invoice::InvoiceStatusDto;
use crate::application::accounting_usecases::{RevenueByClientInput, RevenueByPeriodInput};
use crate::application::ports::{
    AgingBucket, AgingRow, ClientBalance, DashboardSummary, DerivedPaymentStatus,
    InvoicePaymentRow, RevenueBucket, RevenueByClient, RevenueGrouping,
};

// ---- DerivedPaymentStatusDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum DerivedPaymentStatusDto {
    Draft,
    Unpaid,
    Partial,
    Paid,
    Overdue,
    Cancelled,
}

impl From<DerivedPaymentStatus> for DerivedPaymentStatusDto {
    fn from(s: DerivedPaymentStatus) -> Self {
        match s {
            DerivedPaymentStatus::Draft => Self::Draft,
            DerivedPaymentStatus::Unpaid => Self::Unpaid,
            DerivedPaymentStatus::Partial => Self::Partial,
            DerivedPaymentStatus::Paid => Self::Paid,
            DerivedPaymentStatus::Overdue => Self::Overdue,
            DerivedPaymentStatus::Cancelled => Self::Cancelled,
        }
    }
}

// ---- RevenueGroupingDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum RevenueGroupingDto {
    Day,
    Month,
    Year,
}

impl From<RevenueGrouping> for RevenueGroupingDto {
    fn from(g: RevenueGrouping) -> Self {
        match g {
            RevenueGrouping::Day => Self::Day,
            RevenueGrouping::Month => Self::Month,
            RevenueGrouping::Year => Self::Year,
        }
    }
}

impl From<RevenueGroupingDto> for RevenueGrouping {
    fn from(dto: RevenueGroupingDto) -> Self {
        match dto {
            RevenueGroupingDto::Day => Self::Day,
            RevenueGroupingDto::Month => Self::Month,
            RevenueGroupingDto::Year => Self::Year,
        }
    }
}

// ---- AgingBucketDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum AgingBucketDto {
    Current,
    Days1To30,
    Days31To60,
    Days61To90,
    Days91Plus,
}

impl From<AgingBucket> for AgingBucketDto {
    fn from(b: AgingBucket) -> Self {
        match b {
            AgingBucket::Current => Self::Current,
            AgingBucket::Days1To30 => Self::Days1To30,
            AgingBucket::Days31To60 => Self::Days31To60,
            AgingBucket::Days61To90 => Self::Days61To90,
            AgingBucket::Days91Plus => Self::Days91Plus,
        }
    }
}

// ---- InvoicePaymentRowDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct InvoicePaymentRowDto {
    pub invoice_id: Uuid,
    pub number: Option<u64>,
    pub client_id: Uuid,
    pub client_name: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub total: MoneyDto,
    pub amount_paid: MoneyDto,
    pub amount_due: MoneyDto,
    pub status: InvoiceStatusDto,
    pub payment_status: DerivedPaymentStatusDto,
}

impl From<&InvoicePaymentRow> for InvoicePaymentRowDto {
    fn from(r: &InvoicePaymentRow) -> Self {
        Self {
            invoice_id: r.invoice_id.0,
            number: r.number,
            client_id: r.client_id.0,
            client_name: r.client_name.clone(),
            date: r.date,
            due_date: r.due_date,
            total: (&r.total).into(),
            amount_paid: (&r.amount_paid).into(),
            amount_due: (&r.amount_due).into(),
            status: r.status.into(),
            payment_status: r.payment_status.into(),
        }
    }
}

// ---- RevenueBucketDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RevenueBucketDto {
    pub bucket_start: NaiveDate,
    pub amount: MoneyDto,
    pub invoice_count: u64,
}

impl From<&RevenueBucket> for RevenueBucketDto {
    fn from(b: &RevenueBucket) -> Self {
        Self {
            bucket_start: b.bucket_start,
            amount: (&b.amount).into(),
            invoice_count: b.invoice_count,
        }
    }
}

// ---- RevenueByClientDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RevenueByClientDto {
    pub client_id: Uuid,
    pub client_name: String,
    pub total_invoiced: MoneyDto,
    pub invoice_count: u64,
}

impl From<&RevenueByClient> for RevenueByClientDto {
    fn from(r: &RevenueByClient) -> Self {
        Self {
            client_id: r.client_id.0,
            client_name: r.client_name.clone(),
            total_invoiced: (&r.total_invoiced).into(),
            invoice_count: r.invoice_count,
        }
    }
}

// ---- ClientBalanceDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ClientBalanceDto {
    pub client_id: Uuid,
    pub client_name: String,
    pub total_invoiced: MoneyDto,
    pub total_paid: MoneyDto,
    pub outstanding: MoneyDto,
}

impl From<&ClientBalance> for ClientBalanceDto {
    fn from(b: &ClientBalance) -> Self {
        Self {
            client_id: b.client_id.0,
            client_name: b.client_name.clone(),
            total_invoiced: (&b.total_invoiced).into(),
            total_paid: (&b.total_paid).into(),
            outstanding: (&b.outstanding).into(),
        }
    }
}

// ---- AgingRowDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AgingRowDto {
    pub invoice_id: Uuid,
    pub number: Option<u64>,
    pub client_id: Uuid,
    pub client_name: String,
    pub total: MoneyDto,
    pub amount_due: MoneyDto,
    pub due_date: Option<NaiveDate>,
    pub bucket: AgingBucketDto,
}

impl From<&AgingRow> for AgingRowDto {
    fn from(r: &AgingRow) -> Self {
        Self {
            invoice_id: r.invoice_id.0,
            number: r.number,
            client_id: r.client_id.0,
            client_name: r.client_name.clone(),
            total: (&r.total).into(),
            amount_due: (&r.amount_due).into(),
            due_date: r.due_date,
            bucket: r.bucket.into(),
        }
    }
}

// ---- DashboardSummaryDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DashboardSummaryDto {
    pub revenue_this_year: MoneyDto,
    pub outstanding_total: MoneyDto,
    pub overdue_count: u64,
    pub draft_count: u64,
    pub finalized_count: u64,
    pub sent_count: u64,
}

impl From<&DashboardSummary> for DashboardSummaryDto {
    fn from(s: &DashboardSummary) -> Self {
        Self {
            revenue_this_year: (&s.revenue_this_year).into(),
            outstanding_total: (&s.outstanding_total).into(),
            overdue_count: s.overdue_count,
            draft_count: s.draft_count,
            finalized_count: s.finalized_count,
            sent_count: s.sent_count,
        }
    }
}

// ---- RevenueByPeriodInputDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RevenueByPeriodInputDto {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub grouping: RevenueGroupingDto,
}

impl From<RevenueByPeriodInputDto> for RevenueByPeriodInput {
    fn from(dto: RevenueByPeriodInputDto) -> Self {
        RevenueByPeriodInput {
            start: dto.start,
            end: dto.end,
            grouping: dto.grouping.into(),
        }
    }
}

// ---- RevenueByClientInputDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RevenueByClientInputDto {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl From<RevenueByClientInputDto> for RevenueByClientInput {
    fn from(dto: RevenueByClientInputDto) -> Self {
        RevenueByClientInput {
            start: dto.start,
            end: dto.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::client::ClientId;
    use crate::domain::money::{Currency, Money};

    #[test]
    fn derived_payment_status_covers_all_variants() {
        for status in [
            DerivedPaymentStatus::Draft,
            DerivedPaymentStatus::Unpaid,
            DerivedPaymentStatus::Partial,
            DerivedPaymentStatus::Paid,
            DerivedPaymentStatus::Overdue,
            DerivedPaymentStatus::Cancelled,
        ] {
            let _dto: DerivedPaymentStatusDto = status.into();
        }
    }

    #[test]
    fn revenue_grouping_round_trips() {
        for g in [
            RevenueGrouping::Day,
            RevenueGrouping::Month,
            RevenueGrouping::Year,
        ] {
            let dto: RevenueGroupingDto = g.into();
            let back: RevenueGrouping = dto.into();
            assert_eq!(back, g);
        }
    }

    #[test]
    fn client_balance_to_dto_preserves_amounts() {
        let eur = Currency::new("EUR").unwrap();
        let balance = ClientBalance {
            client_id: ClientId::new(),
            client_name: "Acme".into(),
            total_invoiced: Money::new(10000, eur),
            total_paid: Money::new(3000, eur),
            outstanding: Money::new(7000, eur),
        };
        let dto: ClientBalanceDto = (&balance).into();
        assert_eq!(dto.outstanding.amount_minor, 7000);
        assert_eq!(dto.total_invoiced.amount_minor, 10000);
    }

    #[test]
    fn revenue_by_period_input_dto_maps() {
        let dto = RevenueByPeriodInputDto {
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            grouping: RevenueGroupingDto::Month,
        };
        let input: RevenueByPeriodInput = dto.into();
        assert_eq!(input.grouping, RevenueGrouping::Month);
    }
}
