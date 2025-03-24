use super::AppError;
use crate::{
    db::{self},
    food::{EditFood, Food},
    server::AppState,
};
use askama::Template;
use axum::{
    Json,
    extract::{Path, State},
    response::{Html, IntoResponse},
};

pub async fn get_all_foods_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let foods = db::food::get_all_foods(conn).await?;

    Ok(serde_json::to_string(&foods)?)
}

pub async fn get_food_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;

    let food = db::food::get_food(conn, id).await?;

    Ok(serde_json::to_string(&food)?)
}

pub async fn update_food_handler(
    State(state): State<AppState>,
    Json(food): Json<Food>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let id = db::food::update_food(conn, &food).await?;

    let result = serde_json::json!({ "id": id, "result": "success" });
    Ok(serde_json::to_string(&result)?)
}

pub async fn create_food_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let existing_ingredients = db::food::get_all_ingredient_names(conn).await?;
    let conn = state.pool.get().await?;
    let existing_tags = db::food::get_all_existing_tags(conn).await?;

    let edit_food_template = EditFood::create(existing_ingredients, existing_tags);
    Ok(Html(edit_food_template.render()?))
}

pub async fn delete_food_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    db::food::delete_food(conn, id).await?;

    let response = serde_json::json!({ "id": id, "result": "success" });
    Ok(Json(response))
}

pub async fn edit_food_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let food = db::food::get_food(conn, id).await?;
    let conn = state.pool.get().await?;
    let existing_ingredients = db::food::get_all_ingredient_names(conn).await?;
    let conn = state.pool.get().await?;
    let existing_tags = db::food::get_all_existing_tags(conn).await?;

    let edit_food_template = EditFood::edit_food(food, existing_ingredients, existing_tags);

    Ok(Html(edit_food_template.render()?))
    // Ok(Json(food))
}

pub async fn food_list_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let foods = db::food::get_all_foods(conn).await?;

    #[derive(Template)]
    #[template(path = "food/list.html")]
    struct FoodList {
        foods: Vec<Food>,
    }

    let food_list = FoodList { foods };

    Ok(Html(food_list.render()?))
}
