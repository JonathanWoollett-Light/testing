use bson::doc;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::info;

struct Mongodb {
    port: u16,
    db_path: PathBuf,
    mongodb: std::process::Child,
}
async fn download_mongodb() -> PathBuf {
    #[cfg(target_os = "windows")]
    const URL: &str = "https://fastdl.mongodb.org/windows/mongodb-windows-x86_64-7.0.14.zip";

    const PATH: &str = "mongodb-win32-x86_64-windows-7.0.14/bin/mongod.exe";
    info!("sent mongodb get request");

    let mut zip_path = std::env::temp_dir();
    zip_path.push("mongodb.zip");

    if !std::fs::exists(&zip_path).unwrap() {
        let bytes = reqwest::get(URL).await.unwrap().bytes().await.unwrap();
        let mut zip = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&zip_path)
            .unwrap();
        info!("writing to zip archive");
        zip.write_all(&bytes).unwrap();
    }

    info!("extracing zip archive");
    let zip = OpenOptions::new().read(true).open(zip_path).unwrap();
    let base = std::env::temp_dir();
    let mut archive = base.clone();
    archive.push(PATH);
    if !std::fs::exists(&archive).unwrap() {
        zip_extract::extract(zip, &base, false).unwrap();
    }

    info!("returning mongodb path");
    archive
}

impl Mongodb {
    async fn new() -> Self {
        static BINARY: OnceCell<PathBuf> = OnceCell::const_new();
        info!("getting mongodb binary");
        let binary = BINARY.get_or_init(download_mongodb).await;
        info!("mongodb binary: {}", binary.display());
        static PORT: AtomicU16 = AtomicU16::new(27018);
        let port = PORT.fetch_add(1, Ordering::SeqCst);
        info!("mongodb URI: mongodb://localhost:{port}/");
        let mut db_path = std::env::temp_dir();
        db_path.set_file_name(format!("mongodb-{}", uuid::Uuid::new_v4()));
        info!("db_path: {}", db_path.display());
        fs::create_dir(&db_path).unwrap();
        let mongodb = Command::new(binary)
            .args([
                "--port",
                &port.to_string(),
                "--dbpath",
                &db_path.display().to_string(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Mongodb {
            port,
            db_path,
            mongodb,
        }
    }
    fn client(this: Arc<Self>) -> MongodbClient {
        let host = mongodb::options::ServerAddress::Tcp {
            host: Ipv4Addr::LOCALHOST.to_string(),
            port: Some(this.port),
        };
        let opts = mongodb::options::ClientOptions::builder()
            .hosts(vec![host])
            .direct_connection(true)
            .build();
        let client = mongodb::Client::with_options(opts).unwrap();
        MongodbClient {
            client,
            server: this,
        }
    }
}
impl Drop for Mongodb {
    fn drop(&mut self) {
        self.mongodb.kill().unwrap();
        let _status = self.mongodb.wait().unwrap();
        fs::remove_dir_all(&self.db_path).unwrap();
    }
}
struct MongodbClient {
    client: mongodb::Client,
    server: Arc<Mongodb>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub _id: ObjectId,
    pub ids: BTreeMap<ObjectId, String>
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mongodb = Arc::new(Mongodb::new().await);
    let client = Mongodb::client(mongodb);
    let collection = client
        .client
        .database("testdb")
        .collection("testcollection");
    let id = ObjectId::new();
    println!("id: {id:?}");
    let user = DatabaseUser { _id: id.clone(), ids: [(ObjectId::new(),String::new()),(ObjectId::new(),String::new())].into_iter().collect() };
    println!("user: {user:?}");
    collection
        .insert_one(user)
        .await
        .unwrap();
    let user = collection.find_one(doc! { "_id": id }).await.unwrap();
    println!("user: {user:?}");
}
