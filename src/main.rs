use bson::doc;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct TestDoc {
    _id: ObjectId,
    name: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port = 27018;
    info!("starting mongodb");
    let mut mongodb = Command::new("docker")
        .args([
            "run",
            "-p",
            &format!("{port}:{port}"),
            "mongodb/mongodb-atlas-local",
        ])
        .stdout(Stdio::null()) // Uncomment these lines to see mongodb container output.
        .stderr(Stdio::null()) // Uncomment these lines to see mongodb container output.
        .spawn()
        .unwrap();
    sleep(Duration::from_secs(3)).await;
    info!("started mongodb");

    let host = mongodb::options::ServerAddress::Tcp {
        host: Ipv4Addr::LOCALHOST.to_string(),
        port: Some(port),
    };
    let opts = mongodb::options::ClientOptions::builder()
        .hosts(vec![host])
        .direct_connection(true)
        .build();
    info!("connecting client");
    let client = mongodb::Client::with_options(opts).unwrap();
    let db = client.database("test");
    let collection = db.collection::<TestDoc>("test");
    let id = ObjectId::new();
    collection
        .insert_one(TestDoc {
            _id: id.clone(),
            name: "test".to_string(),
        })
        .await
        .unwrap();
    let doc = collection
        .find_one(doc! { "_id": id })
        .await
        .unwrap()
        .unwrap();
    info!("doc: {doc:?}");

    let kill_result = mongodb.kill().await;
    info!("Mongodb kill_result: {kill_result:?}");
    kill_result.unwrap();
    let exit_status = mongodb.wait().await;
    info!("Mongodb exit_status: {exit_status:?}");
    exit_status.unwrap();
}
