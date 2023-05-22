use crate::task_info::TaskInfo;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Derive the [`IndexConfig`](crate::documents::IndexConfig) trait.
///
/// ## Field attribute
/// Use the `#[index_config(..)]` field attribute to generate the correct settings
/// for each field. The available parameters are:
/// - `primary_key` (can only be used once)
/// - `distinct` (can only be used once)
/// - `searchable`
/// - `displayed`
/// - `filterable`
/// - `sortable`
///
/// ## Index name
/// The name of the index will be the name of the struct converted to snake case.
///
/// ## Sample usage:
/// ```
/// use serde::{Serialize, Deserialize};
/// use meilisearch_sdk::documents::IndexConfig;
/// use meilisearch_sdk::settings::Settings;
/// use meilisearch_sdk::indexes::Index;
/// use meilisearch_sdk::client::Client;
///
/// #[derive(Serialize, Deserialize, IndexConfig)]
/// struct Movie {
///     #[index_config(primary_key)]
///     movie_id: u64,
///     #[index_config(displayed, searchable)]
///     title: String,
///     #[index_config(displayed)]
///     description: String,
///     #[index_config(filterable, sortable, displayed)]
///     release_date: String,
///     #[index_config(filterable, displayed)]
///     genres: Vec<String>,
/// }
///
/// async fn usage(client: Client) {
///     // Default settings with the distinct, searchable, displayed, filterable, and sortable fields set correctly.
///     let settings: Settings = Movie::generate_settings();
///     // Index created with the name `movie` and the primary key set to `movie_id`
///     let index: Index = Movie::generate_index(&client).await.unwrap();
/// }
/// ```
pub use meilisearch_index_setting_macro::IndexConfig;

use crate::settings::Settings;
use crate::tasks::Task;
use crate::Client;
use crate::{errors::Error, indexes::Index};

#[async_trait]
pub trait IndexConfig {
    const INDEX_STR: &'static str;

    fn index(client: &Client) -> Index {
        client.index(Self::INDEX_STR)
    }
    fn generate_settings() -> Settings;
    async fn generate_index(client: &Client) -> Result<Index, Task>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentsResults<T> {
    pub results: Vec<T>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentQuery<'a> {
    #[serde(skip_serializing)]
    pub index: &'a Index,

    /// The fields that should appear in the documents. By default all of the fields are present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<&'a str>>,
}

impl<'a> DocumentQuery<'a> {
    pub fn new(index: &Index) -> DocumentQuery {
        DocumentQuery {
            index,
            fields: None,
        }
    }

    /// Specify the fields to return in the document.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));
    /// let index = client.index("document_query_with_fields");
    /// let mut document_query = DocumentQuery::new(&index);
    ///
    /// document_query.with_fields(["title"]);
    /// ```
    pub fn with_fields(
        &mut self,
        fields: impl IntoIterator<Item = &'a str>,
    ) -> &mut DocumentQuery<'a> {
        self.fields = Some(fields.into_iter().collect());
        self
    }

    /// Execute the get document query.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// # use serde::{Deserialize, Serialize};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));
    /// # futures::executor::block_on(async move {
    /// #[derive(Debug, Serialize, Deserialize, PartialEq)]
    /// struct MyObject {
    ///     id: String,
    ///     kind: String,
    /// }
    ///
    /// #[derive(Debug, Serialize, Deserialize, PartialEq)]
    /// struct MyObjectReduced {
    ///     id: String,
    /// }
    /// # let index = client.index("document_query_execute");
    /// # index.add_or_replace(&[MyObject{id:"1".to_string(), kind:String::from("a kind")},MyObject{id:"2".to_string(), kind:String::from("some kind")}], None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    ///
    /// let document = DocumentQuery::new(&index).with_fields(["id"])
    ///     .execute::<MyObjectReduced>("1")
    ///     .await
    ///     .unwrap();
    ///
    /// assert_eq!(
    ///     document,
    ///     MyObjectReduced { id: "1".to_string() }
    /// );
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    pub async fn execute<T: DeserializeOwned + 'static>(
        &self,
        document_id: &str,
    ) -> Result<T, Error> {
        self.index.get_document_with::<T>(document_id, self).await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentsQuery<'a> {
    #[serde(skip_serializing)]
    pub index: &'a Index,

    /// The number of documents to skip.
    ///
    /// If the value of the parameter `offset` is `n`, the `n` first documents will not be returned.
    /// This is helpful for pagination.
    ///
    /// Example: If you want to skip the first document, set offset to `1`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,

    /// The maximum number of documents returned.
    /// If the value of the parameter `limit` is `n`, there will never be more than `n` documents in the response.
    /// This is helpful for pagination.
    ///
    /// Example: If you don't want to get more than two documents, set limit to `2`.
    ///
    /// **Default: `20`**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// The fields that should appear in the documents. By default all of the fields are present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<&'a str>>,
}

