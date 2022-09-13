use actix_web::http::StatusCode;
use actix_web::ResponseError;
use actix_web::{web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(pool, parameters))]
pub async fn confirm(
    pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>,
) -> Result<HttpResponse, ConfirmError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber token.")?;
    match id {
        None => Err(ConfirmError::UnauthorisedError),
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id)
                .await
                .context("could not confirm subscriber in the database.")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("unauthorised")]
    UnauthorisedError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::routes::error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmError::UnauthorisedError => StatusCode::UNAUTHORIZED,
            ConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "Mark a subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(
    name = "Get a subscriber's ID from a token",
    skip(pool, subscription_token)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id FROM subscription_tokens
        WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}
