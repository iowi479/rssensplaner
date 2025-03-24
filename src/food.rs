use crate::utils::Amount;
use askama::Template;
use serde::{Deserialize, Serialize};

/// This represents a food card in the overview. For editing a food look at `EditFood`.
#[derive(Debug, Serialize, Deserialize, Clone, Template)]
#[template(path = "food/food.html")]
pub struct Food {
    pub id: Option<i32>,
    pub name: String,
    pub tags: Vec<String>,
    pub details: String,
    pub portions: i32,
    pub ingredients: Vec<Ingredient>,
}

/// This is an Ingredient like it is stored in the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ingredient {
    pub id: Option<i32>,
    pub name: String,
    pub amount: Option<Amount>,
    pub optional: bool,
}

/// This represents an Ingredient in the edit view.
/// This can be used to create a new Ingredient or edit an existing one.
#[derive(Debug, Clone, Template)]
#[template(path = "food/edit_ingredient.html")]
pub struct EditIngredient {
    pub id: Option<i32>,
    pub name: String,
    pub amount: Option<Amount>,
    pub optional: bool,
}

impl From<Ingredient> for EditIngredient {
    fn from(ingredient: Ingredient) -> Self {
        EditIngredient {
            id: ingredient.id,
            name: ingredient.name,
            amount: ingredient.amount,
            optional: ingredient.optional,
        }
    }
}

impl Into<Ingredient> for EditIngredient {
    fn into(self) -> Ingredient {
        Ingredient {
            id: self.id,
            name: self.name,
            amount: self.amount,
            optional: self.optional,
        }
    }
}

/// This represents the editing view of a food.
/// This can be used to create a new food or edit an existing one.
#[derive(Debug, Clone, Template)]
#[template(path = "food/edit_food.html")]
pub struct EditFood {
    pub id: Option<i32>,
    pub name: String,
    pub tags: Vec<String>,
    pub details: String,

    pub portions: i32,
    pub edit_ingredients: Vec<EditIngredient>,

    pub existing_items: Vec<String>,
    pub existing_tags: Vec<String>,
}

impl EditFood {
    pub fn create(existing_items: Vec<String>, existing_tags: Vec<String>) -> Self {
        EditFood {
            id: None,
            name: "".to_string(),
            tags: vec![],
            details: "".to_string(),
            portions: 4,
            edit_ingredients: vec![],
            existing_items,
            existing_tags,
        }
    }

    pub fn edit_food(food: Food, existing_items: Vec<String>, existing_tags: Vec<String>) -> Self {
        EditFood {
            id: food.id,
            name: food.name,
            tags: food.tags,
            details: food.details,
            portions: food.portions,
            edit_ingredients: food.ingredients.into_iter().map(|i| i.into()).collect(),
            existing_items,
            existing_tags,
        }
    }
}

impl Into<Food> for EditFood {
    fn into(self) -> Food {
        Food {
            id: self.id,
            name: self.name,
            tags: self.tags,
            details: self.details,
            portions: self.portions,
            ingredients: self
                .edit_ingredients
                .into_iter()
                .map(|i| i.into())
                .collect(),
        }
    }
}
