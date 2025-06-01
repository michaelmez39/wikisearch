use anyhow::Error as E;
use qdrant_client::{
    Payload, Qdrant,
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, ScalarQuantizationBuilder,
        SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
        value::Kind,
    },
};
use uuid::Uuid;

use crate::embedder::Embedder;

const WIKITEXT: &'static str = "wikitext";

pub struct SearchDb {
    client: Qdrant,
    embed: Embedder,
}

impl SearchDb {
    pub fn from_url(url: &str) -> Self {
        let client = Qdrant::from_url(url).build().unwrap();
        let embed = Embedder::new().expect("should be able to create embedder");
        Self { client, embed }
    }

    /// TODO: Build from config file
    pub fn new() -> Self {
        Self::from_url("http://localhost:6334")
    }

    pub async fn upsert(&self, message: &str) -> Result<(), E> {
        let point = self.map_message(message)?;

        self.client
            .upsert_points(UpsertPointsBuilder::new(WIKITEXT, vec![point]))
            .await?;

        Ok(())
    }

    fn map_message(&self, message: &str) -> Result<PointStruct, E> {
        let embedding = self.embed.embed(message)?;
        let id = Uuid::new_v4().to_string();
        let payload: Payload = serde_json::json!({ "message": message }).try_into()?;

        Ok(PointStruct::new(id, embedding, payload))
    }

    pub async fn upsert_bulk(&self, messages: &Vec<&str>) -> Result<(), E> {
        let points: Vec<PointStruct> = self
            .embed
            .bulk_embed(messages)?
            .into_iter()
            .enumerate()
            .map(|(idx, embedding)| {
                let id = Uuid::new_v4().to_string();
                let payload: Payload = serde_json::json!({ "message": messages[idx]})
                    .try_into()
                    .unwrap();

                PointStruct::new(id, embedding, payload)
            })
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(WIKITEXT, points))
            .await?;

        Ok(())
    }

    pub async fn search(&self, search_text: &str) -> Result<String, E> {
        let search_vector = self.embed.embed(search_text)?;

        let search = SearchPointsBuilder::new(WIKITEXT, search_vector, 384)
            .with_payload(true)
            .limit(3)
            .params(SearchParamsBuilder::default().exact(false));

        let search_result = self.client.search_points(search).await?;

        let Some(best_match) = search_result.result.iter().next() else {
            return Err(E::msg("No matching point"));
        };

        let message = best_match
            .payload
            .get("message")
            .ok_or(E::msg("missing message"))?;

        match &message.kind {
            Some(Kind::StringValue(s)) => Ok(s.clone()),
            _ => Err(E::msg("invalid message")),
        }
    }

    pub async fn create_wikitext_collection(&self) -> Result<(), E> {
        let vector_config = VectorParamsBuilder::new(384, Distance::Cosine);

        let wikitext_collection = CreateCollectionBuilder::new(WIKITEXT)
            .vectors_config(vector_config)
            .quantization_config(ScalarQuantizationBuilder::default());

        self.client.create_collection(wikitext_collection).await?;
        Ok(())
    }

    pub async fn delete_wikitext_collection(&self) -> Result<(), E> {
        self.client.delete_collection(WIKITEXT).await?;
        Ok(())
    }
}
