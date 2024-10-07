use axum::body::Body;
use axum::{http::header, response::Response};
use axum::{http::StatusCode, routing::get, Router};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(index))
        .route("/hi", get(|| async { "Hello, world!" }))
        .layer(tower_http::cors::CorsLayer::permissive());
    let addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        4044,
    );
    info!("Listening on http://{addr}");
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
async fn index() -> Response {
    info!("Request received");
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
        )
        .body(Body::from(include_bytes!("../index.html").to_vec()))
        .unwrap()
}