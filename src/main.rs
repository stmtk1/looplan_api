
use axum::{
    extract::{ State, Query, },
    http::{ HeaderValue ,Method, StatusCode, header::{ CONTENT_TYPE, AUTHORIZATION, HeaderMap } },
    Json, Router,
    routing::{ get, post },
};
use mongodb::{options::ClientOptions, Client, bson::{oid::ObjectId, doc, DateTime }, Database};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/signup", post(create_user))
        .route("/signin", post(create_session))
        .route("/schedule", get(get_schedule))
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

async fn insert_user(db_pool: &Database, payload: CreateUser) {
    let password_hash = argon2::hash_encoded(payload.password.as_bytes(), b"salt_salt_salt", &argon2::Config::default()).unwrap();
    let user= User {
        id: None,
        name: payload.user_name.clone(),
        password_hash,
    };

    let user_collection = db_pool.collection::<User>("users");
    user_collection.insert_one(user, None).await.unwrap().inserted_id;
}

async fn insert_session(db_pool: &Database, dto: CreateSession) -> Session {
    let user_collection = db_pool.collection::<User>("users");
    let user_id = user_collection
        .find_one(Some(doc!{"name": dto.user_name.clone()}), None).await.unwrap().unwrap().id.unwrap();
    let session_collection = db_pool.collection::<Session>("sessions");
    let token = uuid::Uuid::new_v4().to_string();
    let session = Session {
        id: None,
        token: token.clone(),
        user_id,
    };
    session_collection.insert_one(session, None).await.unwrap();
    session_collection.find_one(Some(doc!["token": token.clone()]), None).await.unwrap().unwrap()
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(db_pool): State<Database>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<Session>) {
    // insert your application logic here
    insert_user(&db_pool, payload.clone()).await;
    let session = insert_session(&db_pool, CreateSession { user_name: payload.user_name.clone(), password: payload.password }).await;


    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(session))
}

// the input to our `create_user` handler
#[derive(Deserialize, Serialize, Clone)]
struct CreateUser {
    user_name: String,
    password: String,
}

#[derive(Deserialize)]
struct CreateSession {
    user_name: String,
    password: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Session {
    #[serde(rename="_id", skip_serializing)]
    id: Option<ObjectId>,
    user_id: ObjectId,
    token: String,
}

async fn create_session(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(db_pool): State<Database>,
    Json(payload): Json<CreateSession>,
) -> (StatusCode, Json<Session>) {
    // insert your application logic here
    let session = insert_session(&db_pool, payload).await;
    // with a status code of `201 Created`
    (StatusCode::ACCEPTED, Json(session))
}

async fn get_schedule(
    State(db_pool): State<Database>,
    map :HeaderMap<HeaderValue>,
    Query(getSchedules): Query<GetSchedules>
) -> (StatusCode, Json<Schedules>) {
    let token = map.get("authorization").unwrap().to_str().unwrap().get(7..).unwrap();
    let session_collection = db_pool.collection::<Session>("sessions");
    let session = session_collection.find_one(Some(doc!{ "token": token }), None).await.unwrap().unwrap();
    let schedule_collection = db_pool.collection::<Schedule>("schedule");
    let mut schedules_cursor = schedule_collection.find(Some(doc!{ "user_id": session.user_id }), None).await.unwrap();
    let mut schedules = vec![];
    while schedules_cursor.advance().await.unwrap() {
        schedules.push(schedules_cursor.deserialize_current().unwrap());
    }
    (StatusCode::OK, Json(Schedules{ schedules }))
}

#[derive(Deserialize)]
struct GetSchedules {
     #[serde(with = "bson::serde_helpers::bson_datetime_as_rfc3339_string")]
    start_time: DateTime,
     #[serde(with = "bson::serde_helpers::bson_datetime_as_rfc3339_string")]
    end_time: DateTime,
}

#[derive(Deserialize, Serialize)]
struct Schedules {
    schedules: Vec<Schedule>
}

#[derive(Deserialize, Serialize)]
struct Schedule {
    #[serde(rename="_id", skip_serializing)]
    id: Option<ObjectId>,
    user_id: ObjectId,
    start_time: DateTime,
    end_time: DateTime,
    name: String,
    description: String,
}