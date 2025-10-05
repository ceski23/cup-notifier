use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cup::{self, Image};
use crate::discord::{Embed, Thumbnail, WebhookPayload, send_webhook};
use anyhow::{Result, bail};
use cup_notifier::Config;
use futures::TryFutureExt;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{error, info};

pub async fn start_scheduler(config: &Config) -> Result<()> {
    let notifications_cache = Arc::new(Mutex::new(HashSet::<(String, String)>::new()));
    let scheduler = JobScheduler::new().await?;
    let config_clone = config.clone();
    let cache_for_job = notifications_cache.clone();

    let job_result = Job::new_async(format!("0 {}", config_clone.cron), move |_, _| {
        Box::pin({
            let value = config_clone.clone();
            let cache = cache_for_job.clone();
            async move {
                info!("Started job");

                match cup::fetch_fresh_data(&value)
                    .and_then(|root| handle_images(root, &value, cache))
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

fn create_pair(image: &Image) -> (String, String) {
    match image.result.info.as_ref().unwrap() {
        cup::Info::Version {
            current_version: _,
            new_tag: _,
            new_version,
            version_update_type: _,
        } => (image.parts.repository.clone(), new_version.to_owned()),
        cup::Info::Digest {
            local_digests: _,
            remote_digest,
        } => (image.parts.repository.clone(), remote_digest.to_owned()),
    }
}

async fn handle_images(
    root: cup::Root,
    config: &Config,
    cache: Arc<Mutex<HashSet<(String, String)>>>,
) -> Result<()> {
    let used_images = root
        .images
        .iter()
        .filter(|img| img.in_use)
        .filter(|img| img.result.has_update)
        .collect::<Vec<_>>();

    info!("Found {} outdated images", used_images.len());

    let not_notified = {
        let guard = cache.lock().await;

        used_images
            .iter()
            .filter_map(|img_ref| {
                let pair = create_pair(img_ref);
                if guard.contains(&pair) {
                    None
                } else {
                    Some((*img_ref, pair))
                }
            })
            .collect::<Vec<_>>()
    };

    if not_notified.is_empty() {
        info!("No images updates to notify");
        return Ok(());
    }

    info!(
        "Images to send notifications for: {:?}",
        not_notified
            .iter()
            .map(|img| img.1.clone())
            .collect::<Vec<_>>()
    );

    let embeds = not_notified.iter().map(|(image, _pair)| {
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

    let mut guard = cache.lock().await;
    for (_image, pair) in not_notified {
        guard.insert(pair);
    }

    println!("{guard:?}");

    Ok(())
}
