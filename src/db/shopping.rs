use std::collections::HashSet;

use crate::db::Connection;
use crate::shopping::{EditItem, Item};
use crate::utils::{amount_unit_to_string, string_to_amount_unit};
use anyhow::Result;

/// This function retrieves all items from the database.
pub async fn get_items(mut conn: Connection<'_>) -> Result<Vec<Item>> {
    let tx = conn.transaction().await?;
    let item_rows = tx
        .query("SELECT * FROM item ORDER BY ordering ASC", &[])
        .await?;
    tx.commit().await?;

    let items = rows_to_items(item_rows)?;
    Ok(items)
}

/// This removes an item from the database.
pub async fn delete_item(mut conn: Connection<'_>, id: i32) -> Result<()> {
    let tx = conn.transaction().await?;

    tx.execute("DELETE FROM item WHERE id = $1", &[&id]).await?;

    tx.commit().await?;

    Ok(())
}

/// This can update existing items or create new ones.
pub async fn update_items(mut conn: Connection<'_>, items: &Vec<EditItem>) -> Result<()> {
    let tx = conn.transaction().await?;

    // Fetch all the old item ids to be able to remove the ones that are no longer used.
    let mut old_item_ids: HashSet<i32> = tx
        .query("SELECT id FROM item", &[])
        .await?
        .iter()
        .map(|row| row.get("id"))
        .collect();

    for item in items {
        match item.id {
            Some(id) => {
                old_item_ids.remove(&id);
                tx.execute(
                    "UPDATE item SET name = $2, amount = $3, ordering = $4 WHERE id = $1",
                    &[
                        &id,
                        &item.name,
                        &amount_unit_to_string(&item.amount),
                        &item.order,
                    ],
                )
                .await?;
            }
            None => {
                tx.execute(
                    "INSERT INTO item (name, amount, ordering) VALUES ($1, $2, $3)",
                    &[
                        &item.name,
                        &amount_unit_to_string(&item.amount),
                        &item.order,
                    ],
                )
                .await?;
            }
        };
    }

    // Remove the items that are no longer used.
    for id in old_item_ids {
        tx.execute("DELETE FROM item WHERE id = $1", &[&id]).await?;
    }

    tx.commit().await?;

    Ok(())
}

/// This function is used to convert rows from the database into a Vec of Item structs.
fn rows_to_items(rows: Vec<tokio_postgres::Row>) -> Result<Vec<Item>> {
    let mut items = Vec::new();

    for row in rows {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let amount_str = row.get::<_, Option<String>>("amount");
        let amount = string_to_amount_unit(&amount_str)?;
        let order: i32 = row.get("ordering");

        let item = Item {
            id: Some(id),
            name,
            amount,
            order,
        };
        items.push(item);
    }

    Ok(items)
}
