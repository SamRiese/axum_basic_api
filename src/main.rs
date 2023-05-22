use axum::{routing::{get, post, put, delete}, Router, Json, Extension};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::{Executor, MySqlPool, query};
use axum::debug_handler;

#[tokio::main]
async fn main() {

    let pool = MySqlPool::connect("mysql://root:root@localhost/employees")
        .await
        .unwrap();

    let app = Router::new()
        .route("/create",post(create_employee))
        .route("/get", get(get_employee))
        .route("/update", put(update_employee))
        .route("/delete", delete(delete_employee))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0 , 0, 1], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn create_employee(State(pool): State<MySqlPool>,
                         Json(payload): Json<CreateEmployee>) -> (StatusCode, Json<Employee>) {

    let id = query!("INSERT INTO employee (`name`) VALUES (?);", payload.name)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id();

    (StatusCode::CREATED, Json(Employee{id: id as i32, name: payload.name}))
}

async fn get_employee(State(pool): State<MySqlPool>,
                      Json(payload): Json<GetEmployee>) -> (StatusCode, Json<Employee>) {

    let name = query!("SELECT employee.id, employee.name FROM employee WHERE employee.id = ?;", payload.id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .name;

    (StatusCode::OK, Json(Employee{id: payload.id, name}))
}

async fn update_employee(State(pool): State<MySqlPool>,
                      Json(payload): Json<UpdateEmployee>) -> (StatusCode, Json<Employee>) {

    query!("UPDATE employee SET name = ? WHERE employee.id = ?;", payload.name, payload.id)
        .execute(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(Employee{id: payload.id, name: payload.name}))
}

async fn delete_employee(State(pool): State<MySqlPool>,
                         Json(payload): Json<DeleteEmployee>) -> (StatusCode, Json<Employee>) {

    let mut transaction = pool.begin().await.unwrap();

    let name = query!("Select name FROM employee WHERE id = ?", payload.id)
        .fetch_one(&mut transaction)
        .await
        .unwrap()
        .name;

    query!("DELETE FROM employee WHERE id = ?;", payload.id)
        .execute(&mut transaction)
        .await
        .unwrap();

    transaction.commit().await.unwrap();
    (StatusCode::OK, Json(Employee{id: payload.id, name}))
}

#[derive(Serialize)]
struct Employee {
    id: i32,
    name: String
}

#[derive(Deserialize)]
struct CreateEmployee {
    name: String
}

#[derive(Deserialize)]
struct GetEmployee {
    id: i32
}

#[derive(Deserialize)]
struct UpdateEmployee {
    id: i32,
    name: String
}

#[derive(Deserialize)]
struct DeleteEmployee {
    id: i32
}