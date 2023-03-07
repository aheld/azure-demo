use std::sync::Mutex;

use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path, Query, ServiceConfig},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

use crate::{LogApiKey, RequireApiKey};

#[derive(Default)]
pub(super) struct ConfigStore {
    configs: Mutex<Vec<Config>>,
}

pub(super) fn configure(store: Data<ConfigStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            .service(search_configs)
            .service(get_configs)
            .service(create_config)
            .service(delete_config)
            .service(get_config_by_id)
            .service(update_config);
    }
}

/// Task to do.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub(super) struct Config {
    /// Unique id for the config item.
    #[schema(example = 1)]
    id: i32,
    /// Description of the config
    #[schema(example = "Chain name")]
    desc: String,
    /// Key of the config
    #[schema(example = "chain_name")]
    key: String,
    /// Value of the config
    #[schema(example = "Top Shelf")]
    value: String,
}

/// Request to update existing `Config` item.`
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub(super) struct ConfigUpdateRequest {
    /// Optional new value for the `Config` task.
    #[schema(example = "Loyalty Key")]
    value: Option<String>,
    /// Optional check status to mark the config as a secret
    secret: Option<bool>,
}

/// config endpoint error responses
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) enum ErrorResponse {
    /// When Config is not found by search term.
    NotFound(String),
    /// When there is a conflict storing a new config.
    Conflict(String),
    /// When config endpoint was called without correct credentials
    Unauthorized(String),
}

/// Get list of Configs.
///
/// List configs from in-memory config store.
///
/// One could call the api endpoint with following curl.
/// ```text
/// curl localhost:8080/config
/// ```
#[utoipa::path(
    responses(
        (status = 200, description = "List current config items", body = [Config])
    )
)]
#[get("/config")]
pub(super) async fn get_configs(config_store: Data<ConfigStore>) -> impl Responder {
    let configs = config_store.configs.lock().unwrap();

    HttpResponse::Ok().json(configs.clone())
}

/// Create new config to shared in-memory storage.
///
/// Post a new `config` in request body as json to store it. Api will return
/// created `config` on success or `ErrorResponse::Conflict` if config with same id already exists.
///
/// One could call the api with.
/// ```text
/// curl localhost:8080/config -d '{"id": 1, "desc": "chain name", "key": "chain", "value": "top shelf"}'
/// ```
#[utoipa::path(
    request_body = Config,
    responses(
        (status = 201, description = "Config created successfully", body = Config),
        (status = 409, description = "Config with id already exists", body = ErrorResponse, example = json!(ErrorResponse::Conflict(String::from("id = 1"))))
    )
)]
#[post("/config")]
pub(super) async fn create_config(config: Json<Config>, config_store: Data<ConfigStore>) -> impl Responder {
    let mut configs = config_store.configs.lock().unwrap();
    let config = &config.into_inner();

    configs
        .iter()
        .find(|existing| existing.id == config.id)
        .map(|existing| {
            HttpResponse::Conflict().json(ErrorResponse::Conflict(format!("id = {}", existing.id)))
        })
        .unwrap_or_else(|| {
            configs.push(config.clone());

            HttpResponse::Ok().json(config)
        })
}

/// Delete config by given path variable id.
///
/// This endpoint needs `api_key` authentication in order to call. Api key can be found from README.md.
///
/// Api will delete config from shared in-memory storage by the provided id and return success 200.
/// If storage does not contain `config` with given id 404 not found will be returned.
#[utoipa::path(
    responses(
        (status = 200, description = "Config deleted successfully"),
        (status = 401, description = "Unauthorized to delete Config", body = ErrorResponse, example = json!(ErrorResponse::Unauthorized(String::from("missing api key")))),
        (status = 404, description = "Config not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Config")
    ),
    security(
        ("api_key" = [])
    )
)]
#[delete("/config/{id}", wrap = "RequireApiKey")]
pub(super) async fn delete_config(id: Path<i32>, config_store: Data<ConfigStore>) -> impl Responder {
    let mut configs = config_store.configs.lock().unwrap();
    let id = id.into_inner();

    let new_configs = configs
        .iter()
        .filter(|config| config.id != id)
        .cloned()
        .collect::<Vec<_>>();

    if new_configs.len() == configs.len() {
        HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
    } else {
        *configs = new_configs;
        HttpResponse::Ok().finish()
    }
}

/// Get by given id.
///
/// Return found `Config` with status 200 or 404 not found if `config` is not found from shared in-memory storage.
#[utoipa::path(
    responses(
        (status = 200, description = "Config found from storage", body = Config),
        (status = 404, description = "Config not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Config")
    )
)]
#[get("/config/{id}")]
pub(super) async fn get_config_by_id(id: Path<i32>, config_store: Data<ConfigStore>) -> impl Responder {
    let configs = config_store.configs.lock().unwrap();
    let id = id.into_inner();

    configs
        .iter()
        .find(|config| config.id == id)
        .map(|config| HttpResponse::Ok().json(config))
        .unwrap_or_else(|| {
            HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
        })
}

/// Update config with given id.
///
/// This endpoint supports optional authentication.
///
/// Tries to update `config` by given id as path variable. If config is found by id values are
/// updated according `configUpdateRequest` and updated `config` is returned with status 200.
/// If config is not found then 404 not found is returned.
#[utoipa::path(
    request_body = ConfigUpdateRequest,
    responses(
        (status = 200, description = "Config updated successfully", body = config),
        (status = 404, description = "Config not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Config")
    ),
    security(
        (),
        ("api_key" = [])
    )
)]
#[put("/config/{id}", wrap = "LogApiKey")]
pub(super) async fn update_config(
    id: Path<i32>,
    config: Json<ConfigUpdateRequest>,
    config_store: Data<ConfigStore>,
) -> impl Responder {
    let mut configs = config_store.configs.lock().unwrap();
    let id = id.into_inner();
    let config = config.into_inner();

    configs
        .iter_mut()
        .find_map(|c| if c.id == id { Some(c) } else { None })
        .map(|existing| {
            if let Some(value) = config.value {
                existing.value = value;
            }

            HttpResponse::Ok().json(existing)
        })
        .unwrap_or_else(|| {
            HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
        })
}

/// Search configs Query
#[derive(Deserialize, Debug, IntoParams)]
pub(super) struct SearchConfigs {
    /// Content that should be found from config's value field
    value: String,
}

/// Search configs with by value
///
/// Perform search from `config`s present in in-memory storage by matching config's value to
/// value provided as query parameter. Returns 200 and matching `config` items.
#[utoipa::path(
    params(
        SearchConfigs
    ),
    responses(
        (status = 200, description = "Search did not result error", body = [Config]),
    )
)]
#[get("/config/search")]
pub(super) async fn search_configs(
    query: Query<SearchConfigs>,
    configs_store: Data<ConfigStore>,
) -> impl Responder {
    let configs = configs_store.configs.lock().unwrap();

    HttpResponse::Ok().json(
        configs
            .iter()
            .filter(|config| {
                config.value
                    .to_lowercase()
                    .contains(&query.value.to_lowercase())
            })
            .cloned()
            .collect::<Vec<_>>(),
    )
}