impl<'a> DocumentsQuery<'a> {
    pub fn new(index: &Index) -> DocumentsQuery {
        DocumentsQuery {
            index,
            offset: None,
            limit: None,
            fields: None,
        }
    }

    /// Specify the offset.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));
    /// let index = client.index("my_index");
    ///
    /// let mut documents_query = DocumentsQuery::new(&index).with_offset(1);
    /// ```
    pub fn with_offset(&mut self, offset: usize) -> &mut DocumentsQuery<'a> {
        self.offset = Some(offset);
        self
    }

    /// Specify the limit.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));
    /// let index = client.index("my_index");
    ///
    /// let mut documents_query = DocumentsQuery::new(&index);
    ///
    /// documents_query.with_limit(1);
    /// ```
    pub fn with_limit(&mut self, limit: usize) -> &mut DocumentsQuery<'a> {
        self.limit = Some(limit);
        self
    }

    /// Specify the fields to return in the documents.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));
    /// let index = client.index("my_index");
    ///
    /// let mut documents_query = DocumentsQuery::new(&index);
    ///
    /// documents_query.with_fields(["title"]);
    /// ```
    pub fn with_fields(
        &mut self,
        fields: impl IntoIterator<Item = &'a str>,
    ) -> &mut DocumentsQuery<'a> {
        self.fields = Some(fields.into_iter().collect());
        self
    }

    /// Execute the get documents query.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// # use serde::{Deserialize, Serialize};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY));
    /// # futures::executor::block_on(async move {
    /// # let index = client.create_index("documents_query_execute", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap().try_make_index(&client).unwrap();
    /// #[derive(Debug, Serialize, Deserialize, PartialEq)]
    /// struct MyObject {
    ///     id: Option<usize>,
    ///     kind: String,
    /// }
    /// let index = client.index("documents_query_execute");
    ///
    /// let document = DocumentsQuery::new(&index)
    ///     .with_offset(1)
    ///     .execute::<MyObject>()
    ///     .await
    ///     .unwrap();
    ///
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn execute<T: DeserializeOwned + 'static>(
        &self,
    ) -> Result<DocumentsResults<T>, Error> {
        self.index.get_documents_with::<T>(self).await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentDeletionQuery<'a> {
    #[serde(skip_serializing)]
    pub index: &'a Index,

    /// Filters to apply.
    ///
    /// Read the [dedicated guide](https://docs.meilisearch.com/reference/features/filtering.html) to learn the syntax.
    pub filter: &'a str,
}

impl<'a> DocumentDeletionQuery<'a> {
    pub fn new(index: &Index) -> DocumentDeletionQuery {
        DocumentDeletionQuery { index, filter: "" }
    }

