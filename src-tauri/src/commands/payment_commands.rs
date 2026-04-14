use tauri::State;

use crate::application::payment_usecases::UpdatePaymentInput;
use crate::application::ports::ListPaymentsQuery;
use crate::domain::payment::{NewPayment, Payment, PaymentId};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn payment_record(
    state: State<'_, AppState>,
    input: NewPayment,
) -> Result<Payment, String> {
    state.record_payment.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn payment_update(
    state: State<'_, AppState>,
    input: UpdatePaymentInput,
) -> Result<Payment, String> {
    state.update_payment.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn payment_delete(
    state: State<'_, AppState>,
    id: PaymentId,
) -> Result<(), String> {
    state.delete_payment.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn payment_list(
    state: State<'_, AppState>,
    query: Option<ListPaymentsQuery>,
) -> Result<Vec<Payment>, String> {
    state
        .list_payments
        .execute(query.unwrap_or_default())
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn payment_get(
    state: State<'_, AppState>,
    id: PaymentId,
) -> Result<Payment, String> {
    state.get_payment.execute(id).map_err(to_ipc_err)
}
