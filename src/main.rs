use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{AddExtensionLayer, Json, Router};
use http::Method;
use srcom_book::db::DbConnection;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, any};

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(
        rusqlite::Connection::open("srcom-book.sqlite")
            .map_err(anyhow::Error::from)
            .and_then(srcom_book::db::DbConnection::try_from)
            .unwrap(),
    ));

    let app = Router::new()
        .route("/pending", get(get_pending))
        .route("/deleted", get(get_deleted))
        .route("/fetch", get(fetch_runs))
        .route("/mods", get(get_mods))
        .route("/book/:run/:user", post(book_run))
        .route("/run/:run", post(enable_run).delete(disable_run))
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(db))
                .layer(
                    CorsLayer::new()
                        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
                        .allow_origin(any()),
                )
                .into_inner(),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3007));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn get_mods() -> impl IntoResponse {
    (StatusCode::OK, Json(srcom_book::srcom::get_mods().await.map_err(|e| format!("{:?}", e))))
}

#[axum_macros::debug_handler]
async fn get_pending(Extension(db): Extension<Arc<Mutex<DbConnection>>>) -> impl IntoResponse {
    let pending_web = match srcom_book::srcom::get_pending_runs().await {
        Ok(i) => i,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "err": format!("{:?}", e) })),
            )
        }
    };

    let mut db = db.lock().unwrap();
    let pending_db = match db.get_pending_runs() {
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
async fn get_deleted(Extension(db): Extension<Arc<Mutex<DbConnection>>>) -> impl IntoResponse {
    let mut db = db.lock().unwrap();
    (
        StatusCode::OK,
        Json(db.get_deleted_runs().map_err(|e| format!("{:?}", e))),
    )
}

#[axum_macros::debug_handler]
async fn fetch_runs(Extension(db): Extension<Arc<Mutex<DbConnection>>>) -> impl IntoResponse {
    let r = match srcom_book::fetch_and_store(db.clone()).await {
        Ok(()) => format!(""),
        Err(e) => format!("{:?}", e),
    };
    (StatusCode::OK, r)
}

#[axum_macros::debug_handler]
async fn book_run(
    Path((run, user)): Path<(String, String)>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
) -> impl IntoResponse {
    let r = match srcom_book::book_run(run, user, db.clone()).await {
        Ok(()) => format!(""),
        Err(e) => format!("{:?}", e),
    };
    (StatusCode::OK, r)
}

#[axum_macros::debug_handler]
async fn enable_run(
    Path(run): Path<String>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
) -> impl IntoResponse {
    match db.lock().unwrap().undelete_run(&run) {
        Ok(()) => (StatusCode::OK, format!("")),
        Err(e) => (StatusCode::OK, format!("{:?}", e)),
    }
}

#[axum_macros::debug_handler]
async fn disable_run(
    Path(run): Path<String>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
) -> impl IntoResponse {
    match db.lock().unwrap().delete_run(&run) {
        Ok(()) => (StatusCode::OK, format!("")),
        Err(e) => (StatusCode::OK, format!("{:?}", e)),
    }
}
