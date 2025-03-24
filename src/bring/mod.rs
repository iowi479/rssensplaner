use anyhow::Result;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;

mod types;
pub use types::*;

#[derive(Clone)]
pub struct BringConnection {
    client: reqwest::Client,

    _name: String,
    _email: String,
    _password: String,
    base_url: String,
    uuid: String,
    headers: HeaderMap,
    put_headers: HeaderMap,
    _bearer_token: String,
    _refresh_token: String,
}

impl BringConnection {
    pub async fn login(Config { email, password }: &Config) -> Result<Self> {
        let base_url: &'static str = "https://api.getbring.com/rest/v2";
        let login_params: HashMap<&'static str, String> = HashMap::from([
            ("email", email.to_string()),
            ("password", password.to_string()),
        ]);

        let client = reqwest::Client::new();
        let resp = client
            .post(base_url.to_string() + "/bringauth")
            .form(&login_params)
            .send()
            .await?;

        let data = resp.text().await.expect("Could not read response body");
        let data: Value = serde_json::from_str(data.as_str()).expect("Could not parse JSON");

        let name = data["name"].to_string().replace("\"", "");
        let uuid = data["uuid"].to_string().replace("\"", "");
        let bearer_token = data["access_token"].to_string().replace("\"", "");
        let refresh_token = data["refresh_token"].to_string().replace("\"", "");

        let mut headers: HeaderMap = HeaderMap::new();

        headers.insert(
            "X-BRING-API-KEY",
            HeaderValue::from_static("cof4Nc6D8saplXjE3h3HXqHH8m7VU2i1Gs0g85Sp"),
        );
        headers.insert("X-BRING-CLIENT", HeaderValue::from_static("webApp"));
        headers.insert("X-BRING-CLIENT-SOURCE", HeaderValue::from_static("webApp"));
        headers.insert("X-BRING-COUNTRY", HeaderValue::from_static("DE"));
        headers.insert(
            "X-BRING-USER-UUID",
            HeaderValue::from_static("2f7591c5-8a20-4a24-b0ac-5974cb557011"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Bearer {bearer_token}").as_str()).unwrap(),
        );

        let mut put_headers: HeaderMap = headers.clone();
        put_headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );

        return Ok(Self {
            client,
            _name: name,
            _email: email.to_string(),
            _password: password.to_string(),
            base_url: base_url.to_string(),
            uuid,
            headers,
            put_headers,
            _bearer_token: bearer_token,
            _refresh_token: refresh_token,
        });
    }

    pub async fn get_all_lists(&self) -> Result<LoadListsResponse> {
        let resp = self
            .client
            .get(self.base_url.to_owned() + "/bringusers/" + self.uuid.as_str() + "/lists")
            .headers(self.headers.clone())
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let lists: LoadListsResponse = serde_json::from_str(body.as_str()).unwrap();
        return Ok(lists);
    }

    pub async fn get_items_from_list(&self, list_uuid: String) -> Result<GetItemsResponse> {
        let resp = self
            .client
            .get(self.base_url.to_owned() + "/bringlists/" + list_uuid.as_str())
            .headers(self.headers.clone())
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let items: GetItemsResponse = serde_json::from_str(body.as_str()).unwrap();
        return Ok(items);
    }

    pub async fn save_item(
        &self,
        list_uuid: String,
        item_name: String,
        specification: String,
    ) -> Result<()> {
        let mut params: HashMap<&'static str, String> = HashMap::new();
        params.insert("purchase", item_name);
        params.insert("specification", specification);
        params.insert("remove", String::from(""));
        params.insert("sender", String::from("null"));

        let resp = self
            .client
            .put(self.base_url.to_owned() + "/bringlists/" + list_uuid.as_str())
            .headers(self.put_headers.clone())
            .form(&params)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap();
            println!("{}", body);
            return Err(anyhow::anyhow!("Could not save item. {}", body));
        }

        return Ok(());
    }

    pub async fn remove_item(&self, list_uuid: String, item_name: String) -> Result<()> {
        let mut params: HashMap<&'static str, String> = HashMap::new();
        params.insert("purchase", String::from(""));
        params.insert("recently", String::from(""));
        params.insert("specification", String::from(""));
        params.insert("remove", item_name);
        params.insert("sender", String::from("null"));

        let resp = self
            .client
            .put(self.base_url.to_owned() + "/bringlists/" + list_uuid.as_str())
            .headers(self.put_headers.clone())
            .form(&params)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap();
            println!("{}", body);
            return Err(anyhow::anyhow!("Could not remove item. {}", body));
        }

        return Ok(());
    }

    pub async fn move_to_recent_list(&self, list_uuid: String, item_name: String) -> Result<()> {
        let mut params: HashMap<&'static str, String> = HashMap::new();
        params.insert("purchase", String::from(""));
        params.insert("recently", item_name);
        params.insert("specification", String::from(""));
        params.insert("remove", String::from(""));
        params.insert("sender", String::from("null"));

        let resp = self
            .client
            .put(self.base_url.to_owned() + "/bringlists/" + list_uuid.as_str())
            .headers(self.put_headers.clone())
            .form(&params)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap();
            return Err(anyhow::anyhow!("Could not move item to recently. {}", body));
        }

        return Ok(());
    }

    pub async fn load_catalog(&self, locale: &str) -> Result<LoadCatalogResponse> {
        let resp = self
            .client
            .get(format!(
                "https://web.getbring.com/locale/catalog.{locale}.json"
            ))
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let catalog: LoadCatalogResponse = serde_json::from_str(body.as_str()).unwrap();
        return Ok(catalog);
    }
}
