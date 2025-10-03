use crate::cup;
use crate::discord::{Embed, Thumbnail, WebhookPayload, send_webhook};
use anyhow::{Result, bail};
use cup_notifier::Config;
use futures::TryFutureExt;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{error, info};

pub async fn start_scheduler(config: &Config) -> Result<()> {
    let scheduler = JobScheduler::new().await?;
    let config_clone = config.clone();
    let job_result = Job::new_async(config_clone.cron.clone(), move |_, _| {
        Box::pin({
            let value = config_clone.clone();
            async move {
                info!("Started job");

                match cup::fetch_fresh_data(&value)
                    .and_then(|root| handle_images(root, &value))
                    .await
                {
                    Ok(()) => info!("Finished job"),
                    Err(err) => error!("Error job {:#?}", err),
                }
            }
        })
    });
    let job = match job_result {
        Ok(job) => job,
        Err(JobSchedulerError::ParseSchedule) => bail!("Invalid cron pattern"),
        Err(err) => bail!(err),
    };

    info!("Scheduling recurring job by cron pattern: {}", config.cron);
    scheduler.add(job).await?;
    scheduler.shutdown_on_ctrl_c();
    scheduler.start().await?;

    tokio::signal::ctrl_c().await?;

    info!("Exiting...");

    Ok(())
}

async fn handle_images(root: cup::Root, config: &Config) -> Result<()> {
    let used_images = root
        .images
        .iter()
        .filter(|img| img.in_use)
        .filter(|img| img.result.has_update)
        .collect::<Vec<_>>();

    info!("Found {} outdated images", used_images.len());

    let embeds = used_images.iter().map(|image| {
        let name = image.parts.repository.split('/').next_back().unwrap();
        let description = match image.result.info.as_ref().unwrap() {
            cup::Info::Version {
                current_version,
                new_tag: _,
                new_version,
                version_update_type: _,
            } => format!(
                "Image {name} running with version {current_version} can be updated to {new_version}",
            ),
            cup::Info::Digest {
                local_digests,
                remote_digest,
            } => format!(
                "Image {} running with digest {} can be updated to {}",
                name,
                local_digests.first().unwrap(),
                remote_digest,
            ),
        };

        Embed {
            title: format!("New version of {}", image.parts.repository),
            color: 2_326_507,
            description,
            url: image.url.clone(),
            thumbnail: Thumbnail {
                url: format!("https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/{name}.png"),
            },
        }
    });

    info!("Sending notifications...");

    for batch in embeds.collect::<Vec<_>>().chunks(10) {
        send_webhook(
            &config.webhook_url,
            WebhookPayload {
                embeds: batch.to_vec(),
            },
        )
        .await?;
    }

    Ok(())
}
