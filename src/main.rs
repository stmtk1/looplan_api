use std::net::SocketAddr;

use axum::{
    http::{ HeaderValue ,Method, StatusCode, header::CONTENT_TYPE },
    routing::{ get, post },
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/login", post(create_session))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "{\"hello\": \"world\" }"
}

async fn create_session(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateSession>,
) -> (StatusCode, Json<Session>) {
    // insert your application logic here
    let session = Session {
        token: String::from("hello world"),
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(session))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateSession {
    user_name: String,
    password: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct Session {
    token: String,
}
