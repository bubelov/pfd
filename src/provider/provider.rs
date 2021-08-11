use anyhow::Result;
use chrono::Utc;
use cron::Schedule;
use futures::join;
use std::str::FromStr;
use tokio::time::sleep;
use tracing::warn;

#[rocket::async_trait]
pub trait Provider {
    fn name(&self) -> String;

    fn fiat_sync_enabled(&self) -> bool;

    fn fiat_sync_schedule(&self) -> String;

    async fn sync_fiat(&self) -> Result<()>;

    fn crypto_sync_enabled(&self) -> bool;

    fn crypto_sync_schedule(&self) -> String;

    async fn sync_crypto(&self) -> Result<()>;

    async fn schedule(&self) -> Result<()> {
        let (_, _) = join!(self.schedule_fiat(), self.schedule_crypto());
        Ok(())
    }

    async fn schedule_fiat(&self) -> Result<()> {
        if !self.fiat_sync_enabled() {
            return Ok(());
        }

        warn!(provider = %self.name(), "Scheduling fiat sync...");
        let schedule = Schedule::from_str(&self.fiat_sync_schedule())?;

        for next_sync in schedule.upcoming(Utc) {
            warn!(provider = %self.name(), %next_sync, "Got next sync date");
            let time_to_next_sync = next_sync.signed_duration_since(Utc::now());
            if time_to_next_sync.num_nanoseconds().unwrap() < 0 {
                warn!("Skipping next sync because the old one didn't finish in time");
                continue;
            }
            let time_to_next_sync = time_to_next_sync.to_std().unwrap();
            warn!(
                provider = %self.name(),
                secs_to_next_sync = time_to_next_sync.as_secs(),
                "Going to sleep till next sync"
            );
            sleep(time_to_next_sync).await;
            warn!(provider = %self.name(), "Syncing...");
            self.sync_fiat().await.unwrap();
        }

        Ok(())
    }

    async fn schedule_crypto(&self) -> Result<()> {
        if !self.crypto_sync_enabled() {
            return Ok(());
        }

        warn!(provider = %self.name(), "Scheduling crypto sync...");
        let schedule = Schedule::from_str(&self.crypto_sync_schedule())?;

        for next_sync in schedule.upcoming(Utc) {
            warn!(provider = %self.name(), %next_sync, "Got next sync date");
            let time_to_next_sync = next_sync.signed_duration_since(Utc::now());
            if time_to_next_sync.num_nanoseconds().unwrap() < 0 {
                warn!("Skipping next sync because the old one didn't finish in time");
                continue;
            }
            let time_to_next_sync = time_to_next_sync.to_std().unwrap();
            warn!(
                provider = %self.name(),
                secs_to_next_sync = time_to_next_sync.as_secs(),
                "Going to sleep till next sync"
            );
            sleep(time_to_next_sync).await;
            warn!(provider = %self.name(), "Syncing...");
            self.sync_crypto().await.unwrap();
        }

        Ok(())
    }

    async fn sync(&self) -> Result<()> {
        if self.fiat_sync_enabled() {
            self.sync_fiat().await?
        }

        if self.crypto_sync_enabled() {
            self.sync_crypto().await?
        }

        Ok(())
    }
}
