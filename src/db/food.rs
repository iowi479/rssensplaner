use crate::db::Connection;
use crate::food::{Food, Ingredient};
use crate::utils::{amount_unit_to_string, string_to_amount_unit, string_to_vec, vec_to_string};
use anyhow::Result;
use std::collections::HashSet;
use tokio_postgres::Transaction;

/// This function retrieves all ingredient names from the database.
/// This is for autocompletion in the frontend.
pub async fn get_all_ingredient_names(mut conn: Connection<'_>) -> Result<Vec<String>> {
    let tx = conn.transaction().await?;
    let rows = tx
        .query("SELECT DISTINCT name FROM ingredient", &[])
        .await?;
    tx.commit().await?;

    let ingredients = rows.iter().map(|row| row.get("name")).collect();

    Ok(ingredients)
}

pub async fn get_all_existing_tags(mut conn: Connection<'_>) -> Result<Vec<String>> {
    let tx = conn.transaction().await?;
    let rows = tx.query("SELECT DISTINCT tags FROM food", &[]).await?;
    tx.commit().await?;

    let tags = rows
        .iter()
        .map(|row| string_to_vec(&row.get::<_, String>("tags")))
        .flatten()
        .collect::<HashSet<String>>();

    Ok(tags.into_iter().collect())
}

pub async fn get_food(mut conn: Connection<'_>, id: i32) -> Result<Food> {
    let tx = conn.transaction().await?;
    let food_row = tx
        .query_one("SELECT * FROM food WHERE id = $1", &[&id])
        .await?;

    let ingredient_rows = tx
        .query("SELECT * FROM ingredient WHERE food_id = $1", &[&id])
        .await?;

    tx.commit().await?;

    let food = row_to_food(food_row, ingredient_rows.into_iter())?;

    Ok(food)
}

pub async fn get_all_foods(mut conn: Connection<'_>) -> Result<Vec<Food>> {
    let tx = conn.transaction().await?;

    let food_rows = tx.query("SELECT * FROM food ORDER BY id ASC", &[]).await?;

    let ingredient_rows = tx
        .query("SELECT * FROM ingredient ORDER BY food_id DESC", &[])
        .await?;

    tx.commit().await?;

    let foods = rows_to_food(food_rows, ingredient_rows)?;

    Ok(foods)
}

pub async fn delete_food(mut conn: Connection<'_>, id: i32) -> Result<()> {
    let tx = conn.transaction().await?;

    tx.execute("DELETE FROM ingredient WHERE food_id = $1", &[&id])
        .await?;
    tx.execute("DELETE FROM food WHERE id = $1", &[&id]).await?;

    tx.commit().await?;

    Ok(())
}

/// This can update an existing food or create a new one.
/// This should also handle all cases where ingredients are added, removed, or updated.
pub async fn update_food(mut conn: Connection<'_>, food: &Food) -> Result<i32> {
    let tx = conn.transaction().await?;

    // Update or insert the food
    let food_id = if let Some(id) = food.id {
        tx.execute(
            "UPDATE food SET name = $2, tags = $3, details = $4, portions = $5 WHERE id = $1",
            &[
                &id,
                &food.name,
                &vec_to_string(&food.tags),
                &food.details,
                &food.portions,
            ],
        )
        .await?;

        id
    } else {
        let id: i32 = tx
            .query_one(
                "INSERT INTO food (name, tags, details, portions) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&food.name, &vec_to_string(&food.tags), &food.details, &food.portions],
            )
            .await?
            .get("id");

        id
    };

    // Fetch all the old ingredient ids to be able to remove the ones that are no longer used.
    let mut old_ingredient_ids: HashSet<i32> = tx
        .query("SELECT id From ingredient WHERE food_id = $1", &[&food.id])
        .await?
        .iter()
        .map(|row| row.get("id"))
        .collect();

    // Update or insert the ingredients
    for ingredient in &food.ingredients {
        if let Some(id) = ingredient.id {
            old_ingredient_ids.remove(&id);
            tx.execute(
                "UPDATE ingredient SET name = $1, amount = $2, optional = $3 WHERE id = $4",
                &[
                    &ingredient.name,
                    &amount_unit_to_string(&ingredient.amount),
                    &ingredient.optional,
                    &id,
                ],
            )
            .await?;
        } else {
            tx.execute(
                "INSERT INTO ingredient (food_id, name, amount, optional) VALUES ($1, $2, $3, $4)",
                &[
                    &food_id,
                    &ingredient.name,
                    &amount_unit_to_string(&ingredient.amount),
                    &ingredient.optional,
                ],
            )
            .await?;
        }
    }

    // Remove the ingredients that are no longer used.
    for id in old_ingredient_ids {
        tx.execute("DELETE FROM ingredient WHERE id = $1", &[&id])
            .await?;
    }

    tx.commit().await?;

    Ok(food_id)
}

