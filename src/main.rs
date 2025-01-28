use bytes::Buf;
use futures::TryStreamExt;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};
use tokio::sync::Mutex;
use warp::{
    Filter, Rejection, Reply,
    http::StatusCode,
    multipart::{FormData, Part},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ModMetadata {
    id: String,
    title: String,
    version: String,
    thumbnail: String,
    file_path: String,
}

type DbConnection = Arc<Mutex<Connection>>;

#[tokio::main]
async fn main() {
    let db = Connection::open("mods.db").expect("Failed to open database");
    let db = Arc::new(Mutex::new(db));

    setup_db(db.clone())
        .await
        .expect("Failed to setup database");

    fs::create_dir_all("thumbnails").expect("Failed to create thumbnails directory");
    fs::create_dir_all("mods").expect("Failed to create mods directory");

    let db_filter = warp::any().map(move || db.clone());

    let get_metadata = warp::path("metadata")
        .and(warp::get())
        .and(db_filter.clone())
        .and_then(handle_get_metadata);

    let upload = warp::path("upload")
        .and(warp::post())
        .and(db_filter.clone())
        .and(warp::multipart::form().max_length(1000 * 1024 * 1024))
        .and_then(handle_upload);

    let download = warp::path!("download" / String)
        .and(warp::get())
        .and(db_filter.clone())
        .and_then(handle_download);

    let setup = warp::path("setup")
        .and(warp::get())
        .and(db_filter.clone())
        .and_then(handle_setup);

    let routes = get_metadata.or(upload).or(download).or(setup);

    println!("Server started on localhost:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

async fn setup_db(db: DbConnection) -> Result<(), rusqlite::Error> {
    let conn = db.lock().await;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mods (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            version TEXT NOT NULL,
            thumbnail TEXT NOT NULL,
            file_path TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

async fn handle_get_metadata(db: DbConnection) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;
    let mut stmt = conn
        .prepare("SELECT id, title, version, thumbnail, file_path FROM mods")
        .map_err(|_| warp::reject::custom(DbError))?;

    let mods = stmt
        .query_map([], |row| {
            Ok(ModMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                version: row.get(2)?,
                thumbnail: row.get(3)?,
                file_path: row.get(4)?,
            })
        })
        .map_err(|_| warp::reject::custom(DbError))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| warp::reject::custom(DbError))?;

    Ok(warp::reply::json(&mods))
}

async fn handle_upload(db: DbConnection, mut form: FormData) -> Result<impl Reply, Rejection> {
    let mut mod_metadata = ModMetadata {
        id: String::new(),
        title: String::new(),
        version: String::new(),
        thumbnail: String::new(),
        file_path: String::new(),
    };

    while let Ok(Some(part)) = form.try_next().await {
        if let name = part.name() {
            match name {
                "id" => {
                    let value = read_part_to_string(part)
                        .await
                        .map_err(|_| warp::reject::custom(UploadError))?;
                    mod_metadata.id = value;
                }
                "title" => {
                    let value = read_part_to_string(part)
                        .await
                        .map_err(|_| warp::reject::custom(UploadError))?;
                    mod_metadata.title = value;
                }
                "version" => {
                    let value = read_part_to_string(part)
                        .await
                        .map_err(|_| warp::reject::custom(UploadError))?;
                    mod_metadata.version = value;
                }
                "thumbnail" => {
                    let data = read_part_to_bytes(part)
                        .await
                        .map_err(|_| warp::reject::custom(UploadError))?;
                    let base64_thumbnail = base64::encode(&data);
                    mod_metadata.thumbnail = base64_thumbnail;
                }
                "file" => {
                    let data = read_part_to_bytes(part)
                        .await
                        .map_err(|_| warp::reject::custom(UploadError))?;
                    let file_path = format!("mods/{}.gz", mod_metadata.id);
                    fs::write(&file_path, data).map_err(|_| warp::reject::custom(UploadError))?;
                    mod_metadata.file_path = file_path;
                }
                _ => {}
            }
        }
    }

    let conn = db.lock().await;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM mods WHERE id = ?1)",
            params![mod_metadata.id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        conn.execute(
            "UPDATE mods SET title = ?1, version = ?2, thumbnail = ?3, file_path = ?4 WHERE id = ?5",
            params![
                mod_metadata.title,
                mod_metadata.version,
                mod_metadata.thumbnail,
                mod_metadata.file_path,
                mod_metadata.id
            ],
        ).map_err(|_| warp::reject::custom(DbError))?;
    } else {
        conn.execute(
            "INSERT INTO mods (id, title, version, thumbnail, file_path) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                mod_metadata.id,
                mod_metadata.title,
                mod_metadata.version,
                mod_metadata.thumbnail,
                mod_metadata.file_path
            ],
        ).map_err(|_| warp::reject::custom(DbError))?;
    }

    Ok(StatusCode::OK)
}

async fn read_part_to_string(mut part: Part) -> Result<String, warp::Error> {
    let bytes = read_part_to_bytes(part).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

async fn read_part_to_bytes(mut part: Part) -> Result<Vec<u8>, warp::Error> {
    let mut bytes = Vec::new();
    while let Some(data) = part.data().await {
        let data = data?;
        bytes.extend_from_slice(data.chunk());
    }
    Ok(bytes)
}

async fn handle_download(id: String, db: DbConnection) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;
    let file_path: String = conn
        .query_row(
            "SELECT file_path FROM mods WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| warp::reject::custom(DbError))?;

    let file_data = fs::read(&file_path).map_err(|_| warp::reject::custom(FileError))?;
    Ok(file_data)
}

async fn handle_setup(db: DbConnection) -> Result<impl Reply, Rejection> {
    setup_db(db)
        .await
        .map_err(|_| warp::reject::custom(DbError))?;
    Ok(StatusCode::OK)
}

#[derive(Debug)]
struct DbError;
impl warp::reject::Reject for DbError {}

#[derive(Debug)]
struct UploadError;
impl warp::reject::Reject for UploadError {}

#[derive(Debug)]
struct FileError;
impl warp::reject::Reject for FileError {}
