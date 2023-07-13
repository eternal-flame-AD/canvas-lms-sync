use super::{Client, Error};
use async_stream::stream;
use serde::Deserialize;
use tokio_stream::Stream;

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleResp {
    pub id: i64,
    pub name: String,
    pub position: i64,
    pub unlock_at: Option<String>,
    pub require_sequential_progress: bool,
    pub publish_final_grade: bool,
    pub prerequisite_module_ids: Vec<i64>,
    pub state: String,
    pub completed_at: Option<String>,
    pub items_count: i64,
    pub items_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleItemResp {
    pub id: i64,
    pub module_id: i64,
    pub position: i64,
    pub title: String,
    pub indent: i64,
    #[serde(rename = "type")]
    pub type_: String,
    pub content_id: Option<i64>,
    pub html_url: Option<String>,
    pub url: Option<String>,
    pub page_url: Option<String>,
    pub external_url: Option<String>,
    pub new_tab: Option<bool>,
}

impl Client {
    pub fn list_modules(
        &self,
        courseid: i64,
    ) -> impl Stream<Item = Result<ModuleResp, Error>> + '_ {
        let url = self.build_url(&format!("/api/v1/courses/{}/modules", courseid));
        stream! {
            let mut next = Some(url);
            while let Some(url) = next {
                let (resp, pagination) = self.make_json_request::<Vec<ModuleResp>, _>(url).await?;

                next = pagination.and_then(|p| p.next);

                for module in resp {
                    yield Ok(module);
                }

            }
        }
    }
    pub fn list_module_items(
        &self,
        courseid: i64,
        moduleid: i64,
    ) -> impl Stream<Item = Result<ModuleItemResp, Error>> + '_ {
        let url = self.build_url(&format!(
            "/api/v1/courses/{}/modules/{}/items",
            courseid, moduleid
        ));
        stream! {
            let mut next = Some(url);
            while let Some(url) = next {
                let (resp, pagination) = self.make_json_request::<Vec<ModuleItemResp>, _>(url).await?;

                next = pagination.and_then(|p| p.next);

                for module in resp {
                    yield Ok(module);
                }

            }
        }
    }
}
