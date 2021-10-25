//! # 🚀 Getting Started
//!
//! ```rust
//! use meilisearch_sdk::{document::*, client::*, search::*};
//! use serde::{Serialize, Deserialize};
//! use futures::executor::block_on;
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Movie {
//!     id: usize,
//!     title: String,
//!     genres: Vec<String>,
//! }
//!
//! // That trait is required to make a struct usable by an index
//! impl Document for Movie {
//!     type UIDType = usize;
//!
//!     fn get_uid(&self) -> &Self::UIDType {
//!         &self.id
//!     }
//! }
//!
//! fn main() { block_on(async move {
//!     // Create a client (without sending any request so that can't fail)
//!     let client = Client::new("http://localhost:7700", "masterKey");
//!
//!     // Get the index called "movies"
//!     let movies = client.get_or_create("movies").await.unwrap();
//!
//!     // Add some movies in the index
//!     movies.add_documents(&[
//!         Movie{id: 1, title: String::from("Carol"), genres: vec!["Romance".to_string(), "Drama".to_string()]},
//!         Movie{id: 2, title: String::from("Wonder Woman"), genres: vec!["Action".to_string(), "Adventure".to_string()]},
//!         Movie{id: 3, title: String::from("Life of Pi"), genres: vec!["Adventure".to_string(), "Drama".to_string()]},
//!         Movie{id: 4, title: String::from("Mad Max"), genres: vec!["Adventure".to_string(), "Science Fiction".to_string()]},
//!         Movie{id: 5, title: String::from("Moana"), genres: vec!["Fantasy".to_string(), "Action".to_string()]},
//!         Movie{id: 6, title: String::from("Philadelphia"), genres: vec!["Drama".to_string()]},
//!     ], Some("id")).await.unwrap();
//!
//!     // Query movies (note that there is a typo)
//!     println!("{:?}", movies.search().with_query("carol").execute::<Movie>().await.unwrap().hits);
//! })}
//! ```
//!
//! Output:
//!
//! ```text
//! [Movie{id: 1, title: String::from("Carol"), genres: vec!["Romance", "Drama"]}]
//! ```
//!
//! #### Custom Search With Filters <!-- omit in TOC -->
//!
//! If you want to enable filtering, you must add your attributes to the `filterableAttributes`
//! index setting.
//!
//! ```
//! # use meilisearch_sdk::{document::*, client::*, search::*};
//! # use serde::{Serialize, Deserialize};
//! # use futures::executor::block_on;
//! # #[derive(Serialize, Deserialize, Debug)]
//! # struct Movie {
//! #     id: usize,
//! #     title: String,
//! #     genres: Vec<String>,
//! # }
//! # // That trait is required to make a struct usable by an index
//! # impl Document for Movie {
//! #     type UIDType = usize;
//! #    fn get_uid(&self) -> &Self::UIDType {
//! #        &self.id
//! #   }
//! # }
//! # fn main() { block_on(async move {
//! #    // Create a client (without sending any request so that can't fail)
//! #    let client = Client::new("http://localhost:7700", "masterKey");
//! #     // Get the index called "movies"
//! #    let movies = client.get_or_create("movies").await.unwrap();
//! let filterable_attributes = [
//!     "id",
//!     "genres"
//! ];
//! movies.set_filterable_attributes(&filterable_attributes).await.unwrap();
//! #    // Add some movies in the index
//! #    movies.add_documents(&[
//! #        Movie{id: 1, title: String::from("Carol"), genres: vec!["Romance".to_string(), "Drama".to_string()]},
//! #        Movie{id: 2, title: String::from("Wonder Woman"), genres: vec!["Action".to_string(), "Adventure".to_string()]},
//! #        Movie{id: 3, title: String::from("Life of Pi"), genres: vec!["Adventure".to_string(), "Drama".to_string()]},
//! #        Movie{id: 4, title: String::from("Mad Max"), genres: vec!["Adventure".to_string(), "Science Fiction".to_string()]},
//! #        Movie{id: 5, title: String::from("Moana"), genres: vec!["Fantasy".to_string(), "Action".to_string()]},
//! #        Movie{id: 6, title: String::from("Philadelphia"), genres: vec!["Drama".to_string()]},
//! #    ], Some("id")).await.unwrap();
//! #    // Query movies (note that there is a typo)
//! #    println!("{:?}", movies.search().with_query("carol").execute::<Movie>().await.unwrap().hits);
//! # })}
//! ```
//!
//! You only need to perform this operation once.
//!
//! Note that MeiliSearch will rebuild your index whenever you update `filterableAttributes`.
//! Depending on the size of your dataset, this might take time. You can track the whole process
//! using the [update
//! status](https://docs.meilisearch.com/reference/api/updates.html#get-an-update-status).
//!
//! Then, you can perform the search:
//!
//! ```
//! # use meilisearch_sdk::{document::*, client::*, search::*};
//! # use serde::{Serialize, Deserialize};
//! # use futures::executor::block_on;
//! # #[derive(Serialize, Deserialize, Debug)]
//! # struct Movie {
//! #    id: usize,
//! #    title: String,
//! #    genres: Vec<String>,
//! # }
//! # // That trait is required to make a struct usable by an index
//! # impl Document for Movie {
//! #    type UIDType = usize;
//! #    fn get_uid(&self) -> &Self::UIDType {
//! #        &self.id
//! #    }
//! # }
//! # fn main() { block_on(async move {
//! # // Create a client (without sending any request so that can't fail)
//! # let client = Client::new("http://localhost:7700", "masterKey");
//! #   // Get the index called "movies"
//! # let movies = client.get_or_create("movies").await.unwrap();
//! # let filterable_attributes = [
//! #     "id",
//! #    "genres"
//! # ];
//! # movies.set_filterable_attributes(&filterable_attributes).await.unwrap();
//! #   // Add some movies in the index
//! #   movies.add_documents(&[
//! #       Movie{id: 1, title: String::from("Carol"), genres: vec!["Romance".to_string(), "Drama".to_string()]},
//! #       Movie{id: 2, title: String::from("Wonder Woman"), genres: vec!["Action".to_string(), "Adventure".to_string()]},
//! #       Movie{id: 3, title: String::from("Life of Pi"), genres: vec!["Adventure".to_string(), "Drama".to_string()]},
//! #       Movie{id: 4, title: String::from("Mad Max"), genres: vec!["Adventure".to_string(), "Science Fiction".to_string()]},
//! #       Movie{id: 5, title: String::from("Moana"), genres: vec!["Fantasy".to_string(), "Action".to_string()]},
//! #       Movie{id: 6, title: String::from("Philadelphia"), genres: vec!["Drama".to_string()]},
//! #   ], Some("id")).await.unwrap();
//! println!("{:?}", movies.search().with_query("wonder").with_filter("id > 1 AND genres = Action")
//! .execute::<Movie>().await.unwrap().hits);
//! # })}
//! ```
//!
//! ```json
//! {
//!   "hits": [
//!     {
//!       "id": 2,
//!       "title": "Wonder Woman",
//!       "genres": ["Action", "Adventure"]
//!     }
//!   ],
//!   "offset": 0,
//!   "limit": 20,
//!   "nbHits": 1,
//!   "processingTimeMs": 0,
//!   "query": "wonder"
//! }
//! ```

#![warn(clippy::all)]
#![allow(clippy::needless_doctest_main)]

/// Module containing the Client struct.
pub mod client;
/// Module containing the Document trait.
pub mod document;
pub mod dumps;
/// Module containing the Error struct.
pub mod errors;
/// Module containing the Index struct.
pub mod indexes;
/// Module containing objects useful for tracking the progress of async operations.
pub mod progress;
mod request;
/// Module related to search queries and results.
pub mod search;
/// Module containing settings
pub mod settings;

#[cfg(feature = "sync")]
pub(crate) type Rc<T> = std::sync::Arc<T>;
#[cfg(not(feature = "sync"))]
pub(crate) type Rc<T> = std::rc::Rc<T>;
