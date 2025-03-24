use super::food::get_foods_in;
use crate::calendar::{Day, ResponseDay};
use crate::db::Connection;
use crate::food::Food;
use anyhow::Result;
use chrono::NaiveDate;

/// Fetches a range of days from the database.
pub async fn get_days(
    mut conn: Connection<'_>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<Day>> {
    let tx = conn.transaction().await?;

    let day_rows = tx
        .query(
            "SELECT * FROM day WHERE date >= $1 AND date < $2 ORDER BY date DESC",
            &[&from, &to],
        )
        .await?;

    // fetch all referenced foods
    let mut ids = Vec::new();
    for row in &day_rows {
        let lunch = db_string_to_foods(row.get("lunch"));
        let dinner = db_string_to_foods(row.get("dinner"));
        lunch.iter().for_each(|(id, _)| ids.push(*id));
        dinner.iter().for_each(|(id, _)| ids.push(*id));
    }
    let foods = get_foods_in(&tx, &ids).await?;

    tx.commit().await?;

    let mut current_date = from;
    let mut days = Vec::new();
    let mut day_iter = day_rows
        .iter()
        .rev()
        .map(|row| row_to_day(row, &foods))
        .peekable();

    // Iterate over all days in the range. Fill in missing days if needed.
    while current_date < to {
        match day_iter.peek() {
            Some(a) if a.date == current_date => {
                // Day was peekable so we can unwrap
                days.push(day_iter.next().unwrap());
            }
            _ => {
                days.push(Day::new(current_date));
            }
        }

        // Increment to next day
        current_date = current_date.succ_opt().expect(&format!(
            "Exceeded maximum date by incrementing {:?}",
            current_date
        ));
    }

    Ok(days)
}

/// Updates or creates a non existing day in the database.
///
/// WARNING: Carefull this allows to create duplicates of a day if two requests interfere.
pub async fn update_day(mut conn: Connection<'_>, mut day: ResponseDay) -> Result<i32> {
    let lunch = foods_to_db_string(day.lunch.into_iter());
    let dinner = foods_to_db_string(day.dinner.into_iter());

    let tx = conn.transaction().await?;

    // Check if there is indeed a day with the given date
    if day.id.is_none() {
        let res = tx
            .query_one("SELECT id FROM day WHERE date = $1", &[&day.date])
            .await;
        if let Ok(row) = res {
            let id: i32 = row.get("id");
            day.id = Some(id);
        }
    }

    let day_id = match day.id {
        Some(id) => {
            tx.execute(
                "UPDATE day SET lunch = $2, dinner = $3 WHERE id = $1",
                &[&id, &lunch, &dinner],
            )
            .await?;

            id
        }
        None => tx
            .query_one(
                "INSERT INTO day (date, lunch, dinner) VALUES ($1, $2, $3) RETURNING id",
                &[&day.date, &lunch, &dinner],
            )
            .await?
            .get("id"),
    };

    tx.commit().await?;

    Ok(day_id)
}

fn row_to_day(row: &tokio_postgres::Row, foods: &Vec<Food>) -> Day {
    let id: i32 = row.get("id");
    let date: NaiveDate = row.get("date");
    let lunch = db_string_to_foods(row.get("lunch"));
    let dinner = db_string_to_foods(row.get("dinner"));

    let lunch = lunch
        .iter()
        .filter_map(|(id, factor)| {
            let food = foods.iter().find(|f| f.id.unwrap() == *id);
            match food {
                Some(f) => Some((f.clone(), *factor)),
                None => None,
            }
        })
        .collect();

    let dinner = dinner
        .iter()
        .filter_map(|(id, factor)| {
            let food = foods.iter().find(|f| f.id.unwrap() == *id);
            match food {
                Some(f) => Some((f.clone(), *factor)),
                None => None,
            }
        })
        .collect();

    Day {
        id: Some(id),
        date,
        lunch,
        dinner,
    }
}

/// Convert food ids and factors to a string that can be stored in the database.
/// Example: [(1, 1.5), (2, 2.0)] -> "1,1.5;2,2.0"
fn foods_to_db_string(pairs: impl Iterator<Item = (i32, f32)>) -> String {
    let mut s = String::new();
    for (id, factor) in pairs {
        s.push_str(&id.to_string());
        s.push(',');
        s.push_str(&factor.to_string());
        s.push(';');
    }
    s.pop();
    s
}

/// Convert a string from the database to a vector of food ids and factors.
/// Example: "1,1.5;2,2.0" -> [(1, 1.5), (2, 2.0)]
fn db_string_to_foods(s: &str) -> Vec<(i32, f32)> {
    if s.is_empty() {
        return Vec::new();
    }

    s.split(';')
        .map(|s| {
            let (id, factor) = s.split_once(',').unwrap();
            (id.parse().unwrap(), factor.parse().unwrap())
        })
        .collect()
}
