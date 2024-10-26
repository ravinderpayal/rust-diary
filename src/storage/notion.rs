// storage/notion.rs
// use crate::Storage;
use async_trait::async_trait;
use chrono::NaiveDate;
use ids::{AsIdentifier, BlockId};
use notion::models::block::Block;
use notion::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, ClientBuilder, RequestBuilder};
use serde::Serialize;
use serde_json::json;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use tracing::Instrument;

use diary_app::Storage;
use std::collections::HashMap;

use crate::storage::notion_md_interop::{MarkdownToNotionBlocks, ToMarkdown};

pub struct NotionStorage {
    client: Arc<NotionApi>,
    api_token: String,
    database_id: ids::DatabaseId,
}

impl NotionStorage {
    pub fn new(token: String, database_id: String) -> Self {
        let d_id = ids::DatabaseId::from_str(format!("{}", &database_id).as_ref())
            .expect("Valid Database Id is required");
        let notion_api = Arc::new(NotionApi::new(token.clone()).expect("Notion is setup"));
        NotionStorage {
            client: notion_api,
            api_token: token.clone(),
            database_id: d_id,
        }
    }

    async fn get_blocks_in_a_page(
        &self,
        page_id: &ids::PageId,
    ) -> Result<Vec<models::block::Block>, Box<dyn Error>> {
        println!("Finding blocks in  Page:{}", &page_id.to_string());

        /*
                let page = self
                    .client
                    .get_page(ids::PageId::from_str(&page_id).expect("Page id is not in correct format"))
                    .await?;
                let title: String = page.properties.title().unwrap_or_else(|| "Title not available".to_string());

                println!("{}", title);
        */
        let blocks = self
            .client
            .get_block_children(
                ids::BlockId::from_str(page_id.to_string().as_ref())
                    .expect("This shouldn't go south"),
            )
            .await?;
        // todo : implement filter
        Ok(blocks.results.into_iter().map(|b| b.clone()).collect())
    }
    async fn find_latest_page(&self) -> Result<Option<ids::PageId>, Box<dyn Error>> {
        println!("Finding Page in the database:{}", &self.database_id);

        let pages = self
            .client
            .query_database(
                &self.database_id,
                models::search::DatabaseQuery {
                    filter: None,
                    sorts: Some(vec![models::search::DatabaseSort {
                        property: None,
                        direction: notion::models::search::SortDirection::Descending,
                        timestamp: Some(notion::models::search::DatabaseSortTimestamp::CreatedTime),
                    }]),
                    paging: None,
                },
            )
            .await?;
        println!("pages fetched: {}", pages.results.len());
        // todo : implement filter
        Ok(pages.results.get(0).map(|page| page.id.clone()))
    }

    async fn find_page_for_date(
        &self,
        date: NaiveDate,
    ) -> Result<Option<ids::PageId>, Box<dyn Error>> {
        let filter = json!({
            "property": "Date",
            "date": {
                "equals": date.format("%Y-%m-%d").to_string()
            }
        });

        println!("Finding Page in the database:{}", &self.database_id);

        let pages = self
            .client
            .query_database(
                &self.database_id,
                models::search::DatabaseQuery {
                    filter: Some(models::search::FilterCondition::Property {
                        property: "title".to_string(),
                        condition: models::search::PropertyCondition::RichText(
                            models::search::TextCondition::Equals(
                                date.format("%Y-%m-%d").to_string(),
                            ),
                        ),
                    }),
                    sorts: None,
                    paging: None,
                },
            )
            .await?;
        println!("pages fetched: {}", pages.results.len());
        // todo : implement filter
        Ok(pages.results.get(0).map(|page| page.id.clone()))
    }
 
    async fn create_new_page(&self, content: &str, date: NaiveDate)  -> Result<(), Box<dyn Error>> {
            // Update existing page
            // let blocks = notion_to_blocks::string_to_blocks(content);
            // self.client.update_block_children(&page_id, blocks).await?;

            let title = date.format("%Y-%m-%d").to_string();
            let properties = HashMap::from([(
                "Name".to_string(),
                models::properties::PropertyValue::Title {
                    id: ids::PropertyId::from_str("title-asdadadadada")
                        .expect("Title PropertyId, what might have went wrong lol?"),
                    title: vec![models::text::RichText::Text {
                        rich_text: models::text::RichTextCommon {
                            plain_text: title.clone(),
                            href: None,
                            annotations: None,
                        },
                        text: models::text::Text {
                            content: title.clone(),
                            link: None,
                        },
                    }],
                },
            )]);

            let page = models::PageCreateRequest {
                parent: models::Parent::Database {
                    database_id: self.database_id.clone()
                },
                properties: models::Properties { properties },
                children: Some(
                    content.to_notion_blocks(), // vec![get_notion_block_for_content(content.to_string())]
                ),
            };

            let page_req = async move { self.client.create_page(page).await };

            let page_req_response = page_req.await;
            match page_req_response {
                Ok(_) => println!("Synced with notion"),
                Err(err) => {
                    println!("Sync failed {}", err);
                },
            };
            // todo: Implement backup and sync if notion call fails
 
        return Ok(());
    }

}

