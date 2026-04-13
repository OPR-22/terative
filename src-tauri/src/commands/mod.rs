pub mod client_commands;
pub mod service_commands;
pub mod settings_commands;

use std::sync::Arc;

use crate::adapters::sqlite::{
    SqliteClientRepository, SqliteServiceRepository, SqliteSettingsRepository,
};
use crate::application::client_usecases::{
    CreateClient, DeleteClient, GetClientDetail, ListClients, UpdateClient,
};
use crate::application::service_usecases::{
    CreateService, DeleteService, ListServices, UpdateService,
};
use crate::application::settings_usecases::{
    GetSettings, UpdateAppPreferences, UpdateCurrency, UpdateSellerProfile,
};

pub struct AppState {
    pub create_client: CreateClient,
    pub update_client: UpdateClient,
    pub delete_client: DeleteClient,
    pub list_clients: ListClients,
    pub get_client_detail: GetClientDetail,

    pub create_service: CreateService,
    pub update_service: UpdateService,
    pub delete_service: DeleteService,
    pub list_services: ListServices,

    pub get_settings: GetSettings,
    pub update_seller_profile: UpdateSellerProfile,
    pub update_currency: UpdateCurrency,
    pub update_app_preferences: UpdateAppPreferences,
}

impl AppState {
    pub fn new(db: crate::adapters::sqlite::Db) -> Self {
        let client_repo = Arc::new(SqliteClientRepository::new(db.clone()));
        let service_repo = Arc::new(SqliteServiceRepository::new(db.clone()));
        let settings_repo = Arc::new(SqliteSettingsRepository::new(db));

        Self {
            create_client: CreateClient::new(client_repo.clone()),
            update_client: UpdateClient::new(client_repo.clone()),
            delete_client: DeleteClient::new(client_repo.clone()),
            list_clients: ListClients::new(client_repo.clone()),
            get_client_detail: GetClientDetail::new(client_repo),

            create_service: CreateService::new(service_repo.clone()),
            update_service: UpdateService::new(service_repo.clone()),
            delete_service: DeleteService::new(service_repo.clone()),
            list_services: ListServices::new(service_repo),

            get_settings: GetSettings::new(settings_repo.clone()),
            update_seller_profile: UpdateSellerProfile::new(settings_repo.clone()),
            update_currency: UpdateCurrency::new(settings_repo.clone()),
            update_app_preferences: UpdateAppPreferences::new(settings_repo),
        }
    }
}

pub fn to_ipc_err(e: crate::application::AppError) -> String {
    e.to_string()
}
