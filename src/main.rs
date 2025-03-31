use axum::{
    routing::{get, post, delete, put},
    Router,
    Json,
    extract::{State, Path},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use dotenv::dotenv;

// Database model
#[derive(Debug, FromRow, Serialize)]
struct Prompt {
    id: Uuid,
    title: String,
    content: String,
    created_at: DateTime<Utc>,
}

// Request/response models
#[derive(Debug, Deserialize)]
struct CreatePrompt {
    title: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct UpdatePrompt {
    title: String,
    content: String,
}

// App state
#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db = PgPool::connect(&database_url).await?;
    
    // Create app state
    let state = AppState { db };

    // Build our application with routes
    let app = Router::new()
        .route("/prompts", post(create_prompt))
        .route("/prompts", get(list_prompts))
        .route("/prompts/:id", get(get_prompt))
        .route("/prompts/:id", put(update_prompt))
        .route("/prompts/:id", delete(delete_prompt))
        .with_state(state);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Handlers
async fn create_prompt(
    State(state): State<AppState>,
    Json(payload): Json<CreatePrompt>,
) -> Result<(StatusCode, Json<Prompt>), (StatusCode, String)> {
    let prompt = sqlx::query_as::<_, Prompt>(
        "INSERT INTO prompts (title, content) VALUES ($1, $2) RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(prompt)))
}

async fn list_prompts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Prompt>>, (StatusCode, String)> {
    let prompts = sqlx::query_as::<_, Prompt>("SELECT * FROM prompts")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(prompts))
}

async fn get_prompt(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Prompt>, (StatusCode, String)> {
    let prompt = sqlx::query_as::<_, Prompt>("SELECT * FROM prompts WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match prompt {
        Some(p) => Ok(Json(p)),
        None => Err((StatusCode::NOT_FOUND, "Prompt not found".to_string())),
    }
}

async fn update_prompt(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePrompt>,
) -> Result<Json<Prompt>, (StatusCode, String)> {
    let prompt = sqlx::query_as::<_, Prompt>(
        "UPDATE prompts SET title = $1, content = $2 WHERE id = $3 RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match prompt {
        Some(p) => Ok(Json(p)),
        None => Err((StatusCode::NOT_FOUND, "Prompt not found".to_string())),
    }
}

async fn delete_prompt(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM prompts WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Prompt not found".to_string()))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}