#[async_trait(?Send)]
impl Storage for NotionStorage {
    async fn save_entry(&self, date: NaiveDate, content: &str) -> Result<(), Box<dyn Error>> {
        if let Some(page_id) = self.find_page_for_date(date.clone()).await? {
            // let client = std::sync::Arc::clone(&self.client);
            println!("Updating existing page: {}", page_id.to_string());
            if let Ok(blocks) = self.get_blocks_in_a_page(&page_id).await {
                let block_ids: Vec<ids::BlockId> = blocks
                    .iter()
                    .map(get_block_id)
                    .filter(|opt| opt.is_some())
                    .map(|opt| opt.unwrap())
                    .collect();

                println!("Deleting Existing Blocks(Count: {}) using page archive method in single hit", block_ids.len());
                // delete_page
                match delete_blocks(&self.api_token, vec![BlockId::from_str(page_id.to_string().as_ref()).unwrap()]).await {
                    Err(er) => eprintln!(
                        "Delete Process failed somehow {}: {}",
                        &page_id.as_id(),
                        &er
                    ),
                    Ok(_) => (),
                };
                println!("Inserting fresh content");
                /*
                match insert_blocks_in_page(&self.api_token, &page_id, content.to_notion_blocks())
                    .await
                {
                    Err(er) => eprintln!("Insert failed somehow {}: {}", &page_id.as_id(), &er),
                    Ok(_) => println!("Saved Successfully"),
                };*/

                self.create_new_page(content, date).await?;
                /*if block_id_opt.is_some() {
                    let block_id = block_id_opt.unwrap();
                    println!("Block Id Acquired:{}", block_id);
                    let block = get_notion_block_for_content(content_copy);

                    let update_res = update_block(&api_token, block_id, block).await;
                    match update_res {
                        Ok(_) => println!("Saved Successfully"),
                        Err(e) => println!("Failed because {}", e),
                    }
                };*/
            };
        } else {
            self.create_new_page(content, date).await?;
       }
        Ok(())
    }

     async fn get_entry(&self, date: NaiveDate) -> Result<Option<String>, Box<dyn Error>> {
        if let Ok(Some(page_id)) = self.find_page_for_date(date).await {
            if let Ok(blocks) = self.get_blocks_in_a_page(&page_id).await {
                println!("Block found, converting to MD[GE]");
                Ok(Some(blocks.iter().map(|b| b.to_markdown()).collect()))
            } else {
                Ok(Some("Block not found-GE".to_string()))
            }
        } else {
            println!("Page Not Found");
            Ok(None)
        }
    }

    async fn get_latest_entry(&self) -> Result<Option<(NaiveDate, String)>, Box<dyn Error>> {
        if let Ok(Some(page_id)) = self.find_latest_page().await {
            // todo: figure this code out, first get first block and the childran
            // let blocks = self.client.get_block_children(&page_id, None).await?;
            // let content = notion_to_blocks::blocks_to_string(&blocks.results);
            // let client = std::sync::Arc::clone(&self.client);
            if let Ok(blocks) = self.get_blocks_in_a_page(&page_id).await {
                println!("Block found, converting to MD[LE]");
                Ok(Some((
                    NaiveDate::from_ymd_opt(2024, 10, 6).unwrap(),
                    blocks.iter().map(|b| b.to_markdown()).collect(),
                )))
            } else {
                println!("Block not found[LE]");
                Ok(Some((
                    NaiveDate::from_ymd_opt(2024, 10, 6).unwrap(),
                    "Block not found-LE".to_string(),
                )))
            }
        } else {
            Ok(None)
        }
    }
}

fn get_notion_block_for_content(content: String) -> models::block::CreateBlock {
    models::block::CreateBlock::Paragraph {
        paragraph: models::block::TextAndChildren {
            color: models::text::TextColor::Default,
            children: None,
            rich_text: vec![models::text::RichText::Text {
                rich_text: models::text::RichTextCommon {
                    plain_text: content.clone(),
                    href: None,
                    annotations: None,
                },
                text: models::text::Text {
                    content: content.clone(),
                    link: None,
                },
            }],
        },
    }
}

