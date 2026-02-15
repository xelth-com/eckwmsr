use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::time::sleep;
use tracing::info;

use super::delivery::{DeliveryProvider, DeliveryRequest, DeliveryResponse};
use crate::utils::webdriver::ensure_chromedriver;

pub struct OpalProvider {
    username: String,
    password: String,
    url: String,
}

impl OpalProvider {
    pub fn new(username: String, password: String, url: String) -> Self {
        let url = if url.is_empty() {
            "https://opal-kurier.de".to_string()
        } else {
            url
        };
        Self {
            username,
            password,
            url,
        }
    }

    async fn setup_driver(&self) -> anyhow::Result<WebDriver> {
        ensure_chromedriver().await?;

        let mut caps = DesiredCapabilities::chrome();
        caps.add_arg("--disable-infobars")?;
        caps.add_arg("--headless=new")?;
        caps.add_arg("--window-size=1920,1080")?;
        caps.add_arg("--disable-dev-shm-usage")?;
        caps.add_arg("--no-sandbox")?;

        WebDriver::new("http://localhost:9515", caps)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn login(&self, driver: &WebDriver) -> anyhow::Result<()> {
        info!("OPAL: Navigating to login/frameset...");
        driver.goto(&self.url).await?;
        sleep(Duration::from_secs(3)).await;

        // Check if we need to login (if login form exists)
        if let Ok(user_field) = driver
            .query(By::Css("input[name='username'], input[type='email']"))
            .first()
            .await
        {
            if user_field.is_displayed().await.unwrap_or(false) {
                user_field.send_keys(&self.username).await?;

                if let Ok(pass_field) = driver
                    .query(By::Css("input[type='password']"))
                    .first()
                    .await
                {
                    pass_field.send_keys(&self.password).await?;
                }

                if let Ok(submit_btn) = driver
                    .query(By::Css("button[type='submit'], input[type='submit']"))
                    .first()
                    .await
                {
                    submit_btn.click().await?;
                    sleep(Duration::from_secs(4)).await;
                }
            }
        }
        Ok(())
    }
}

impl DeliveryProvider for OpalProvider {
    fn code(&self) -> &'static str {
        "opal"
    }

    fn name(&self) -> &'static str {
        "OPAL Kurier"
    }

    fn create_shipment<'a>(
        &'a self,
        req: &'a DeliveryRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<DeliveryResponse>> + Send + 'a>> {
        Box::pin(async move {
            info!("OPAL: Starting shipment creation for {}", req.order_number);

            let driver = self.setup_driver().await?;

            if let Err(e) = self.login(&driver).await {
                let _ = driver.quit().await;
                return Err(anyhow::anyhow!("OPAL Login failed: {}", e));
            }

            // TODO: Implement frame switching to 'optop' and 'opmain', filling the shipment form (Phase 8.3).
            let _ = driver.quit().await;

            Ok(DeliveryResponse {
                tracking_number: format!("OPAL-DUMMY-{}", req.order_number),
                raw_response: serde_json::json!({
                    "status": "simulated",
                    "message": "OPAL Scraper initialized successfully"
                }),
            })
        })
    }

    fn fetch_recent_shipments(
        &self,
        _days: i32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<serde_json::Value>>> + Send + '_>> {
        Box::pin(async move {
            // TODO: Implement parsing order list (Phase 8.3)
            Ok(vec![])
        })
    }
}
