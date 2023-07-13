use super::{Client, Error};
use async_stream::stream;
use serde::Deserialize;
use tokio_stream::Stream;

#[derive(Debug, Clone, Deserialize)]
pub struct FolderResp {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub context_id: i64,
    pub context_type: String,
    pub parent_folder_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub lock_at: Option<String>,
    pub unlock_at: Option<String>,
    pub position: Option<i64>,
    pub locked: bool,
    pub folders_url: String,
    pub files_url: String,
    pub files_count: i64,
    pub folders_count: i64,
    pub hidden: Option<bool>,
    pub locked_for_user: bool,
    pub hidden_for_user: bool,
    pub for_submissions: bool,
    pub can_upload: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileResp {
    pub id: i64,
    pub uuid: String,
    pub folder_id: i64,
    pub display_name: String,
    pub filename: String,
    pub upload_status: String,
    #[serde(rename = "content-type")]
    pub content_type: String,
    pub url: String,
    pub size: i64,
    pub created_at: String,
    pub updated_at: String,
    pub unlock_at: Option<String>,
    pub locked: bool,
    pub hidden: bool,
    pub lock_at: Option<String>,
    pub hidden_for_user: bool,
    pub modified_at: String,
    pub mime_class: String,
    pub media_entry_id: Option<String>,
    pub locked_for_user: bool,
}

impl Client {
    pub fn get_all_folders(
        &self,
        courseid: i64,
    ) -> impl Stream<Item = Result<FolderResp, Error>> + '_ {
        let url = self.build_url(&format!("/api/v1/courses/{}/folders", courseid));

        stream! {
            let mut next = Some(url);
            while let Some(url) = next {
                let (data, pagination) = self.make_json_request::<Vec<FolderResp>, _>(url).await?;
                next = pagination.and_then(|p| p.next);
                for folder in data {
                    yield Ok(folder);
                }
            }
        }
    }
    pub fn get_all_files(&self, courseid: i64) -> impl Stream<Item = Result<FileResp, Error>> + '_ {
        let url = self.build_url(&format!("/api/v1/courses/{}/files", courseid));

        stream! {
            let mut next = Some(url);
            while let Some(url) = next {
                let (data, pagination) = self.make_json_request::<Vec<FileResp>, _>(url).await?;
                next = pagination.and_then(|p| p.next);
                for file in data {
                    yield Ok(file);
                }
            }
        }
    }
    pub async fn get_course_file(&self, courseid: i64, fileid: i64) -> Result<FileResp, Error> {
        let url = self.build_url(&format!("/api/v1/courses/{}/files/{}", courseid, fileid));

        let (data, _) = self.make_json_request::<FileResp, _>(url).await?;

        Ok(data)
    }
}
