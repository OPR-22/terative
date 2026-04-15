use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    ListPaymentsQueryDto, NewPaymentDto, PaymentDto, UpdatePaymentDto,
};
use crate::application::AppError;
use crate::domain::payment::PaymentId;

use super::{to_ipc_err, AppState};

fn dto_err(e: crate::application::dto::DtoConvertError) -> String {
    to_ipc_err(AppError::from(e))
}

#[tauri::command]
#[specta::specta]
pub fn payment_record(
    state: State<'_, AppState>,
    input: NewPaymentDto,
) -> Result<PaymentDto, String> {
    let domain = input.try_into().map_err(dto_err)?;
    state
        .record_payment
        .execute(domain)
        .map(|p| (&p).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn payment_update(
    state: State<'_, AppState>,
    input: UpdatePaymentDto,
) -> Result<PaymentDto, String> {
    let domain = input.try_into().map_err(dto_err)?;
    state
        .update_payment
        .execute(domain)
        .map(|p| (&p).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn payment_delete(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .delete_payment
        .execute(PaymentId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn payment_list(
    state: State<'_, AppState>,
    query: Option<ListPaymentsQueryDto>,
) -> Result<Vec<PaymentDto>, String> {
    state
        .list_payments
        .execute(query.unwrap_or_default().into())
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn payment_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<PaymentDto, String> {
    state
        .get_payment
        .execute(PaymentId(id))
        .map(|p| (&p).into())
        .map_err(to_ipc_err)
}
