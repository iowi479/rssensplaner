use crate::food::Food;
use askama::Template;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Template)]
#[template(path = "calendar/day.html")]
/// The foods get associated with a factor of how many times the recipe is needed.
/// Example:
///     Recipe has 4 portions, but we need 6 portions. The factor is 1.5
pub struct Day {
    pub id: Option<i32>,
    pub date: NaiveDate,
    pub lunch: Vec<(Food, f32)>,
    pub dinner: Vec<(Food, f32)>,
}

impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Day {
            id: None,
            date,
            lunch: Vec::new(),
            dinner: Vec::new(),
        }
    }
}

/// This is used in the template to calculate the amount of ingredients needed.
pub fn mult_portions(portions: &i32, factor: &f32) -> f32 {
    (*portions as f32) * factor
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseDay {
    pub id: Option<i32>,
    pub date: NaiveDate,
    pub lunch: Vec<(i32, f32)>,
    pub dinner: Vec<(i32, f32)>,
}
