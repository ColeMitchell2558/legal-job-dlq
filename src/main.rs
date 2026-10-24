use legal_job_dlq::{should_dead_letter, MatterJob};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let queue = std::env::var("LEGAL_QUEUE").unwrap_or_else(|_| "legal-jobs".into());
    let client = legal_job_dlq::infrai_queue::QueueClient::from_env()?;
    client.create(&queue).await?;

    let job = MatterJob::DeadlineFollowUp { matter_id: "matter-42".into(), deadline: "2026-11-01".into() };
    let payload = json!({"delivery_id": "matter-42-deadline-1", "job": job});
    client.publish(&queue, payload).await?;
    let received = client.consume(&queue).await?;
    println!("consumed: {received}");

    if let Err(error) = should_dead_letter(&job) {
        println!("dead-letter decision: {error}");
    }
    println!("queue={queue}; inspect DLQ with client.dead_letters and restore with client.redrive");
    Ok(())
}
