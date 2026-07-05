use crate::shared_log::log::LogContent;
use crate::shared_log::user_embedding_model::SlimUserEmbeddingInput;
use crate::storage::traits::{StorageEngine, StorageEngineErrors};
use chrono::{DateTime, Utc};
use log::LevelFilter;
use sqlx::migrate::Migrator;
use sqlx::{Pool, Postgres, Row, pool::PoolConnection, postgres::PgPoolOptions};
use uuid::Uuid;

const MAX_CONNECTIONS: u8 = 5;
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct PostgresStorage {
    connection_pool: Pool<Postgres>,
}

impl PostgresStorage {
    fn new(connection_string: String) -> Self {
        let connection_pool: Pool<Postgres> = match PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS as u32)
            .acquire_slow_level(LevelFilter::Error)
            .connect_lazy(connection_string.as_str())
        {
            Err(_) => panic!("unable to initialize connection pool for postgres memory backend"),
            Ok(pool) => pool,
        };

        PostgresStorage { connection_pool }
    }

    async fn run_migration(&self) -> Result<(), StorageEngineErrors> {
        match MIGRATOR.run(&self.connection_pool).await {
            Ok(_) => Ok(()),
            Err(err) => Err(StorageEngineErrors::UnableToExecuteMigrations(format!(
                "unable to execute migrations: {}",
                err
            ))),
        }
    }

    async fn acquire_connection(&self) -> Result<PoolConnection<Postgres>, StorageEngineErrors> {
        match self.connection_pool.acquire().await {
            Ok(conn) => Ok(conn),
            Err(err) => Err(StorageEngineErrors::UnableToAcquireConnection(format!(
                "unable to acquire connection from Postgres connection pool: {}",
                err
            ))),
        }
    }
}

#[async_trait::async_trait]
impl StorageEngine for PostgresStorage {
    async fn load_storage(config: crate::global_config::GlobalConfig) -> Self {
        let storage_engine = PostgresStorage::new(config.database_connection_string);

        match storage_engine.run_migration().await {
            Ok(_) => storage_engine,
            Err(err) => panic!("{}", err),
        }
    }

    async fn store_event(
        &self,
        event: &dyn crate::shared_log::traits::Event,
    ) -> Result<(), StorageEngineErrors> {
        let timestamp: isize = event.get_timestamp();
        let content: String = event.get_content();
        let event_type: String = event.get_event_type().to_string();
        let id: Uuid = event.get_id();

        let datetime: DateTime<Utc> = DateTime::from_timestamp(timestamp as i64, 0)
            .ok_or(StorageEngineErrors::InvalidTimestamp(timestamp))?;

        let mut connection = self.acquire_connection().await?;

        let query = sqlx::query!(
            r#"
            INSERT INTO LogEvent (id, content, timestamp, event_type)
            VALUES ($1, $2, $3, $4)
            "#,
            id,
            content,
            datetime,
            event_type
        );

        query
            .execute(&mut *connection)
            .await
            .map_err(|e| StorageEngineErrors::DatabaseError(Box::new(e)))?;

        Ok(())
    }

    async fn store_user_input_embedding(
        &self,
        user_event_id: uuid::Uuid,
        embedding: &[f32],
    ) -> Result<(), StorageEngineErrors> {
        let mut connection = self.acquire_connection().await?;

        let embedding_str: String = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        sqlx::query(
            "INSERT INTO UserInputEmbedding (user_event_id, embedding) VALUES ($1, $2::vector)",
        )
        .bind(user_event_id)
        .bind(&embedding_str)
        .execute(&mut *connection)
        .await
        .map_err(|e| StorageEngineErrors::DatabaseError(Box::new(e)))?;

        Ok(())
    }

    async fn store_cached_agent_response(
        &self,
        log_event_id: uuid::Uuid,
        user_query_id: uuid::Uuid,
    ) -> Result<(), StorageEngineErrors> {
        let mut connection = self.acquire_connection().await?;

        let query = sqlx::query!(
            r#"
            INSERT INTO CachedAgentResponse (log_event_id, user_query_id)
            VALUES ($1, $2)
            "#,
            log_event_id,
            user_query_id,
        );

        query
            .execute(&mut *connection)
            .await
            .map_err(|e| StorageEngineErrors::DatabaseError(Box::new(e)))?;

        Ok(())
    }

    async fn get_similar_user_input_embedding(
        &self,
        embedding: Vec<f32>,
    ) -> Result<SlimUserEmbeddingInput, StorageEngineErrors> {
        let embedding_as_vec: String = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let query = sqlx::query(
            r#"
            WITH closest_matches AS (
                SELECT user_event_id FROM UserInputEmbedding ORDER BY embedding <=> $1::vector ASC LIMIT 1
            )
            SELECT * FROM closest_matches ORDER BY id DESC;
            "#,
        )
        .bind(&embedding_as_vec);

        let mut connection = self.acquire_connection().await?;
        let rows = query
            .fetch_all(&mut *connection)
            .await
            .map_err(|error| StorageEngineErrors::DatabaseError(Box::new(error)))?;

        let mut input_reference: Vec<SlimUserEmbeddingInput> = rows
            .iter()
            .map(|row| {
                let user_event_id: uuid::Uuid = row.try_get("user_event_id").map_err(|_| {
                    StorageEngineErrors::NoDataForField(
                        "no user event id reference found".to_string(),
                    )
                })?;

                Ok(SlimUserEmbeddingInput { user_event_id })
            })
            .collect::<Result<Vec<SlimUserEmbeddingInput>, StorageEngineErrors>>()?
            .into_iter()
            .collect();

        match input_reference.pop() {
            Some(result) => Ok(result),
            None => Err(StorageEngineErrors::NoDataForField(String::from(
                "No similar user embeddings found.",
            ))),
        }
    }

