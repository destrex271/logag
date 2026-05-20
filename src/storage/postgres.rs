use crate::shared_log::traits::Event;
use crate::storage::traits::{
    StorageEngine,
    StorageEngineErrors,
};
use chrono::{DateTime, Utc};
use log::LevelFilter;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, pool::PoolConnection};
use sqlx::migrate::Migrator;
use uuid::Uuid;

const MAX_CONNECTIONS: u8 = 5;
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub struct PostgresStorage {
    connection_string: String,
    connection_pool: Pool<Postgres>
}

impl PostgresStorage {
    fn new(connection_string: String) -> Self{
        let connection_pool: Pool<Postgres> = match PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS as u32)
            .acquire_slow_level(LevelFilter::Error)
            .connect_lazy(connection_string.as_str()){
                Err(_) => panic!("unable to initialize connection pool for postgres memory backend"),
                Ok(pool) => pool,
            };

        PostgresStorage { 
            connection_string: connection_string,
            connection_pool: connection_pool,
        }
    }

    async fn run_migration(&self) -> Result<(), StorageEngineErrors>{
        match MIGRATOR.run(&self.connection_pool).await{
            Ok(_) => Ok(()),
            Err(err) => Err(
                StorageEngineErrors::UnableToExecuteMigrations(
                    format!("unable to execute migrations: {}", err)
                )
            )
        }
    }

    async fn acquire_connection(&self) -> Result<PoolConnection<Postgres>, StorageEngineErrors>{
        match self.connection_pool.acquire().await{
            Ok(conn) => Ok(conn),
            Err(err) => Err(
                StorageEngineErrors::UnableToAcquireConnection(
                    format!("unable to acquire connection from Postgres connection pool: {}", err)
                )
            ),
        }
    }
}

impl StorageEngine for PostgresStorage{
    async fn load_storage(config: crate::global_config::GlobalConfig) -> Self {
        let storage_engine = PostgresStorage::new(
            config.database_connection_string,
        );

        match storage_engine.run_migration().await{
            Ok(_) => storage_engine,
            Err(err) => panic!("{}", err)
        }
    }

    async fn store_event(&self, event: &dyn crate::shared_log::traits::Event) -> Result<(), StorageEngineErrors> {
        let timestamp: isize = event.get_timestamp();
        let content: String = event.get_content();
        let event_type: String = event.get_event_type().to_string();
        let id: Uuid = event.get_id();

        let datetime: DateTime<Utc>= DateTime::from_timestamp(timestamp as i64, 0).ok_or(
            StorageEngineErrors::InvalidTimestamp(timestamp)
        )?;

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

        query.execute(&mut *connection).await.unwrap();

        Ok(())
    }

    async fn get_events<T, F>(
        &self,
        from_timestamp: isize,
        to_timestamp: isize,
        factory_fn: F,
    ) -> Result<
        Vec<T>,
        StorageEngineErrors
    > where 
        T: Event,
        F: Fn(uuid::Uuid, String, isize, String) -> T
    {
        
        let begin: DateTime<Utc> = DateTime::from_timestamp(from_timestamp as i64, 0).ok_or(
            StorageEngineErrors::InvalidTimestamp(from_timestamp),
        )?;

        let end: DateTime<Utc> = DateTime::from_timestamp(to_timestamp as i64, 0).ok_or(
            StorageEngineErrors::InvalidTimestamp(to_timestamp),
        )?;

        let query = sqlx::query!(
            r#"
            SELECT * FROM LogEvent WHERE timestamp BETWEEN $1 AND $2
            "#,
            begin,
            end
        );

        let mut connection = self.acquire_connection().await?;

        
        let rows = query.fetch_all(&mut *connection).await.map_err(|error|{
            return StorageEngineErrors::DatabaseError(Box::new(error))
        })?;

        let events: Vec<T> = rows
            .into_iter()
            .map(|row| {
                let content = match row.content {
                    Some(content) => content,
                    None => return Err(StorageEngineErrors::NoDataForField(format!("content not found"))),
                };

                let timestamp_epoch: isize = match row.timestamp {
                    Some(timestamp) => timestamp.timestamp() as isize, // No 'return' keyword here!
                    None => return Err(StorageEngineErrors::NoDataForField(format!("no timezone data found"))),
                };

                let product = factory_fn(
                    row.id,
                    content,
                    timestamp_epoch,
                    row.event_type,
                );

                Ok(product)
            })
            .collect::<Result<Vec<T>, StorageEngineErrors>>()?;


        Ok(events)
    }
}
