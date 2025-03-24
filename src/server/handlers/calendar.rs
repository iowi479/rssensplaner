use super::AppError;
use crate::{
    calendar::{Day, ResponseDay},
    db,
    server::AppState,
};
use askama::Template;
use axum::{
    Json,
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::{Days, NaiveDate};

/// This handler returns the html for the calendar.
pub async fn get_calendar_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let today = chrono::Local::now().date_naive();

    let current_week = today.week(chrono::Weekday::Mon);
    let first = current_week.first_day();
    let last = current_week.last_day();
    let from = first.checked_sub_days(Days::new(7)).unwrap();
    let to = last.checked_add_days(Days::new(8)).unwrap();

    let conn = state.pool.get().await?;
    let days = db::calendar::get_days(conn, from, to).await?;

    #[derive(Debug, Template)]
    #[template(path = "calendar/index.html")]
    struct Calendar<'a> {
        last_week: &'a [Day],
        current_week: &'a [Day],
        next_week: &'a [Day],
        current_date: NaiveDate,
    }

    let calendar = Calendar {
        last_week: &days[0..7],
        current_week: &days[7..14],
        next_week: &days[14..21],
        current_date: today,
    };

    Ok(Html(calendar.render()?))
}

/// This will update the day in the internal database.
pub async fn update_day_handler(
    State(state): State<AppState>,
    Json(day): Json<ResponseDay>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    db::calendar::update_day(conn, day).await?;

    Ok(Json(serde_json::json!({ "result": "success" })))
}

/// This handler returns the day specified by the given ID.
pub async fn get_day_handler(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let today = chrono::Local::now().date_naive();
    let days = db::calendar::get_days(conn, today, today.succ_opt().unwrap()).await?;

    Ok(Json(days))
}