    async fn get_agent_output_for_user_input(
        &self,
        user_input_id: uuid::Uuid,
    ) -> Result<LogContent, StorageEngineErrors> {
        let query1 = sqlx::query(
            r#"
            SELECT log_event_id FROM CachedAgentResponse where user_query_id = $1;
            "#,
        )
        .bind(user_input_id);

        let mut connection = self.acquire_connection().await?;
        let rows = query1
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| StorageEngineErrors::DatabaseError(Box::new(error)))?;

        let event_id: uuid::Uuid;
        if let Ok(id) = rows.try_get("log_event_id") {
            event_id = id;
        } else {
            return Err(StorageEngineErrors::NoDataForField(format!(
                "No log event id found for user event id {}",
                user_input_id
            )));
        }

        let query2 = sqlx::query(
            r#"
            SELECT content FROM LogEvent WHERE id=$1
            "#,
        )
        .bind(event_id);
        let row = query2
            .fetch_one(&mut *connection)
            .await
            .map_err(|err| StorageEngineErrors::DatabaseError(Box::new(err)))?;

        if let Ok(content) = row.try_get("content") {
            return Ok(LogContent::new(content));
        };

        Err(StorageEngineErrors::NoDataForField(format!(
            "no content found for agent event {}",
            event_id
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::global_config::GlobalConfig;
    use crate::shared_log::log::{LogContent, LogEvent};
    use crate::shared_log::traits::{Event, EventType};
    use testcontainers::core::{IntoContainerPort, WaitFor};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::{ContainerAsync, GenericImage, ImageExt};

    async fn setup_postgres_container() -> (ContainerAsync<GenericImage>, String) {
        let container = GenericImage::new("pgvector/pgvector", "pg18")
            .with_exposed_port(5432.tcp())
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_USER", "testuser")
            .with_env_var("POSTGRES_PASSWORD", "testpassword")
            .with_env_var("POSTGRES_DB", "testdatabase")
            .start()
            .await
            .expect("Failed to start Postgres container");

        let port = container
            .get_host_port_ipv4(5432.tcp())
            .await
            .expect("Failed to get host port");

        let connection_string =
            format!("postgres://testuser:testpassword@127.0.0.1:{port}/testdatabase");

        // Wait a moment for the pool to stabilize
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        (container, connection_string)
    }

    #[tokio::test]
    async fn test_postgres_storage_get_similar_user_input_embedding() {
        let (_container, conn_string) = setup_postgres_container().await;
        let storage = PostgresStorage::new(conn_string.clone());
        storage.run_migration().await.unwrap();

        let timestamp = "1000000".to_string();
        let user_event = LogEvent::new(
            EventType::UserInput,
            "test user input".to_string(),
            timestamp.clone(),
        );

        storage.store_event(&user_event).await.unwrap();

        let embedding: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
        let user_event_id = user_event.get_id();
        storage
            .store_user_input_embedding(user_event_id, &embedding)
            .await
            .unwrap();

        let similar_embedding: Vec<f32> = (0..384).map(|i| (i as f32 + 0.5) / 384.0).collect();
        let result = storage
            .get_similar_user_input_embedding(similar_embedding)
            .await
            .unwrap();

        assert_eq!(result.user_event_id, user_event_id);
    }

    #[tokio::test]
    async fn test_postgres_storage_get_agent_output_for_user_input() {
        let (_container, conn_string) = setup_postgres_container().await;
        let storage = PostgresStorage::new(conn_string.clone());
        storage.run_migration().await.unwrap();

        let user_timestamp = "1000000".to_string();
        let user_event = LogEvent::new(
            EventType::UserInput,
            "test user input".to_string(),
            user_timestamp.clone(),
        );

        storage.store_event(&user_event).await.unwrap();
        let user_event_id = user_event.get_id();

        let agent_timestamp = "2000000".to_string();
        let agent_event = LogEvent::new(
            EventType::AgentOutput,
            "test agent output".to_string(),
            agent_timestamp.clone(),
        );

        storage.store_event(&agent_event).await.unwrap();
        let agent_event_id = agent_event.get_id();

        storage
            .store_cached_agent_response(agent_event_id, user_event_id)
            .await
            .unwrap();

        let log_content = storage
            .get_agent_output_for_user_input(user_event_id)
            .await
            .unwrap();

        assert_eq!(log_content.get_content(), "test agent output");
    }
}
