//! Trefle.io plant database client.
//!
//! Provides species search and detail retrieval from the Trefle.io API,
//! returning images and growth/care data for plant identification enrichment.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

const TREFLE_BASE_URL: &str = "https://trefle.io/api/v1";

/// Trefle.io API client
#[derive(Clone)]
pub struct TrefleClient {
    client: Client,
    token: String,
}

impl TrefleClient {
    /// Create a new Trefle client from the TREFLE_API_TOKEN env var.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("TREFLE_API_TOKEN")
            .context("TREFLE_API_TOKEN environment variable not set")?;
        Self::new(token)
    }

    /// Create a new Trefle client with the given API token.
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .context("Failed to build Trefle HTTP client")?;
        Ok(Self { client, token })
    }

    /// Search for plant species by name (common name or scientific name).
    /// Returns up to `limit` results (max 20 from Trefle).
    pub async fn search_species(&self, query: &str, limit: usize) -> Result<Vec<TrefleSpecies>> {
        let url = format!("{}/plants/search", TREFLE_BASE_URL);
        debug!("Trefle search: query={:?}", query);

        let response = self
            .client
            .get(&url)
            .query(&[("token", &self.token), ("q", &query.to_string())])
            .send()
            .await
            .context("Failed to send request to Trefle.io")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Trefle search error: status={}, body={}", status, body);
            anyhow::bail!("Trefle.io returned status {}", status);
        }

        let search_response: TrefleSearchResponse = response
            .json()
            .await
            .context("Failed to parse Trefle search response")?;

        Ok(search_response.data.into_iter().take(limit).collect())
    }

    /// Get detailed species info by slug (e.g. "monstera-deliciosa").
    /// Returns growth data, images, and full botanical information.
    pub async fn get_species_detail(&self, slug: &str) -> Result<TrefleSpeciesDetail> {
        let url = format!("{}/species/{}", TREFLE_BASE_URL, slug);
        debug!("Trefle species detail: slug={:?}", slug);

        let response = self
            .client
            .get(&url)
            .query(&[("token", &self.token)])
            .send()
            .await
            .context("Failed to send request to Trefle.io")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(
                "Trefle detail error for {}: status={}, body={}",
                slug, status, body
            );
            anyhow::bail!("Trefle.io returned status {} for slug {}", status, slug);
        }

        let detail_response: TrefleDetailResponse = response
            .json()
            .await
            .context("Failed to parse Trefle species detail response")?;

        Ok(detail_response.data)
    }

    /// Search and then fetch detail for the best match.
    /// Returns None if no results found.
    pub async fn search_and_enrich(&self, scientific_name: &str) -> Result<Option<TrefleEnriched>> {
        let results = self.search_species(scientific_name, 1).await?;

        let species = match results.into_iter().next() {
            Some(s) => s,
            None => {
                debug!("No Trefle results for {:?}", scientific_name);
                return Ok(None);
            }
        };

        // Try to get detail for richer data (growth info, multiple images)
        let detail = match self.get_species_detail(&species.slug).await {
            Ok(d) => Some(d),
            Err(e) => {
                debug!(
                    "Could not fetch detail for {}: {} — using search result only",
                    species.slug, e
                );
                None
            }
        };

        Ok(Some(TrefleEnriched { species, detail }))
    }
}

