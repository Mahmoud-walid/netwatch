use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::database::{
    DatabaseManager, accounting::AccountingRepository, devices::DeviceRepository,
    users::UserRepository,
};
use crate::models::{Device, User};

pub struct ApiState {
    pub storage_path: PathBuf,
    pub live_tx: broadcast::Sender<String>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct DevicesResponse {
    pub devices: Vec<Device>,
}

#[derive(Serialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AssignUserRequest {
    pub user_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct UsageQuery {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize)]
pub struct UsageResponse {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub async fn start_api_server(
    storage_path: PathBuf,
    live_tx: broadcast::Sender<String>,
) -> Result<(), std::io::Error> {
    let state = Arc::new(ApiState {
        storage_path,
        live_tx,
    });

    let app = Router::new()
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/devices", get(get_devices))
        .route("/api/v1/devices/:id/user", post(assign_device_user))
        .route("/api/v1/users", get(get_users).post(create_user))
        .route("/api/v1/usage/device/:id", get(get_device_usage))
        .route("/api/v1/usage/user/:id", get(get_user_usage))
        .route("/api/v1/live", get(live_ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3030").await?;
    println!("API Server running at http://127.0.0.1:3030");

    axum::serve(listener, app).await
}

fn get_db(state: &ApiState) -> rusqlite::Connection {
    DatabaseManager::connect(&state.storage_path)
        .expect("Failed to connect to database for API request")
}

async fn get_status() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "online".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_devices(State(state): State<Arc<ApiState>>) -> Json<DevicesResponse> {
    let conn = get_db(&state);
    let devices = DeviceRepository::get_all(&conn).unwrap_or_default();
    Json(DevicesResponse { devices })
}

async fn get_users(State(state): State<Arc<ApiState>>) -> Json<UsersResponse> {
    let conn = get_db(&state);
    let users = UserRepository::get_all(&conn).unwrap_or_default();
    Json(UsersResponse { users })
}

async fn create_user(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<Option<User>> {
    let conn = get_db(&state);
    let user = UserRepository::create(&conn, &payload.name).ok();
    Json(user)
}

async fn assign_device_user(
    State(state): State<Arc<ApiState>>,
    Path(device_id): Path<i64>,
    Json(payload): Json<AssignUserRequest>,
) -> Json<bool> {
    let conn = get_db(&state);
    let success = DeviceRepository::assign_user(&conn, device_id, payload.user_id).is_ok();
    Json(success)
}

async fn get_device_usage(
    State(state): State<Arc<ApiState>>,
    Path(device_id): Path<i64>,
    Query(query): Query<UsageQuery>,
) -> Json<UsageResponse> {
    let conn = get_db(&state);
    let (rx_bytes, tx_bytes) = AccountingRepository::get_device_usage_between(
        &conn,
        device_id,
        &query.start_date,
        &query.end_date,
    )
    .unwrap_or((0, 0));

    Json(UsageResponse { rx_bytes, tx_bytes })
}

async fn get_user_usage(
    State(state): State<Arc<ApiState>>,
    Path(user_id): Path<i64>,
    Query(query): Query<UsageQuery>,
) -> Json<UsageResponse> {
    let conn = get_db(&state);
    let (rx_bytes, tx_bytes) = AccountingRepository::get_user_usage_between(
        &conn,
        user_id,
        &query.start_date,
        &query.end_date,
    )
    .unwrap_or((0, 0));

    Json(UsageResponse { rx_bytes, tx_bytes })
}

// WebSocket Handlers

async fn live_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ApiState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<ApiState>) {
    let mut rx = state.live_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
