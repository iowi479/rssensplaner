use crate::bring::BringConnection;
use crate::db;
use crate::server_config::ServerConfig;
use axum::response::Redirect;
use axum::routing::{get, post};

pub mod handlers;

/// AppState which holds the connections needed to provide the API.
#[derive(Clone)]
pub struct AppState {
    pool: db::ConnectionPool,
    bring: BringConnection,
}

/// Initializes the REST API and returns the router.
pub async fn init_rest_api(config: ServerConfig) -> axum::Router {
    let pool = db::generate_pool(&config.database)
        .await
        .expect("Could not create database pool");

    let bring = BringConnection::login(&config.bring)
        .await
        .expect("Could noit create Bring! API connection");

    let state = AppState { pool, bring };

    let mut app = axum::Router::new();

    app = app.route("/", get(|| async { Redirect::permanent("/list") }));
    app = app.fallback(get(handlers::fallback_handler));

    //food
    app = app.route(
        "/food/delete/{id}",
        post(handlers::food::delete_food_handler),
    );
    app = app
        .route("/food/get/", get(handlers::food::get_all_foods_handler))
        .route("/food/get/{id}", get(handlers::food::get_food_handler))
        .route("/food/{id}", get(handlers::food::edit_food_handler))
        .route("/food/update", post(handlers::food::update_food_handler))
        .route("/food/create", get(handlers::food::create_food_handler))
        .route("/list", get(handlers::food::food_list_handler));

    // calendar
    app = app
        .route("/calendar", get(handlers::calendar::get_calendar_handler))
        .route("/day/update", post(handlers::calendar::update_day_handler))
        .route("/day/{id}", get(handlers::calendar::get_day_handler));

    // shopping
    app = app
        .route(
            "/shopping",
            get(handlers::shopping::get_shopping_index_handler),
        )
        .route(
            "/shopping/get",
            get(handlers::shopping::get_all_items_handler),
        )
        .route(
            "/shopping/delete/{id}",
            post(handlers::shopping::delete_item_handler),
        )
        .route(
            "/shopping/update",
            post(handlers::shopping::update_item_handler),
        )
        .route(
            "/shopping/defaults",
            get(handlers::shopping::default_item_handler),
        )
        .route(
            "/shopping/list",
            get(handlers::shopping::shopping_list_handler),
        );

    // Bring! API
    app = app.route("/shopping/bring", post(handlers::bring::add_bring_handler));

    app.with_state(state)
}
