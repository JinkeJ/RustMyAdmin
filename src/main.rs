#![recursion_limit = "512"]

mod api_console;
mod api_data;
mod api_meta;
mod api_users;
mod error;
mod exec;
mod session;
mod sqlutil;

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use session::AppState;

const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_JS: &str = include_str!("../static/app.js");
const STYLE_CSS: &str = include_str!("../static/style.css");

async fn serve_js() -> Response {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
        .into_response()
}

async fn serve_css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], STYLE_CSS).into_response()
}

fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // session
        .route("/login", post(api_meta::login))
        .route("/logout", post(api_meta::logout))
        .route("/status", get(api_meta::status))
        // databases
        .route(
            "/databases",
            get(api_meta::databases).post(api_meta::create_database),
        )
        .route("/databases/{db}/drop", post(api_meta::drop_database))
        .route("/databases/{db}/rename", post(api_meta::rename_database))
        .route(
            "/databases/{db}/tables",
            get(api_meta::tables_overview).post(api_data::create_table),
        )
        // tables
        .route(
            "/databases/{db}/tables/{table}/structure",
            get(api_meta::table_structure),
        )
        .route("/databases/{db}/tables/{table}/rows", get(api_data::browse_rows))
        .route(
            "/databases/{db}/tables/{table}/rows/delete",
            post(api_data::delete_rows),
        )
        .route(
            "/databases/{db}/tables/{table}/rows/update",
            post(api_data::update_row),
        )
        .route(
            "/databases/{db}/tables/{table}/rows/insert",
            post(api_data::insert_row),
        )
        .route(
            "/databases/{db}/tables/{table}/truncate",
            post(api_data::truncate_table),
        )
        .route("/databases/{db}/tables/{table}/drop", post(api_data::drop_table))
        .route(
            "/databases/{db}/tables/{table}/rename",
            post(api_data::rename_table),
        )
        .route(
            "/databases/{db}/tables/{table}/columns/{col}/rename",
            post(api_data::rename_column),
        )
        .route(
            "/databases/{db}/tables/{table}/columns/{col}/drop",
            post(api_data::drop_column),
        )
        .route(
            "/databases/{db}/tables/{table}/columns/{col}/modify",
            post(api_data::modify_column),
        )
        .route(
            "/databases/{db}/tables/{table}/columns/{col}/move",
            post(api_data::move_column),
        )
        // foreign keys
        .route(
            "/databases/{db}/tables/{table}/foreign-keys",
            get(api_data::list_foreign_keys),
        )
        .route(
            "/databases/{db}/tables/{table}/foreign-keys/create",
            post(api_data::create_foreign_key),
        )
        .route(
            "/databases/{db}/tables/{table}/foreign-keys/{name}/drop",
            post(api_data::drop_foreign_key),
        )
        .route(
            "/databases/{db}/tables/{table}/columns/add",
            post(api_data::add_column),
        )
        // indexes
        .route(
            "/databases/{db}/tables/{table}/indexes",
            get(api_data::list_indexes),
        )
        .route(
            "/databases/{db}/tables/{table}/indexes/create",
            post(api_data::create_index),
        )
        .route(
            "/databases/{db}/tables/{table}/indexes/{name}/drop",
            post(api_data::drop_index),
        )
        // SQL console / import & export
        .route("/sql", post(api_console::exec_sql))
        .route("/export", get(api_console::export))
        .route("/import", post(api_console::import))
        // users & privileges
        .route("/users", get(api_users::list_users))
        .route("/users/create", post(api_users::create_user))
        .route("/users/drop", post(api_users::drop_user))
        .route("/users/rename", post(api_users::rename_user))
        .route("/users/password", post(api_users::change_password))
        .route("/users/grants", get(api_users::show_grants))
        .route("/users/grants/apply", post(api_users::apply_grants))
        .route("/flush-privileges", post(api_users::flush_privileges))
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    // periodically clean up idle sessions
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
                st.cleanup().await;
            }
        });
    }

    let addr = std::env::var("RMA_BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let app = Router::new()
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .route("/app.js", get(serve_js))
        .route("/style.css", get(serve_css))
        .nest("/api", api_routes().with_state(state.clone()))
        .fallback(|| async { (StatusCode::NOT_FOUND, "404 Not Found") })
        .layer(DefaultBodyLimit::max(256 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("端口绑定失败");
    println!("RustMyAdmin 已启动: http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
