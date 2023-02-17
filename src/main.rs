use dotenv_codegen::dotenv;
use futures::executor::block_on;
use sea_orm::{ConnectionTrait, Database, DbBackend, DbErr, Statement};

const DATABASE_URL:&str = dotenv!("DATABASE_URL");
const DB_NAME:&str = dotenv!("DB_NAME");

async fn run() -> Result<(), DbErr> {
    let db = sea_orm::Database::connect(DATABASE_URL).await?;

    let db = &match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS {}", DB_NAME),
            )).await?;

            let url= format!("{}/{}", DATABASE_URL, DB_NAME);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF NOT EXISTS '{}';", DB_NAME),
            )).await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE {}", DB_NAME),
            )).await?;

            let url= format!("{}/{}", DATABASE_URL, DB_NAME);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    Ok(())
}

fn main() {
    if let Err(err) = block_on(run()) {
        panic!("{}", err);
    }
}
