use std::net::SocketAddr;

use axum::{
    http::{ HeaderValue ,Method, StatusCode, header::CONTENT_TYPE },
    routing::{ get, post },
    Json, Router, extract::State,
};
use mongodb::{options::ClientOptions, Client, bson::oid::ObjectId, Database};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    #[serde(rename="_id", skip_serializing)]
    id: Option<ObjectId>,
    name: String,
    password_hash: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let db_client_options = ClientOptions::parse("mongodb://0.0.0.0:27017").await.unwrap();
    let db_client = Client::with_options(db_client_options).unwrap();

    let db = db_client.database("looplan");
    // let collection = db.collection::<User>("users");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/signup", post(create_user))
        .with_state(db)
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "{\"hello\": \"world\" }"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(db_pool): State<Database>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let password_hash = argon2::hash_encoded(payload.password.as_bytes(), b"salt_salt_salt", &argon2::Config::default()).unwrap();
    println!("{}", password_hash);
    let user= User {
        id: None,
        name: payload.user_name,
        password_hash,
    };

    let collection = db_pool.collection::<User>("users");
    collection.insert_one(user.clone(), None).await.unwrap();

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    user_name: String,
    password: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct Session {
    token: String,
}