// ─── API Response Types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TrefleSearchResponse {
    pub data: Vec<TrefleSpecies>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrefleSpecies {
    pub id: i64,
    pub slug: String,
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub family: Option<String>,
    pub family_common_name: Option<String>,
    pub genus: Option<String>,
    pub image_url: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrefleDetailResponse {
    pub data: TrefleSpeciesDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrefleSpeciesDetail {
    pub id: i64,
    pub slug: String,
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
    pub image_url: Option<String>,
    #[serde(default)]
    pub images: TrefleImages,
    #[serde(default)]
    pub growth: Option<TrefleGrowth>,
    #[serde(default)]
    pub specifications: Option<TrefleSpecifications>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrefleImages {
    #[serde(default)]
    pub flower: Vec<TrefleImage>,
    #[serde(default)]
    pub leaf: Vec<TrefleImage>,
    #[serde(default)]
    pub habit: Vec<TrefleImage>,
    #[serde(default)]
    pub fruit: Vec<TrefleImage>,
    #[serde(default)]
    pub bark: Vec<TrefleImage>,
    #[serde(default)]
    pub other: Vec<TrefleImage>,
}

impl TrefleImages {
    /// Get all image URLs flattened, prioritizing habit > leaf > flower > other.
    pub fn all_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();
        for img in &self.habit {
            urls.push(img.url.clone());
        }
        for img in &self.leaf {
            urls.push(img.url.clone());
        }
        for img in &self.flower {
            urls.push(img.url.clone());
        }
        for img in &self.other {
            urls.push(img.url.clone());
        }
        for img in &self.fruit {
            urls.push(img.url.clone());
        }
        for img in &self.bark {
            urls.push(img.url.clone());
        }
        urls
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrefleImage {
    #[serde(alias = "image_url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrefleGrowth {
    /// 0 (no light) to 10 (very intensive)
    pub light: Option<i32>,
    /// 0 (<=10% humidity) to 10 (>=90% humidity)
    pub atmospheric_humidity: Option<i32>,
    /// Minimum precipitation mm/year
    pub minimum_precipitation: Option<TrefleMeasurement>,
    /// Maximum precipitation mm/year
    pub maximum_precipitation: Option<TrefleMeasurement>,
    /// Minimum root depth in cm
    pub minimum_root_depth: Option<TrefleMeasurement>,
    /// Minimum temperature tolerance
    pub minimum_temperature: Option<TrefleMeasurement>,
    /// Maximum temperature tolerance
    pub maximum_temperature: Option<TrefleMeasurement>,
    /// Soil humidity 0-10
    pub soil_humidity: Option<i32>,
    /// Soil nutriments 0-10
    pub soil_nutriments: Option<i32>,
    /// Days to harvest
    pub days_to_harvest: Option<i32>,
    /// Textual sowing description
    pub sowing: Option<String>,
    /// pH range
    pub ph_minimum: Option<f64>,
    pub ph_maximum: Option<f64>,
    /// Bloom months
    #[serde(default)]
    pub bloom_months: Option<Vec<String>>,
    /// Growth months
    #[serde(default)]
    pub growth_months: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrefleMeasurement {
    #[serde(alias = "cm", alias = "mm", alias = "deg_c", alias = "deg_f")]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrefleSpecifications {
    pub growth_rate: Option<String>,
    pub average_height: Option<TrefleMeasurement>,
    pub maximum_height: Option<TrefleMeasurement>,
    pub toxicity: Option<String>,
    pub ligneous_type: Option<String>,
}

// ─── Enriched Result (combining search + detail) ────────────────────────────

#[derive(Debug, Clone)]
pub struct TrefleEnriched {
    pub species: TrefleSpecies,
    pub detail: Option<TrefleSpeciesDetail>,
}

impl TrefleEnriched {
    /// Get the best image URLs available (from detail if possible, fallback to search image_url).
    pub fn image_urls(&self) -> Vec<String> {
        if let Some(ref detail) = self.detail {
            let mut urls = detail.images.all_urls();
            if urls.is_empty() {
                if let Some(ref url) = detail.image_url {
                    urls.push(url.clone());
                }
            }
            if !urls.is_empty() {
                return urls;
            }
        }
        // Fallback to search result image
        self.species
            .image_url
            .iter()
            .cloned()
            .collect()
    }

    /// Get growth/care data if available.
    pub fn growth(&self) -> Option<&TrefleGrowth> {
        self.detail.as_ref().and_then(|d| d.growth.as_ref())
    }

    /// Derive a suggested watering interval (days) from Trefle's growth data.
    /// Uses soil_humidity and atmospheric_humidity as signals.
    pub fn suggested_watering_days(&self) -> Option<i32> {
        let growth = self.growth()?;
        // soil_humidity 0-10: higher means more water needed (more frequent watering)
        // atmospheric_humidity 0-10: higher means less evaporation (less frequent)
        let soil_h = growth.soil_humidity.unwrap_or(5);
        let atmo_h = growth.atmospheric_humidity.unwrap_or(5);

        // Heuristic: base interval 7 days, adjusted by soil/humidity needs
        // High soil humidity need (8-10) → every 3-5 days
        // Medium (4-7) → every 5-10 days
        // Low (0-3) → every 10-21 days (succulents/cacti)
        let base = match soil_h {
            0..=2 => 18,
            3 => 14,
            4 => 10,
            5 => 7,
            6 => 6,
            7 => 5,
            8 => 4,
            9..=10 => 3,
            _ => 7,
        };

        // Adjust for atmospheric humidity (high humidity → slightly less frequent)
        let adjustment = match atmo_h {
            0..=3 => -1, // dry air → water slightly more often
            7..=10 => 1, // humid air → water slightly less often
            _ => 0,
        };

        Some((base + adjustment).clamp(2, 28))
    }

    /// Derive a suggested fertilizing interval (days).
    pub fn suggested_fertilizing_days(&self) -> Option<i32> {
        let growth = self.growth()?;
        let nutriments = growth.soil_nutriments.unwrap_or(5);

        // High nutrient needs → more frequent fertilizing
        let days = match nutriments {
            0..=2 => 42, // low feeder → every 6 weeks
            3..=4 => 28, // moderate → every 4 weeks
            5..=6 => 21, // medium-high → every 3 weeks
            7..=8 => 14, // heavy feeder → every 2 weeks
            9..=10 => 10,
            _ => 21,
        };

        Some(days)
    }

    /// Get a light description string from the 0-10 scale.
    pub fn light_description(&self) -> Option<String> {
        let light = self.growth()?.light?;
        let desc = match light {
            0..=2 => "low light (shade tolerant)",
            3..=4 => "low to medium light (partial shade)",
            5..=6 => "medium light (bright indirect)",
            7..=8 => "bright light (some direct sun)",
            9..=10 => "full sun (direct sunlight)",
            _ => "medium light",
        };
        Some(desc.to_string())
    }

    /// Get humidity description.
    pub fn humidity_description(&self) -> Option<String> {
        let h = self.growth()?.atmospheric_humidity?;
        let desc = match h {
            0..=2 => "low humidity (arid/desert)",
            3..=4 => "moderate humidity",
            5..=6 => "moderate to high humidity",
            7..=8 => "high humidity (tropical)",
            9..=10 => "very high humidity",
            _ => "moderate humidity",
        };
        Some(desc.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watering_days_heuristic() {
        // High soil humidity plant (tropical)
        let enriched = TrefleEnriched {
            species: TrefleSpecies {
                id: 1,
                slug: "test".into(),
                scientific_name: "Test plant".into(),
                common_name: None,
                family: None,
                family_common_name: None,
                genus: None,
                image_url: None,
                synonyms: vec![],
            },
            detail: Some(TrefleSpeciesDetail {
                id: 1,
                slug: "test".into(),
                scientific_name: "Test plant".into(),
                common_name: None,
                family: None,
                genus: None,
                image_url: None,
                images: TrefleImages::default(),
                growth: Some(TrefleGrowth {
                    light: Some(6),
                    atmospheric_humidity: Some(7),
                    minimum_precipitation: None,
                    maximum_precipitation: None,
                    minimum_root_depth: None,
                    minimum_temperature: None,
                    maximum_temperature: None,
                    soil_humidity: Some(8),
                    soil_nutriments: Some(6),
                    days_to_harvest: None,
                    sowing: None,
                    ph_minimum: None,
                    ph_maximum: None,
                    bloom_months: None,
                    growth_months: None,
                }),
                specifications: None,
            }),
        };

        let watering = enriched.suggested_watering_days().unwrap();
        assert!(watering >= 3 && watering <= 7, "tropical plant: {}", watering);

        let fertilizing = enriched.suggested_fertilizing_days().unwrap();
        assert_eq!(fertilizing, 21);
    }

    #[test]
    fn test_watering_days_succulent() {
        let enriched = TrefleEnriched {
            species: TrefleSpecies {
                id: 2,
                slug: "cactus".into(),
                scientific_name: "Cactus test".into(),
                common_name: None,
                family: None,
                family_common_name: None,
                genus: None,
                image_url: None,
                synonyms: vec![],
            },
            detail: Some(TrefleSpeciesDetail {
                id: 2,
                slug: "cactus".into(),
                scientific_name: "Cactus test".into(),
                common_name: None,
                family: None,
                genus: None,
                image_url: None,
                images: TrefleImages::default(),
                growth: Some(TrefleGrowth {
                    light: Some(9),
                    atmospheric_humidity: Some(2),
                    minimum_precipitation: None,
                    maximum_precipitation: None,
                    minimum_root_depth: None,
                    minimum_temperature: None,
                    maximum_temperature: None,
                    soil_humidity: Some(1),
                    soil_nutriments: Some(2),
                    days_to_harvest: None,
                    sowing: None,
                    ph_minimum: None,
                    ph_maximum: None,
                    bloom_months: None,
                    growth_months: None,
                }),
                specifications: None,
            }),
        };

        let watering = enriched.suggested_watering_days().unwrap();
        assert!(
            watering >= 14 && watering <= 21,
            "succulent: {}",
            watering
        );

        let fertilizing = enriched.suggested_fertilizing_days().unwrap();
        assert_eq!(fertilizing, 42);
    }

    #[test]
    fn test_light_description() {
        let enriched = TrefleEnriched {
            species: TrefleSpecies {
                id: 3,
                slug: "test".into(),
                scientific_name: "Test".into(),
                common_name: None,
                family: None,
                family_common_name: None,
                genus: None,
                image_url: None,
                synonyms: vec![],
            },
            detail: Some(TrefleSpeciesDetail {
                id: 3,
                slug: "test".into(),
                scientific_name: "Test".into(),
                common_name: None,
                family: None,
                genus: None,
                image_url: None,
                images: TrefleImages::default(),
                growth: Some(TrefleGrowth {
                    light: Some(5),
                    atmospheric_humidity: None,
                    minimum_precipitation: None,
                    maximum_precipitation: None,
                    minimum_root_depth: None,
                    minimum_temperature: None,
                    maximum_temperature: None,
                    soil_humidity: None,
                    soil_nutriments: None,
                    days_to_harvest: None,
                    sowing: None,
                    ph_minimum: None,
                    ph_maximum: None,
                    bloom_months: None,
                    growth_months: None,
                }),
                specifications: None,
            }),
        };

        assert_eq!(
            enriched.light_description().unwrap(),
            "medium light (bright indirect)"
        );
    }
}
