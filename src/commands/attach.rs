use crate::client::AtlassianClient;
use owo_colors::OwoColorize;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run(
    client: &AtlassianClient,
    key: String,
    file_path: String,
    message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attaching {} to issue {}...", file_path, key);
    let result = client.attach_to_issue(&key, &file_path).await?;

    let attachments = result
        .as_array()
        .ok_or("Failed to parse attachment response")?;
    let attachment = attachments
        .first()
        .ok_or("No attachment returned in response")?;
    let filename = attachment["filename"].as_str().unwrap_or("unknown");
    let attachment_id = attachment["id"].as_str().ok_or("Attachment has no numeric ID")?;
    let attachment_url = attachment["content"].as_str().ok_or("Attachment has no content URL")?;

    println!(
        "{} {}",
        "Successfully attached:".green().bold(),
        filename.bold()
    );

    if let Some(msg) = message {
        println!("Resolving Media ID and polling for attachment processing...");
        
        let media_id = match client.get_attachment_media_id(attachment_id).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("{} could not resolve Media ID ({}), falling back to external media.", "warning:".yellow().bold(), e);
                // Fallback to external media immediately if we can't get UUID
                client.add_comment_with_external_media(&key, &msg, attachment_url).await?;
                return Ok(());
            }
        };

        let max_retries = 15;
        let mut attempt = 0;
        let mut success = false;

        while attempt < max_retries {
            match client.add_comment_with_attachment(&key, &msg, &media_id).await {
                Ok(_) => {
                    println!("{}", "Successfully added comment with embedded attachment.".green().bold());
                    success = true;
                    break;
                }
                Err(e) if e.contains("ATTACHMENT_VALIDATION_ERROR") => {
                    attempt += 1;
                    if attempt < max_retries {
                        print!(".");
                        use std::io::{self, Write};
                        io::stdout().flush().ok();
                        sleep(Duration::from_secs(2)).await;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "\n{} unexpected error ({}), falling back to external media.",
                        "warning:".yellow().bold(),
                        e
                    );
                    break;
                }
            }
        }

        if !success {
            if attempt >= max_retries {
                println!("\n{} attachment processing timed out.", "warning:".yellow().bold());
            }
            client.add_comment_with_external_media(&key, &msg, attachment_url).await?;
            println!("{}", "Successfully added comment with external media preview.".green().bold());
        }
    }

    Ok(())
}
