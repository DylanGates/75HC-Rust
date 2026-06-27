use axum::{extract::State, Json};
use serde_json::json;

use crate::db::{MongoDb, RedisCache};

#[derive(Clone)]
pub struct HealthState {
    pub mongodb: MongoDb,
    pub redis: RedisCache,
}

pub async fn health_check(State(state): State<HealthState>) -> Json<serde_json::Value> {
    let mongodb_status = state.mongodb.health_check().await.is_ok();
    let redis_status = state.redis.health_check().await.is_ok();
    
    let overall_status = mongodb_status && redis_status;
    
    Json(json!({
        "status": if overall_status { "healthy" } else { "unhealthy" },
        "services": {
            "mongodb": if mongodb_status { "up" } else { "down" },
            "redis": if redis_status { "up" } else { "down" }
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}