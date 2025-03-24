use crate::utils::Amount;
use askama::Template;
use serde::{Deserialize, Serialize};

/// This represents the editing view of an item in the shoppinglist
#[derive(Debug, Serialize, Deserialize, Clone, Template)]
#[template(path = "shopping/edit_item.html")]
pub struct EditItem {
    pub id: Option<i32>,
    pub name: String,
    pub amount: Option<Amount>,
    pub order: i32,
}

/// This is an Item like it is stored in the database.
/// These are used in the default shopping list.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: Option<i32>,
    pub name: String,
    pub amount: Option<Amount>,
    pub order: i32,
}

impl From<Item> for EditItem {
    fn from(item: Item) -> Self {
        Self {
            id: item.id,
            name: item.name,
            amount: item.amount,
            order: item.order,
        }
    }
}

/// This contains a request to add a list of items to a shopping list.
/// Gets submitted as JSON to the api.
#[derive(Deserialize)]
pub struct AddItemsRequest {
    pub list_id: String,
    pub items: Vec<BringItem>,
}

/// This is a single item that is being added to a shopping list.
/// Bring does not need all the information that is internally stored in the database.
#[derive(Deserialize)]
pub struct BringItem {
    pub name: String,
    pub amount: Option<String>,
}