    pub fn with_filter<'b>(&'b mut self, filter: &'a str) -> &'b mut DocumentDeletionQuery<'a> {
        self.filter = filter;
        self
    }

    pub async fn execute<T: DeserializeOwned + 'static>(&self) -> Result<TaskInfo, Error> {
        self.index.delete_documents_with(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::*, errors::*, indexes::*};
    use meilisearch_test_macro::meilisearch_test;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct MyObject {
        id: Option<usize>,
        kind: String,
    }

    #[allow(unused)]
    #[derive(IndexConfig)]
    struct MovieClips {
        #[index_config(primary_key)]
        movie_id: u64,
        #[index_config(distinct)]
        owner: String,
        #[index_config(displayed, searchable)]
        title: String,
        #[index_config(displayed)]
        description: String,
        #[index_config(filterable, sortable, displayed)]
        release_date: String,
        #[index_config(filterable, displayed)]
        genres: Vec<String>,
    }

    #[allow(unused)]
    #[derive(IndexConfig)]
    struct VideoClips {
        video_id: u64,
    }

    async fn setup_test_index(client: &Client, index: &Index) -> Result<(), Error> {
        let t0 = index
            .add_documents(
                &[
                    MyObject {
                        id: Some(0),
                        kind: "text".into(),
                    },
                    MyObject {
                        id: Some(1),
                        kind: "text".into(),
                    },
                    MyObject {
                        id: Some(2),
                        kind: "title".into(),
                    },
                    MyObject {
                        id: Some(3),
                        kind: "title".into(),
                    },
                ],
                None,
            )
            .await?;

        t0.wait_for_completion(client, None, None).await?;

        Ok(())
    }

    #[meilisearch_test]
    async fn test_get_documents_with_execute(client: Client, index: Index) -> Result<(), Error> {
        setup_test_index(&client, &index).await?;
        let documents = DocumentsQuery::new(&index)
            .with_limit(1)
            .with_offset(1)
            .with_fields(["kind"])
            .execute::<MyObject>()
            .await
            .unwrap();

        assert_eq!(documents.limit, 1);
        assert_eq!(documents.offset, 1);
        assert_eq!(documents.results.len(), 1);

        Ok(())
    }

    #[meilisearch_test]
    async fn test_delete_documents_with(client: Client, index: Index) -> Result<(), Error> {
        setup_test_index(&client, &index).await?;
        index
            .set_filterable_attributes(["id"])
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();
        let mut query = DocumentDeletionQuery::new(&index);
        query.with_filter("id = 1");

        index
            .delete_documents_with(&query)
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();
        let document_result = index.get_document::<MyObject>("1").await;

        match document_result {
            Ok(_) => panic!("The test was expecting no documents to be returned but got one."),
            Err(e) => match e {
                Error::Meilisearch(err) => {
                    assert_eq!(err.error_code, ErrorCode::DocumentNotFound);
                }
                _ => panic!("The error was expected to be a Meilisearch error, but it was not."),
            },
        }

        Ok(())
    }

    #[meilisearch_test]
    async fn test_get_documents_with_only_one_param(
        client: Client,
        index: Index,
    ) -> Result<(), Error> {
        setup_test_index(&client, &index).await?;
        // let documents = index.get_documents(None, None, None).await.unwrap();
        let documents = DocumentsQuery::new(&index)
            .with_limit(1)
            .execute::<MyObject>()
            .await
            .unwrap();

        assert_eq!(documents.limit, 1);
        assert_eq!(documents.offset, 0);
        assert_eq!(documents.results.len(), 1);

        Ok(())
    }

    #[meilisearch_test]
    async fn test_settings_generated_by_macro(client: Client, index: Index) -> Result<(), Error> {
        setup_test_index(&client, &index).await?;

        let movie_settings: Settings = MovieClips::generate_settings();
        let video_settings: Settings = VideoClips::generate_settings();

        assert_eq!(movie_settings.searchable_attributes.unwrap(), ["title"]);
        assert!(video_settings.searchable_attributes.unwrap().is_empty());

        assert_eq!(
            movie_settings.displayed_attributes.unwrap(),
            ["title", "description", "release_date", "genres"]
        );
        assert!(video_settings.displayed_attributes.unwrap().is_empty());

        assert_eq!(
            movie_settings.filterable_attributes.unwrap(),
            ["release_date", "genres"]
        );
        assert!(video_settings.filterable_attributes.unwrap().is_empty());

        assert_eq!(
            movie_settings.sortable_attributes.unwrap(),
            ["release_date"]
        );
        assert!(video_settings.sortable_attributes.unwrap().is_empty());

        Ok(())
    }

    #[meilisearch_test]
    async fn test_generate_index(client: Client) -> Result<(), Error> {
        let index: Index = MovieClips::generate_index(&client).await.unwrap();

        assert_eq!(index.uid, "movie_clips");

        index
            .delete()
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }
    #[derive(Serialize, Deserialize, IndexConfig)]
    struct Movie {
        #[index_config(primary_key)]
        movie_id: u64,
        #[index_config(displayed, searchable)]
        title: String,
        #[index_config(displayed)]
        description: String,
        #[index_config(filterable, sortable, displayed)]
        release_date: String,
        #[index_config(filterable, displayed)]
        genres: Vec<String>,
    }
}