const NOTION_API_VERSION: &str = "2022-02-22";

async fn delete_blocks<P>(api_token: &String, block_ids: Vec<P>) -> Result<bool, DNError>
where
    P: ids::AsIdentifier<ids::BlockId>,
{
    for block_id in block_ids {
        match delete_block(api_token, &block_id).await {
            Err(er) => eprintln!("{} {}", &block_id.as_id(), er),
            Ok(_) => (),
        };
    }
    Ok(true)
}

async fn insert_blocks_in_page<T>(
    api_token: &String,
    page_id: &ids::PageId,
    blocks: T,
) -> Result<bool, DNError>
where
    T: Into<Vec<models::block::CreateBlock>>,
{
    /*
          curl -X PATCH 'https://api.notion.com/v1/blocks/b55c9c91-384d-452b-81db-d1ef79372b75/children' \
      -H 'Authorization: Bearer '"$NOTION_API_KEY"'' \
      -H "Content-Type: application/json" \
      -H "Notion-Version: 2022-06-28" \
      --data '{
       "children": [
            {
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{ "type": "text", "text": { "content": "Lacinato kale" } }]
                }
            },
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": "Lacinato kale is a variety of kale with a long tradition in Italian cuisine, especially that of Tuscany. It is also known as Tuscan kale, Italian kale, dinosaur kale, kale, flat back kale, palm tree kale, or black Tuscan palm.",
                                "link": { "url": "https://en.wikipedia.org/wiki/Lacinato_kale" }
                            }
                        }
                    ]
                }
            }
        ]
    }'
        */
    // todo: implement
    let mut headers = HeaderMap::new();
    headers.insert(
        "Notion-Version",
        HeaderValue::from_static(NOTION_API_VERSION),
    );

    let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", api_token))
        .map_err(|source| DNError::InvalidApiToken { source })?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .map_err(|source| DNError::ErrorBuildingClient { source })?;

    // using CreateBlock is not perfect
    // technically speaking you can update text on a todo block and not touch checked by not settings it
    // but I don't want to create a new type for this
    // or make checked optional in CreateBlock
    // T: Serialize + ?Sized
    let json_body = json!({
        "children": blocks.into()
    });
    let req = client
        .patch(&format!(
            "https://api.notion.com/v1/blocks/{block_id}/children",
            block_id = page_id.to_string()
        ))
        .json(&json_body) // iter().map(|b| b.into()).collect())
        .build()
        .unwrap();
    client.execute(req).await?;

    Ok(true)
}

/// Delete a block by [BlockId].
async fn delete_block<P>(api_token: &String, block_id: &P) -> Result<bool, Box<dyn Error>>
where
    P: ids::AsIdentifier<ids::BlockId>,
{
    //  database_id: String,

    let mut headers = HeaderMap::new();
    headers.insert(
        "Notion-Version",
        HeaderValue::from_static(NOTION_API_VERSION),
    );

    let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", api_token))
        .map_err(|source| DNError::InvalidApiToken { source })?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .map_err(|source| DNError::ErrorBuildingClient { source })?;

    // using CreateBlock is not perfect
    // technically speaking you can update text on a todo block and not touch checked by not settings it
    // but I don't want to create a new type for this
    // or make checked optional in CreateBlock
    let req = client.execute(
        client
            .delete(&format!(
                "https://api.notion.com/v1/blocks/{block_id}",
                block_id = block_id.as_id()
            ))
            .build()
            .unwrap(),
    );
    req.await?;
    Ok(true)
}

/// Update a block by [BlockId].
pub async fn update_block<P, T>(
    api_token: &String,
    block_id: P,
    block: T,
) -> Result<models::block::Block, DNError>
where
    P: ids::AsIdentifier<ids::BlockId>,
    T: Into<models::block::CreateBlock>,
{
    //  database_id: String,

    let mut headers = HeaderMap::new();
    headers.insert(
        "Notion-Version",
        HeaderValue::from_static(NOTION_API_VERSION),
    );

    let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", api_token))
        .map_err(|source| DNError::InvalidApiToken { source })?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .map_err(|source| DNError::ErrorBuildingClient { source })?;

    // using CreateBlock is not perfect
    // technically speaking you can update text on a todo block and not touch checked by not settings it
    // but I don't want to create a new type for this
    // or make checked optional in CreateBlock
    let req = client
        .patch(&format!(
            "https://api.notion.com/v1/blocks/{block_id}",
            block_id = block_id.as_id()
        ))
        .json(&block.into());
    let result = make_json_request(client, req).await?;

    match result {
        models::Object::Block { block } => Ok(block),
        response => Err(DNError::UnexpectedResponse { response }),
    }
}

