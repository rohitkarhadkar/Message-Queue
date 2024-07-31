use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use tokio_postgres::{NoTls};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::time::{self, Duration};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
//use chrono::{Duration as ChronoDuration, Utc};
//use postgres_types::{FromSql, Type};



#[derive(Serialize, Deserialize)]
struct ReviewRequest {
    user_id: i32,
    message: String,
    timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct ReviewRequestInvalid {
    user_id: i32,
    message: String,
    timestamp: u64,
    invalidated_at: u64,
}

async fn push_review_request(
    db_pool: web::Data<Arc<tokio_postgres::Client>>,
    review_request: web::Json<ReviewRequest>
) -> impl Responder {
    let client = db_pool.get_ref();

     let mut review_request = review_request.into_inner();
     review_request.timestamp = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
     let message = serde_json::to_string(&review_request).unwrap();
     let query = format!("SELECT * FROM pgmq.send('review_queue', '{}')", message);
     client.query(&query, &[]).await.unwrap();

     HttpResponse::Ok().body("Review request sent")
}

async fn pop_review_request(
    db_pool: web::Data<Arc<tokio_postgres::Client>>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let client = db_pool.get_ref();
    let query = "SELECT * FROM pgmq.q_review_queue WHERE message->>'user_id' = $1";
    let rows = client.query(query, &[&user_id.to_string()]).await.unwrap();
    if rows.is_empty() {
        return HttpResponse::NotFound().body("No review request found for the user");
    }
    let msg_id: i64 = rows[0].get("msg_id");
    let query = "SELECT pgmq.delete('review_queue', $1::bigint)";
    client.query(query, &[&msg_id]).await.unwrap();
    HttpResponse::Ok().body("Review request removed from queue")
}

async fn cron_job(db_pool: Arc<tokio_postgres::Client>) {
    let mut interval = time::interval(Duration::from_millis(10000));

    loop {
        interval.tick().await;
        println!("Running cron job iteration");
        let client = db_pool.clone();
        let sys_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let query = "SELECT msg_id, message::text FROM pgmq.q_review_queue";
        let rows = match client.query(query, &[]).await {
            Ok(rows) => rows,
            Err(err) => {
                eprintln!("Error querying the database: {}", err);
                continue;
            }
        };
        for row in rows {
            // Get the message as a string
            let message: String = row.get("message");

            // Parse the message string as serde_json::Value
            let message_value: serde_json::Value = serde_json::from_str(&message).unwrap();

            // Deserialize the JSON value into ReviewRequest
            let review_request: ReviewRequest = serde_json::from_value(message_value).unwrap();

            let msg_id: i64 = row.get("msg_id");

            let elapsed_seconds = sys_time - review_request.timestamp.unwrap();
            if elapsed_seconds > 30 {
                let invalid_request = ReviewRequestInvalid {
                    user_id: review_request.user_id,
                    message: review_request.message.clone(),
                    timestamp: review_request.timestamp.unwrap(),
                    invalidated_at: sys_time as u64,
                };
                let invalid_message = serde_json::to_string(&invalid_request).unwrap();
                let insert_query = format!("SELECT * FROM pgmq.send('invalid_review_queue', '{}')", invalid_message);
                if let Err(err) = client.query(&insert_query, &[]).await {
                    eprintln!("Error inserting invalid message: {}", err);
                }

                let delete_query = "SELECT pgmq.delete('review_queue', $1::bigint)";
                if let Err(err) = client.query(delete_query, &[&msg_id]).await {
                    eprintln!("Error deleting message: {}", err);
                }
            }
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Establish database connection
    let (client, connection) =
        tokio_postgres::connect("host=localhost port=5433 user=postgres password=postgres dbname=postgres", NoTls)
            .await
            .expect("Failed to connect to database");

    // Spawn the connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Create an Arc to share the client across threads
    let client = Arc::new(client);

    // Clone the client for the cron job
        let cron_client = client.clone();
        tokio::spawn(async move {
            cron_job(cron_client).await;
        });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone())) // Share database client across threads
            .route("/admin/push_review_request", web::post().to(push_review_request))
            .route("/user/pop_review_request/{user_id}", web::post().to(pop_review_request))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
