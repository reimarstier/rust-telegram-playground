use std::error::Error;
use std::time::Duration;

use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::bot::core::bot_config::BotConfig;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

fn run_migrations(connection: &mut SqliteConnection) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
            .map_err(diesel::r2d2::Error::QueryError)
    }
}

#[derive(Debug, Clone)]
pub struct MyDatabaseConnection {
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl MyDatabaseConnection {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let bot_config = BotConfig::new()?;
        let database_exists = bot_config.storage.database_path().exists();
        let database_url = bot_config.storage.database_url()?;
        tracing::info!("Opening database at url={}", database_url);

        // https://stackoverflow.com/questions/57123453/how-to-use-diesel-with-sqlite-connections-and-avoid-database-is-locked-type-of
        let pool = Pool::builder()
            .max_size(1)  // Only one connection at a time
            .connection_customizer(Box::new(ConnectionOptions {
                enable_wal: true,
                enable_foreign_keys: true,
                busy_timeout: Some(Duration::from_secs(30)),
            }))
            .build(ConnectionManager::<SqliteConnection>::new(database_url))?;

        if !database_exists {
            let mut connection = pool.get()?;
            log::info!("Running migrations for new database ...");
            run_migrations(&mut connection).expect("Failed to run migrations!");
        }

        Ok(Self {
            pool
        })
    }

    pub async fn get(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, r2d2::Error> {
        self.pool.get()
    }
}
