use super::AppError;
use crate::{server::AppState, shopping::AddItemsRequest};
use axum::{Json, extract::State, response::IntoResponse};

/// The submitted items are added to the specified list.
///
/// This is not a transactional operation. If one of the items cannot be added, the already
/// completed items will still be added.
pub async fn add_bring_handler(
    State(state): State<AppState>,
    Json(request): Json<AddItemsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let bring = state.bring.clone();

    for item in request.items.iter() {
        let amount = match &item.amount {
            Some(amount) => amount.to_string(),
            None => String::new(),
        };

        bring
            .save_item(request.list_id.clone(), item.name.clone(), amount)
            .await?;
    }

    Ok(Json(
        serde_json::json!({ "result": "success", "item_count": request.items.len() }),
    ))
}
