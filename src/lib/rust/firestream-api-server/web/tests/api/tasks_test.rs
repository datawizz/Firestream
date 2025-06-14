use axum::{
    body::Body,
    http::{self, Method},
};
use fake::{Fake, Faker};
use firestream_api_server_db::entities::tasks::{
    create as create_task, load as load_task, load_all as load_tasks, Task, TaskChangeset,
};
use firestream_api_server_db::test_helpers::users::{create as create_user, UserChangeset};
use firestream_api_server_macros::db_test;
use firestream_api_server_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use googletest::prelude::*;
use hyper::StatusCode;
use serde_json::json;
use uuid::Uuid;

type TasksList = Vec<Task>;

#[db_test]
async fn test_create_unauthorized(context: &DbTestContext) {
    let response = context
        .app
        .request("/tasks")
        .method(Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNAUTHORIZED));
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(TaskChangeset {
        description: String::from("")
    });

    let response = context
        .app
        .request("/tasks")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(task_changeset);

    let response = context
        .app
        .request("/tasks")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_that!(tasks, len(eq(1)));
    assert_that!(
        tasks.first().unwrap().description,
        eq(&task_changeset.description)
    );
}

#[db_test]
async fn test_create_batch_unauthorized(context: &DbTestContext) {
    let response = context
        .app
        .request("/tasks")
        .method(Method::PUT)
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNAUTHORIZED));
}

#[db_test]
async fn test_create_batch_invalid(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(vec![
        TaskChangeset {
            description: String::from("")
        },
        task_changeset
    ]);

    let response = context
        .app
        .request("/tasks")
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_that!(tasks, empty());
}

#[db_test]
async fn test_create_batch_success(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset1: TaskChangeset = Faker.fake();
    let task_changeset2: TaskChangeset = Faker.fake();
    let payload = json!(vec![task_changeset1.clone(), task_changeset2.clone()]);

    let response = context
        .app
        .request("/tasks")
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let tasks: Vec<Task> = response.into_body().into_json::<Vec<Task>>().await;
    assert_that!(
        tasks.first().unwrap().description,
        eq(&task_changeset1.description)
    );
    assert_that!(
        tasks.get(1).unwrap().description,
        eq(&task_changeset2.description)
    );

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_that!(tasks, len(eq(2)));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request("/tasks")
        .method(Method::GET)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let tasks: TasksList = response.into_body().into_json::<TasksList>().await;
    assert_that!(tasks, len(eq(1)));
    assert_that!(
        tasks.first().unwrap().description,
        eq(&task_changeset.description)
    );
}

#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(format!("/tasks/{}", Uuid::new_v4()).as_str())
        .method(Method::GET)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
    let task_id = task.id;

    let response = context
        .app
        .request(format!("/tasks/{}", task_id).as_str())
        .method(Method::GET)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let task: Task = response.into_body().into_json::<Task>().await;
    assert_that!(task.id, eq(task_id));
    assert_that!(task.description, eq(&task_changeset.description));
}

#[db_test]
async fn test_update_unauthorized(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(format!("/tasks/{}", task.id).as_str())
        .method(Method::PUT)
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNAUTHORIZED));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(TaskChangeset {
        description: String::from("")
    });

    let response = context
        .app
        .request(format!("/tasks/{}", task.id).as_str())
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let task_after = load_task(task.id, &context.db_pool).await.unwrap();
    assert_that!(task_after.description, eq(&task_changeset.description));
}

#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(task_changeset);

    let response = context
        .app
        .request(format!("/tasks/{}", Uuid::new_v4()).as_str())
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(task_changeset);

    let response = context
        .app
        .request(format!("/tasks/{}", task.id).as_str())
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    let task: Task = response.into_body().into_json::<Task>().await;
    assert_that!(task.description, eq(&task_changeset.description.clone()));

    let task = load_task(task.id, &context.db_pool).await.unwrap();
    assert_that!(task.description, eq(&task_changeset.description));
}

#[db_test]
async fn test_delete_unauthorized(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(format!("/tasks/{}", task.id).as_str())
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNAUTHORIZED));
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(format!("/tasks/{}", Uuid::new_v4()).as_str())
        .method(Method::DELETE)
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(format!("/tasks/{}", task.id).as_str())
        .method(Method::DELETE)
        .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = load_task(task.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
