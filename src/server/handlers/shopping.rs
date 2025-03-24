use std::collections::HashMap;

use super::AppError;
use crate::{
    db::{self},
    server::AppState,
    shopping::{EditItem, Item},
};
use askama::Template;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
};
use chrono::{Datelike, Days, NaiveDate, NaiveWeek};
use serde::Deserialize;

pub async fn update_item_handler(
    State(state): State<AppState>,
    Json(items): Json<Vec<EditItem>>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    db::shopping::update_items(conn, &items).await?;

    let result = serde_json::json!({ "result": "success" });
    Ok(serde_json::to_string(&result)?)
}

pub async fn delete_item_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    db::shopping::delete_item(conn, id).await?;

    let response = serde_json::json!({ "id": id, "result": "success" });
    Ok(Json(response))
}

pub async fn get_all_items_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let items = db::shopping::get_items(conn).await?;

    Ok(serde_json::to_string(&items)?)
}

pub async fn default_item_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get().await?;
    let items = db::shopping::get_items(conn).await?;
    let conn = state.pool.get().await?;
    let existing_items = db::food::get_all_ingredient_names(conn).await?;

    let items = items.into_iter().map(EditItem::from).collect();

    #[derive(Template)]
    #[template(path = "shopping/edit_default_list.html")]
    struct EditDefaultItems {
        items: Vec<EditItem>,
        existing_items: Vec<String>,
    }

    let edit_default_items = EditDefaultItems {
        items,
        existing_items,
    };

    Ok(Html(edit_default_items.render()?))
}

pub async fn get_shopping_index_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Template)]
    #[template(path = "shopping/index.html")]
    struct ShoppingMain {
        current_week: Vec<String>,
        next_week: Vec<String>,
        default_date: String,
    }

    let today = chrono::Local::now().date_naive();
    let next_saturday = get_next_saturday(today);
    let current_week = today.week(chrono::Weekday::Mon);
    let next_week = current_week
        .last_day()
        .succ_opt()
        .expect("Could not get next week")
        .week(chrono::Weekday::Mon);

    let shopping_main = ShoppingMain {
        current_week: week_to_day_strings(current_week),
        next_week: week_to_day_strings(next_week),
        default_date: next_saturday.format("%Y-%m-%d").to_string(),
    };

    Ok(Html(shopping_main.render()?))
}

/// If it is Saturday, return the next Saturday.
/// Otherwise, return the Saturday within the next 6 days.
fn get_next_saturday(today: NaiveDate) -> NaiveDate {
    let todays_weekday = today.weekday();

    match todays_weekday {
        chrono::Weekday::Sat => today.checked_add_days(Days::new(7)).unwrap(),
        _ => {
            let this_week = today.week(chrono::Weekday::Sun);
            this_week.last_day()
        }
    }
}

/// Get the dates of the week as strings.
fn week_to_day_strings(week: NaiveWeek) -> Vec<String> {
    let mut day = week.first_day();
    let last_day = week.last_day();

    let mut days = Vec::new();

    while day <= last_day {
        days.push(day.format("%Y-%m-%d").to_string());
        day = day.succ_opt().unwrap();
    }

    days
}

#[derive(Deserialize)]
pub struct ShoppingListParams {
    default: Option<bool>,
    date: Option<String>,
}

pub async fn shopping_list_handler(
    State(state): State<AppState>,
    Query(params): Query<ShoppingListParams>,
) -> Result<impl IntoResponse, AppError> {
    let default = params
        .default
        .ok_or_else(|| anyhow::anyhow!("no default provided"))?;
    let date_str = params
        .date
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("no date provided"))?;
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let today = chrono::Local::now().date_naive();

    let mut items = Vec::new();

    if default {
        let conn = state.pool.get().await?;
        items.extend(db::shopping::get_items(conn).await?);
    }

    let conn = state.pool.get().await?;
    let days = db::calendar::get_days(conn, today, date.succ_opt().unwrap()).await?;
    for day in days {
        for (food, factor) in day.lunch.iter().chain(day.dinner.iter()) {
            for ingredient in food.ingredients.iter() {
                let amount = match &ingredient.amount {
                    Some((amount, unit)) => {
                        let amount = amount * factor;
                        Some((amount, unit.clone()))
                    }
                    None => None,
                };

                let item = Item {
                    id: None,
                    name: ingredient.name.clone(),
                    amount,
                    order: items.len() as i32,
                };

                items.push(item);
            }
        }
    }

    let compressed_items = compress_items(&items);

    let bring = state.bring.clone();
    let lists_resp = bring.get_all_lists().await;
    let lists_resp = lists_resp.unwrap();
    let lists = lists_resp.lists;
    let lists = lists
        .into_iter()
        .map(|list| (list.list_uuid, list.name))
        .collect();

    #[derive(Template)]
    #[template(path = "shopping/list.html")]
    struct ShoppingList {
        items: Vec<FinalItem>,
        lists: Vec<(String, String)>,
    }

    let shopping_list = ShoppingList {
        items: compressed_items,
        lists,
    };

    Ok(Html(shopping_list.render()?))
}

fn compress_items(items: &[Item]) -> Vec<FinalItem> {
    let mut compressed_items = HashMap::new();

    for item in items {
        let (order, amounts) = compressed_items
            .entry(item.name.trim().to_string())
            .or_insert((item.order, HashMap::new()));

        match &item.amount {
            Some((amount, unit)) => {
                *amounts.entry(unit.trim().to_string()).or_insert(0.0) += amount;
            }
            None => {}
        }
        if item.order < *order {
            *order = item.order;
        }
    }

    let mut final_items = Vec::new();

    for (name, (order, amounts)) in compressed_items.iter() {
        let mut amount_str = String::new();

        for (unit, quantity) in amounts.iter() {
            if !unit.is_empty() {
                if !amount_str.is_empty() {
                    amount_str.push_str(", ");
                }

                amount_str.push_str(&format!("{} {}", quantity, unit));
            }
        }

        final_items.push(FinalItem {
            id: None,
            name: name.clone(),
            amount: if amount_str.is_empty() {
                None
            } else {
                Some(amount_str)
            },
            order: *order,
        });
    }

    final_items.sort_by_key(|item| item.order);

    final_items
}

#[derive(Deserialize)]
pub struct FinalItem {
    pub id: Option<i32>,
    pub name: String,
    pub amount: Option<String>,
    pub order: i32,
}
