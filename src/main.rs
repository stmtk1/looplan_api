
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
    #[serde(rename="_id")]
    id: ObjectId,
    name: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InsertUser {
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
        .route("/schedule", post(create_schedule))
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
    let user= InsertUser {
        name: payload.user_name.clone(),
        password_hash,
    };

    let user_collection = db_pool.collection::<InsertUser>("users");
    user_collection.insert_one(user, None).await.unwrap();
}

async fn insert_session(db_pool: &Database, dto: CreateSession) -> Session {
    let user_collection = db_pool.collection::<User>("users");
    let user_id = user_collection
        .find_one(Some(doc!{"name": dto.user_name.clone()}), None).await.unwrap().unwrap().id;
    let session_collection = db_pool.collection::<InsertSession>("sessions");
    let token = uuid::Uuid::new_v4();
    let session = InsertSession {
        token,
        user_id,
    };
    session_collection.insert_one(session, None).await.unwrap();
    let session_collection = db_pool.collection::<Session>("sessions");
    session_collection.find_one(Some(doc!["token": token]), None).await.unwrap().unwrap()
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
    #[allow(dead_code)]
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InsertSession {
    user_id: ObjectId,
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    token: uuid::Uuid,
}

// the output to our `create_user` handler
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Session {
    #[serde(rename="_id")]
    #[allow(dead_code)]
    id: ObjectId,
    user_id: ObjectId,
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    token: uuid::Uuid,
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
async fn validate_session(map: &HeaderMap, db_pool: &Database) -> Session {
    let token = uuid::Uuid::parse_str(map.get("authorization").unwrap().to_str().unwrap().get(7..).unwrap()).unwrap();
    let session_collection = db_pool.collection::<Session>("sessions");
    session_collection.find_one(Some(doc!{ "token": token }), None).await.unwrap().unwrap()
}
async fn get_schedule(
    State(db_pool): State<Database>,
    map :HeaderMap<HeaderValue>,
    Query(get_schedules): Query<GetSchedules>
) -> (StatusCode, Json<Schedules>) {
    let session = validate_session(&map, &db_pool).await;
    let schedule_collection = db_pool.collection::<DbSchedule>("schedule");
    let mut schedules_cursor = schedule_collection.find(Some(doc!{
        "user_id": session.user_id,
        "start_time": { "$gte": get_schedules.start_time, "$lte": get_schedules.end_time, },
    }), None).await.unwrap();
    let mut schedules = vec![];
    while schedules_cursor.advance().await.unwrap() {
        schedules.push(schedules_cursor.deserialize_current().unwrap().to_schedule());
    }
    (StatusCode::OK, Json(Schedules{ schedules }))
}

async fn create_schedule(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(db_pool): State<Database>,
    map :HeaderMap<HeaderValue>,
    Json(payload): Json<CreateSchedule>,
) -> (StatusCode, Json<InsertSchedule>) {
    let session = validate_session(&map, &db_pool).await;
    let schedule_collection = db_pool.collection::<InsertSchedule>("schedule");
    let schedule = InsertSchedule {
        name: payload.name.clone(),
        description: payload.description.clone(),
        start_time: payload.start_time,
        end_time: payload.end_time,
        user_id: session.user_id,
    };
    schedule_collection.insert_one(schedule.clone(), None).await.unwrap();
    (StatusCode::ACCEPTED, Json(schedule))
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

#[derive(Deserialize, Serialize, Clone)]
struct DbSchedule {
    #[serde(rename="_id")]
    id: ObjectId,
    user_id: ObjectId,
    start_time: DateTime,
    end_time: DateTime,
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct Schedule {
    #[serde(rename="_id")]
    id: ObjectId,
    user_id: ObjectId,
    start_time: String,
    end_time: String,
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct InsertSchedule {
    user_id: ObjectId,
    start_time: DateTime,
    end_time: DateTime,
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize)]
struct CreateSchedule {
    #[serde(with = "bson::serde_helpers::bson_datetime_as_rfc3339_string")]
    start_time: DateTime,
    #[serde(with = "bson::serde_helpers::bson_datetime_as_rfc3339_string")]
    end_time: DateTime,
    name: String,
    description: String,
}

impl DbSchedule {
    fn to_schedule(&self) -> Schedule {
        Schedule {
            name: self.name.clone(),
            description: self.description.clone(),
            id: self.id,
            user_id: self.user_id,
            start_time: self.start_time.try_to_rfc3339_string().unwrap(),
            end_time: self.end_time.try_to_rfc3339_string().unwrap(),
        }
    }
}