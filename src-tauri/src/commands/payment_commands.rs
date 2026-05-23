use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    ListPaymentsQueryDto, NewPaymentDto, PaymentDto, UpdatePaymentDto,
};
use crate::application::AppError;
use crate::domain::payment::PaymentId;

use super::AppState;

#[tauri::command]
#[specta::specta]
pub fn payment_record(
    state: State<'_, AppState>,
    input: NewPaymentDto,
) -> Result<PaymentDto, AppError> {
    let domain = input.try_into()?;
    state.org()?
        .record_payment
        .execute(domain)
        .map(|p| (&p).into())
}

#[tauri::command]
#[specta::specta]
pub fn payment_update(
    state: State<'_, AppState>,
    input: UpdatePaymentDto,
) -> Result<PaymentDto, AppError> {
    let domain = input.try_into()?;
    state.org()?
        .update_payment
        .execute(domain)
        .map(|p| (&p).into())
}

#[tauri::command]
#[specta::specta]
pub fn payment_delete(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .delete_payment
        .execute(PaymentId(id))
}

#[tauri::command]
#[specta::specta]
pub fn payment_list(
    state: State<'_, AppState>,
    query: Option<ListPaymentsQueryDto>,
) -> Result<Vec<PaymentDto>, AppError> {
    state.org()?
        .list_payments
        .execute(query.unwrap_or_default().into())
        .map(|list| {
            list.iter()
                .map(|(p, name)| PaymentDto::from_payment_enriched(p, name.clone()))
                .collect()
        })
}

#[tauri::command]
#[specta::specta]
pub fn payment_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<PaymentDto, AppError> {
    state.org()?
        .get_payment
        .execute(PaymentId(id))
        .map(|(p, name)| PaymentDto::from_payment_enriched(&p, name))
}
