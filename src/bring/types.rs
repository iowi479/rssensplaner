use serde::{Deserialize, Serialize};

/// Configuration for the Bring! API
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetItemsResponseEntry {
    pub specification: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetItemsResponse {
    pub uuid: String,
    pub status: String,
    pub purchase: Vec<GetItemsResponseEntry>,
    pub recently: Vec<GetItemsResponseEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoadListsEntry {
    #[serde(rename = "listUuid")]
    pub list_uuid: String,
    pub name: String,
    pub theme: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoadListsResponse {
    pub lists: Vec<LoadListsEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CatalogItemsEntry {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CatalogSectionsEntry {
    #[serde(rename = "sectionId")]
    pub section_id: String,
    pub name: String,
    pub items: Vec<CatalogItemsEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoadCatalogResponse {
    pub language: String,
    pub catalog: CatalogSectionList,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CatalogSectionList {
    pub sections: Vec<CatalogSectionsEntry>,
}

pub const LOCALE_DE: &'static str = "de-DE";
