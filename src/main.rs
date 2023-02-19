mod entities;

use dotenv_codegen::dotenv;
use futures::executor::block_on;
use sea_orm::{*};
use entities::{prelude::*, *};

const DATABASE_URL:&str = " mysql://root:s2717244@localhost:3306";
const DB_NAME:&str = dotenv!("DB_NAME");

async fn run() -> Result<(), DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

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

    // Create
    let happy_bakery = bakery::ActiveModel {
        name: ActiveValue::Set("Happy Bakery".to_string()),
        profit_margin: ActiveValue::Set(0.0),
        ..Default::default()
    };
    let res = Bakery::insert(happy_bakery).exec(db).await?;

    // Update
    let sad_bakery = bakery::ActiveModel {
        id: ActiveValue::Set(res.last_insert_id),
        name: ActiveValue::Set("Sad Bakery".to_string()),
        profit_margin: NotSet,
    };
    sad_bakery.update(db).await?;

    let john = chef::ActiveModel {
        name: ActiveValue::Set("John".to_string()),
        bakery_id: ActiveValue::Set(1), // a foreign key
        ..Default::default()
    };
    Chef::insert(john).exec(db).await?;

    // Read
    let bakeries:Vec<bakery::Model> = Bakery::find().all(db).await?;
    println!("Bakeries: {:?}", bakeries);

    let sad_bakery: Option<bakery::Model> = Bakery::find_by_id(1).one(db).await?;
    assert_eq!(sad_bakery.unwrap().name, "Sad Bakery");

    let sad_bakery: Option<bakery::Model> = Bakery::find()
        .filter(bakery::Column::Name.eq("Sad Bakery"))
        .one(db)
        .await?;
    assert_eq!(sad_bakery.unwrap().id, 1);

    // Delete
    let john = chef::ActiveModel {
        id: ActiveValue::Set(1), // The primary key must be set
        ..Default::default()
    };
    john.delete(db).await?;

    let sad_bakery = bakery::ActiveModel {
        id: ActiveValue::Set(1), // The primary key must be set
        ..Default::default()
    };
    sad_bakery.delete(db).await?;

    Ok(())
}

fn main() {
    if let Err(err) = block_on(run()) {
        panic!("{}", err);
    }
}
