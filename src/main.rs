use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use srcom_book::auth::Auth;
use srcom_book::db::DbConnection;

use axum::extract::{Extension, Path, TypedHeader};
use axum::headers::{authorization::Basic, Authorization};
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

    let auth = Auth::new(db.clone());

    let app = Router::new()
        .route("/auth", get(authorize))
        .route("/pending", get(get_pending))
        .route("/mods", get(get_mods))
        .route("/book/:run", post(book_run).delete(unbook_run))
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(db))
                .layer(AddExtensionLayer::new(auth))
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
async fn get_mods() -> impl IntoResponse {
    let mods = srcom_book::srcom::get_mods()
        .await
        .map_err(|e| format!("{:?}", e));
    (
        StatusCode::OK,
        Json(match mods {
            Ok(mods) => serde_json::to_value(mods).unwrap(),
            Err(e) => serde_json::json!({ "err": format!("{:?}", e) }),
        }),
    )
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
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Extension(auth): Extension<Auth>,
) -> impl IntoResponse {
    if let Err(e) = auth.check(authorization.username(), authorization.password()) {
        return (
            StatusCode::BAD_REQUEST,
            format!("Invalid credentials: {}", e),
        );
    } else {
        return (StatusCode::OK, format!(""));
    }
}

#[axum_macros::debug_handler]
async fn book_run(
    Path(run): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Extension(auth): Extension<Auth>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
) -> impl IntoResponse {
    if let Err(e) = auth.check(authorization.username(), authorization.password()) {
        return (
            StatusCode::BAD_REQUEST,
            format!("Invalid credentials: {}", e),
        );
    }

    let r = match srcom_book::book_run(run, authorization.username().to_string(), db.clone()).await
    {
        Ok(()) => format!(""),
        Err(e) => format!("{:?}", e),
    };
    (StatusCode::OK, r)
}

#[axum_macros::debug_handler]
async fn unbook_run(
    Path(run): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Extension(auth): Extension<Auth>,
    Extension(db): Extension<Arc<Mutex<DbConnection>>>,
) -> impl IntoResponse {
    if let Err(e) = auth.check(authorization.username(), authorization.password()) {
        return (
            StatusCode::BAD_REQUEST,
            format!("Invalid credentials: {}", e),
        );
    }

    let r = match srcom_book::book_run(run, "nobody".to_string(), db.clone()).await {
        Ok(()) => format!(""),
        Err(e) => format!("{:?}", e),
    };
    (StatusCode::OK, r)
}
