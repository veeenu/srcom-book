use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use http::HeaderMap;
use srcom_book::db::DbConnection;
use srcom_book::srcom::SrcomAPI;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{AddExtensionLayer, Json, Router};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(
        rusqlite::Connection::open("srcom-book.sqlite")
            .map_err(anyhow::Error::from)
            .and_then(srcom_book::db::DbConnection::try_from)
            .unwrap(),
    ));
    let srcom_api = Arc::new(SrcomAPI::new());

    let app = Router::new()
        .route("/auth", get(authorize))
        .route("/pending", get(get_pending))
        .route("/games", get(get_games))
        .route("/book/:run", post(book_run).delete(unbook_run))
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(db))
                .layer(AddExtensionLayer::new(srcom_api))
                .layer(CorsLayer::permissive())
                .into_inner(),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3007));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn get_games() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::to_value(&*srcom_book::srcom::GAMES).unwrap()),
    )
}

#[axum_macros::debug_handler]
async fn get_pending(
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
    Extension(srcom_api): Extension<Arc<SrcomAPI>>,
) -> impl IntoResponse {
    let pending_web = match srcom_api.get_pending_runs().await {
        Ok(i) => i,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "err": format!("{:?}", e) })),
            )
        }
    };

    let mut db = db.lock().unwrap();
    let pending_db = match db.get_bookings() {
        Ok(i) => i,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "err": format!("{:?}", e) })),
            )
        }
    };

    let pending = srcom_book::merge_pendings(pending_db, pending_web);

    (StatusCode::OK, Json(serde_json::to_value(pending).unwrap()))
}

#[axum_macros::debug_handler]
async fn authorize(
    Extension(srcom_api): Extension<Arc<SrcomAPI>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = srcom_api.get_profile(&headers).await;
    match user {
        Ok(user) => (StatusCode::OK, user),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()),
    }
}

#[axum_macros::debug_handler]
async fn book_run(
    Path(run): Path<String>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
    Extension(srcom_api): Extension<Arc<SrcomAPI>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = match srcom_api.get_profile(&headers).await {
        Ok(user) => user,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
    };

    let r = match srcom_book::book_run(run, user, db.clone()).await
    {
        Ok(()) => String::new(),
        Err(e) => format!("{:?}", e),
    };
    (StatusCode::OK, r)
}

#[axum_macros::debug_handler]
async fn unbook_run(
    Path(run): Path<String>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
    Extension(srcom_api): Extension<Arc<SrcomAPI>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = match srcom_api.get_profile(&headers).await {
        Ok(user) => user,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
    };

    let r = match srcom_book::unbook_run(run, user, db.clone()).await {
        Ok(()) => String::new(),
        Err(e) => format!("{:?}", e),
    };
    (StatusCode::OK, r)
}
