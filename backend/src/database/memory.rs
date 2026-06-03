use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::memory::{
    ExtractedFact, MemoryFactType, MemorySource, PlantHealthScore, PlantMemoriesResponse,
    PlantMemory,
};
use crate::utils::errors::AppError;

// ─── Plant Memories ──────────────────────────────────────────────────────────

pub async fn list_memories_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<PlantMemoriesResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let rows = sqlx::query_as!(
        MemoryRow,
        r#"SELECT id, plant_id, user_id, fact_type, content, confidence, source, source_message_id as "source_message_id?: String", created_at, updated_at
         FROM plant_memories WHERE plant_id = ? AND user_id = ? ORDER BY fact_type, created_at"#,
        plant_id_str,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let memories = rows
        .into_iter()
        .filter_map(|r| r.to_memory().ok())
        .collect();

    Ok(PlantMemoriesResponse { memories })
}

pub async fn get_memory(
    pool: &DatabasePool,
    memory_id: &Uuid,
    user_id: &str,
) -> Result<PlantMemory, AppError> {
    let memory_id_str = memory_id.to_string();
    let row = sqlx::query_as!(
        MemoryRow,
        r#"SELECT id, plant_id, user_id, fact_type, content, confidence, source, source_message_id as "source_message_id?: String", created_at, updated_at
         FROM plant_memories WHERE id = ? AND user_id = ?"#,
        memory_id_str,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or(AppError::NotFound {
        resource: format!("Memory {memory_id}"),
    })?;

    row.to_memory()
}

pub async fn create_memory(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    fact_type: &MemoryFactType,
    content: &str,
    confidence: f64,
    source: MemorySource,
    source_message_id: Option<&str>,
) -> Result<PlantMemory, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now().to_rfc3339();
    let id_str = id.to_string();
    let plant_id_str = plant_id.to_string();
    let fact_type_str = fact_type.to_string();
    let source_str = source.to_string();

    sqlx::query!(
        r#"INSERT INTO plant_memories (id, plant_id, user_id, fact_type, content, confidence, source, source_message_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        id_str,
        plant_id_str,
        user_id,
        fact_type_str,
        content,
        confidence,
        source_str,
        source_message_id,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    get_memory(pool, &id, user_id).await
}

pub async fn update_memory(
    pool: &DatabasePool,
    memory_id: &Uuid,
    user_id: &str,
    content: Option<&str>,
    fact_type: Option<&MemoryFactType>,
) -> Result<PlantMemory, AppError> {
    // Verify it exists and belongs to user
    let _existing = get_memory(pool, memory_id, user_id).await?;

    let now = Utc::now().to_rfc3339();
    let memory_id_str = memory_id.to_string();

    if let Some(content) = content {
        sqlx::query!(
            r#"UPDATE plant_memories SET content = ?, confidence = 1.0, updated_at = ? WHERE id = ? AND user_id = ?"#,
            content,
            now,
            memory_id_str,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(ft) = fact_type {
        let ft_str = ft.to_string();
        sqlx::query!(
            r#"UPDATE plant_memories SET fact_type = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            ft_str,
            now,
            memory_id_str,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    get_memory(pool, memory_id, user_id).await
}

pub async fn delete_memory(
    pool: &DatabasePool,
    memory_id: &Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    let memory_id_str = memory_id.to_string();
    let result = sqlx::query!(
        r#"DELETE FROM plant_memories WHERE id = ? AND user_id = ?"#,
        memory_id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Memory {memory_id}"),
        });
    }
    Ok(())
}

/// Store facts extracted by the coach from a conversation.
/// Deduplicates: if a fact of the same type with very similar content exists,
/// update confidence instead of creating a duplicate.
pub async fn store_extracted_facts(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    facts: &[ExtractedFact],
    source_message_id: Option<&str>,
) -> Result<Vec<PlantMemory>, AppError> {
    let mut stored = Vec::new();

    for fact in facts {
        let fact_type = match fact.fact_type.parse::<MemoryFactType>() {
            Ok(ft) => ft,
            Err(_) => MemoryFactType::General,
        };

        // Check for existing fact of the same type with similar content
        let plant_id_str = plant_id.to_string();
        let fact_type_str = fact_type.to_string();
        let existing = sqlx::query_as!(
            MemoryRow,
            r#"SELECT id, plant_id, user_id, fact_type, content, confidence, source, source_message_id as "source_message_id?: String", created_at, updated_at
             FROM plant_memories WHERE plant_id = ? AND user_id = ? AND fact_type = ?
             ORDER BY updated_at DESC LIMIT 5"#,
            plant_id_str,
            user_id,
            fact_type_str
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        // Simple dedup: if content is very similar (starts with same prefix), update instead
        let duplicate = existing.iter().find(|e| {
            let existing_lower = e.content.to_lowercase();
            let new_lower = fact.content.to_lowercase();
            existing_lower == new_lower
                || (existing_lower.len() > 10
                    && new_lower.len() > 10
                    && existing_lower[..existing_lower.len().min(20)]
                        == new_lower[..new_lower.len().min(20)])
        });

        if let Some(dup) = duplicate {
            // Update existing with new content and bump confidence
            let now = Utc::now().to_rfc3339();
            let new_confidence = (fact.confidence).max(dup.confidence).min(1.0);
            let dup_id = &dup.id;
            sqlx::query!(
                r#"UPDATE plant_memories SET content = ?, confidence = ?, source_message_id = ?, updated_at = ? WHERE id = ?"#,
                fact.content,
                new_confidence,
                source_message_id,
                now,
                dup_id
            )
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

            if let Ok(mem) =
                get_memory(pool, &Uuid::parse_str(&dup.id).unwrap_or_default(), user_id).await
            {
                stored.push(mem);
            }
        } else {
            // Create new memory
            let mem = create_memory(
                pool,
                plant_id,
                user_id,
                &fact_type,
                &fact.content,
                fact.confidence,
                MemorySource::Coach,
                source_message_id,
            )
            .await?;
            stored.push(mem);
        }
    }

    Ok(stored)
}

/// Get memories formatted as context string for the coach system prompt
pub async fn get_memory_context(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<String, AppError> {
    let response = list_memories_for_plant(pool, plant_id, user_id).await?;

    if response.memories.is_empty() {
        return Ok(String::new());
    }

    let mut context = String::from("\n## Known Facts About This Plant\n");
    for mem in &response.memories {
        let confidence_marker = if mem.confidence >= 0.9 {
            ""
        } else if mem.confidence >= 0.7 {
            " (likely)"
        } else {
            " (uncertain)"
        };
        context.push_str(&format!(
            "- [{}] {}{}\n",
            mem.fact_type, mem.content, confidence_marker
        ));
    }
    context.push_str("\nUse these facts for context. If you learn new facts or corrections from the user, include them in extracted_facts.\n");

    Ok(context)
}

// ─── Health Scores ───────────────────────────────────────────────────────────

pub async fn get_latest_health_score(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<Option<PlantHealthScore>, AppError> {
    let plant_id_str = plant_id.to_string();
    let row = sqlx::query_as!(
        HealthScoreRow,
        r#"SELECT id, plant_id, user_id, score, care_adherence as "care_adherence?: f64", overdue_penalty as "overdue_penalty?: f64", coach_sentiment as "coach_sentiment?: f64", scored_at, created_at
         FROM plant_health_scores WHERE plant_id = ? AND user_id = ? ORDER BY scored_at DESC LIMIT 1"#,
        plant_id_str,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(row.and_then(|r| r.to_health_score().ok()))
}

pub async fn store_health_score(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    score: f64,
    care_adherence: Option<f64>,
    overdue_penalty: Option<f64>,
    coach_sentiment: Option<f64>,
) -> Result<PlantHealthScore, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now().to_rfc3339();
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let id_str = id.to_string();
    let plant_id_str = plant_id.to_string();

    // Upsert: replace today's score if it exists
    sqlx::query!(
        r#"INSERT OR REPLACE INTO plant_health_scores (id, plant_id, user_id, score, care_adherence, overdue_penalty, coach_sentiment, scored_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        id_str,
        plant_id_str,
        user_id,
        score,
        care_adherence,
        overdue_penalty,
        coach_sentiment,
        today,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    get_latest_health_score(pool, plant_id, user_id)
        .await?
        .ok_or(AppError::Internal {
            message: "Failed to store health score".to_string(),
        })
}

// ─── Row types ───────────────────────────────────────────────────────────────

#[derive(Debug)]
struct MemoryRow {
    id: String,
    plant_id: String,
    #[allow(dead_code)]
    user_id: String,
    fact_type: String,
    content: String,
    confidence: f64,
    source: String,
    source_message_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl MemoryRow {
    fn to_memory(self) -> Result<PlantMemory, AppError> {
        Ok(PlantMemory {
            id: Uuid::parse_str(&self.id).map_err(|_| AppError::Internal {
                message: "Invalid UUID".to_string(),
            })?,
            plant_id: Uuid::parse_str(&self.plant_id).map_err(|_| AppError::Internal {
                message: "Invalid plant UUID".to_string(),
            })?,
            fact_type: self.fact_type.parse().map_err(|_| AppError::Internal {
                message: "Invalid fact type".to_string(),
            })?,
            content: self.content,
            confidence: self.confidence,
            source: self.source.parse().map_err(|_| AppError::Internal {
                message: "Invalid source".to_string(),
            })?,
            source_message_id: self.source_message_id,
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|_| AppError::Internal {
                    message: "Invalid datetime".to_string(),
                })?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|_| AppError::Internal {
                    message: "Invalid datetime".to_string(),
                })?
                .with_timezone(&Utc),
        })
    }
}

#[derive(Debug)]
struct HealthScoreRow {
    id: String,
    plant_id: String,
    #[allow(dead_code)]
    user_id: String,
    score: f64,
    care_adherence: Option<f64>,
    overdue_penalty: Option<f64>,
    coach_sentiment: Option<f64>,
    scored_at: String,
    created_at: String,
}

impl HealthScoreRow {
    fn to_health_score(self) -> Result<PlantHealthScore, AppError> {
        Ok(PlantHealthScore {
            id: Uuid::parse_str(&self.id).map_err(|_| AppError::Internal {
                message: "Invalid UUID".to_string(),
            })?,
            plant_id: Uuid::parse_str(&self.plant_id).map_err(|_| AppError::Internal {
                message: "Invalid plant UUID".to_string(),
            })?,
            score: self.score,
            care_adherence: self.care_adherence,
            overdue_penalty: self.overdue_penalty,
            coach_sentiment: self.coach_sentiment,
            scored_at: self.scored_at.parse::<DateTime<Utc>>().unwrap_or_else(|_| {
                // Try as date-only
                chrono::NaiveDate::parse_from_str(&self.scored_at, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    .unwrap_or_default()
            }),
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|_| AppError::Internal {
                    message: "Invalid datetime".to_string(),
                })?
                .with_timezone(&Utc),
        })
    }
}