/// An wrapper Error type for all errors produced by the [`NotionApi`](NotionApi) client.
#[derive(Debug, thiserror::Error)]
enum DNError {
    #[error("Invalid Notion API Token: {}", source)]
    InvalidApiToken { source: header::InvalidHeaderValue },

    #[error("Unable to build reqwest HTTP client: {}", source)]
    ErrorBuildingClient { source: reqwest::Error },

    #[error("Error sending HTTP request: {}", source)]
    RequestFailed {
        #[from]
        source: reqwest::Error,
    },

    #[error("Error reading response: {}", source)]
    ResponseIoError { source: reqwest::Error },

    #[error("Error parsing json response: {}", source)]
    JsonParseError { source: serde_json::Error },

    #[error("Unexpected API Response")]
    UnexpectedResponse { response: models::Object },

    #[error("API Error {}({}): {}", .error.code, .error.status, .error.message)]
    ApiError { error: models::error::ErrorResponse },
}

async fn make_json_request(
    client: Client,
    request: RequestBuilder,
) -> Result<models::Object, DNError> {
    let request = request.build()?;
    let url = request.url();
    tracing::trace!(
        method = request.method().as_str(),
        url = url.as_str(),
        "Sending request"
    );
    let json = client
        .execute(request)
        .instrument(tracing::trace_span!("Sending request"))
        .await
        .map_err(|source| DNError::RequestFailed { source })?
        .text()
        .instrument(tracing::trace_span!("Reading response"))
        .await
        .map_err(|source| DNError::ResponseIoError { source })?;

    tracing::debug!("JSON Response: {}", json);
    #[cfg(test)]
    {
        dbg!(serde_json::from_str::<serde_json::Value>(&json)
            .map_err(|source| Error::JsonParseError { source })?);
    }
    let result =
        serde_json::from_str(&json).map_err(|source| DNError::JsonParseError { source })?;

    match result {
        models::Object::Error { error } => Err(DNError::ApiError { error }),
        response => Ok(response),
    }
}

fn get_block_id(block: &Block) -> Option<ids::BlockId> {
    match block {
        Block::Paragraph { common, .. } => Some(common.id.clone()),
        Block::Heading1 { common, .. } => Some(common.id.clone()),
        Block::Heading2 { common, .. } => Some(common.id.clone()),
        Block::Heading3 { common, .. } => Some(common.id.clone()),
        Block::Callout { common, .. } => Some(common.id.clone()),
        Block::Quote { common, .. } => Some(common.id.clone()),
        Block::BulletedListItem { common, .. } => Some(common.id.clone()),
        Block::NumberedListItem { common, .. } => Some(common.id.clone()),
        Block::ToDo { common, .. } => Some(common.id.clone()),
        Block::Toggle { common, .. } => Some(common.id.clone()),
        Block::Code { common, .. } => Some(common.id.clone()),
        Block::ChildPage { common, .. } => Some(common.id.clone()),
        Block::ChildDatabase { common, .. } => Some(common.id.clone()),
        Block::Embed { common, .. } => Some(common.id.clone()),
        Block::Image { common, .. } => Some(common.id.clone()),
        Block::Video { common, .. } => Some(common.id.clone()),
        Block::File { common, .. } => Some(common.id.clone()),
        Block::Pdf { common, .. } => Some(common.id.clone()),
        Block::Bookmark { common, .. } => Some(common.id.clone()),
        Block::Equation { common, .. } => Some(common.id.clone()),
        Block::Divider { common } => Some(common.id.clone()),
        Block::TableOfContents { common, .. } => Some(common.id.clone()),
        Block::Breadcrumb { common } => Some(common.id.clone()),
        Block::ColumnList { common, .. } => Some(common.id.clone()),
        Block::Column { common, .. } => Some(common.id.clone()),
        Block::LinkPreview { common, .. } => Some(common.id.clone()),
        Block::Template { common, .. } => Some(common.id.clone()),
        Block::LinkToPage { common, .. } => Some(common.id.clone()),
        Block::Table { common, .. } => Some(common.id.clone()),
        Block::SyncedBlock { common, .. } => Some(common.id.clone()),
        Block::TableRow { common, .. } => Some(common.id.clone()),
        Block::Unsupported { common } => Some(common.id.clone()),
        Block::Unknown => None,
    }
}