/// Convert rows from the food and ingredient tables to a vector of Food structs.
/// The ingredient rows are expected to be ordered by food_id descending.
/// This is for easier parsing of the ingredients.
fn rows_to_food(
    food_rows: Vec<tokio_postgres::Row>,
    mut ingredient_rows: Vec<tokio_postgres::Row>,
) -> Result<Vec<Food>> {
    let mut foods = Vec::new();

    for food_row in food_rows {
        let id: i32 = food_row.get("id");

        // Iter reversed because the ingredients are ordered by food_id descending.
        // Find the index of the fist ingredient that is no longer for the current food.
        let idx = ingredient_rows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, row)| row.get::<_, i32>("food_id") != id)
            .map(|(i, _)| i + 1)
            .unwrap_or(0);

        let ingredient_rows = ingredient_rows.split_off(idx);
        let food = row_to_food(food_row, ingredient_rows.into_iter())?;

        foods.push(food);
    }

    Ok(foods)
}

fn row_to_food(
    food_row: tokio_postgres::Row,
    ingredient_rows: impl Iterator<Item = tokio_postgres::Row>,
) -> Result<Food> {
    let id = food_row.get("id");
    let name: String = food_row.get("name");
    let portions = food_row.get("portions");
    let tags: Vec<String> = string_to_vec(&food_row.get("tags"));
    let details: String = food_row.get("details");

    let ingredients = rows_to_ingredients(ingredient_rows)?;

    let food = Food {
        id,
        name,
        tags,
        details,
        portions,
        ingredients,
    };

    Ok(food)
}

/// Convert rows from the ingredients table to a vector of Ingredient structs.
fn rows_to_ingredients(rows: impl Iterator<Item = tokio_postgres::Row>) -> Result<Vec<Ingredient>> {
    let mut ingredients = Vec::new();

    for row in rows {
        let id = row.get("id");
        let name: String = row.get("name");
        let amount = string_to_amount_unit(&row.get("amount"))?;
        let optional: bool = row.get("optional");

        let ingredient = Ingredient {
            id,
            name,
            amount,
            optional,
        };

        ingredients.push(ingredient);
    }

    Ok(ingredients)
}

/// Fetches all foods with the given ids.
pub async fn get_foods_in(tx: &Transaction<'_>, ids: &[i32]) -> Result<Vec<Food>> {
    let (in_clause, params) = match build_in_clause(&ids) {
        None => return Ok(Vec::new()),
        Some(in_clause) => in_clause,
    };

    let food_query = format!("SELECT * FROM food WHERE id IN {}", &in_clause);
    let ingredient_query = format!(
        "SELECT * FROM ingredient WHERE food_id IN {} ORDER BY food_id DESC",
        &in_clause
    );

    let food_rows = tx.query(&food_query, &params).await?;
    let ingredient_rows = tx.query(&ingredient_query, &params).await?;

    let foods = rows_to_food(food_rows, ingredient_rows)?;

    Ok(foods)
}

/// Build the query string and parameters for the IN clause.
/// The query string will look like "($1,$2,$3,...$n)".
/// And the parameters will be the ids corresponding to the $1,$2,$3,...$n.
///
/// The Vec with elements of: `&(dyn tokio_postgres::types::ToSql + Sync)>)`
/// is for tokio_postgres.
fn build_in_clause(
    ids: &[i32],
) -> Option<(String, Vec<&(dyn tokio_postgres::types::ToSql + Sync)>)> {
    if ids.is_empty() {
        return None;
    }

    let mut query = String::from("(");
    let mut params = Vec::new();

    for id in ids.iter().enumerate() {
        query.push_str(&format!("${},", id.0 + 1));

        // as _ is needed to convert i32 to tokio_postgres::types::ToSql
        params.push(id.1 as _);
    }

    assert!(query.ends_with(','));

    query.pop();
    query.push(')');

    Some((query, params))
}
