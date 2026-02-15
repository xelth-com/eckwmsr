use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::time::sleep;
use tracing::{error, info};

use super::delivery::{DeliveryProvider, DeliveryRequest, DeliveryResponse};
use crate::utils::webdriver::ensure_chromedriver;

pub struct DhlProvider {
    username: String,
    password: String,
    url: String,
}

impl DhlProvider {
    pub fn new(username: String, password: String, url: String) -> Self {
        let url = if url.is_empty() {
            "https://geschaeftskunden.dhl.de".to_string()
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

        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        Ok(driver)
    }

    async fn login(&self, driver: &WebDriver) -> anyhow::Result<()> {
        info!("DHL: Navigating to login page...");
        driver.goto(&self.url).await?;
        sleep(Duration::from_secs(3)).await;

        // 1. Handle Cookie Banner (OneTrust)
        if let Ok(cookie_btn) = driver.find(By::Id("onetrust-accept-btn-handler")).await {
            if cookie_btn.is_displayed().await.unwrap_or(false) {
                let _ = cookie_btn.click().await;
                info!("DHL: Accepted cookies");
                sleep(Duration::from_secs(2)).await;
            }
        }

        // 2. Click Login button to open form
        if let Ok(login_trigger) = driver
            .find(By::XPath("//button[contains(text(), 'Anmelden')]"))
            .await
        {
            let _ = login_trigger.click().await;
            sleep(Duration::from_secs(2)).await;
        }

        // 3. Fill credentials
        let email_field = driver.query(By::Css("input[type='email']")).first().await?;
        email_field.send_keys(&self.username).await?;

        let pass_field = driver
            .query(By::Css("input[type='password']"))
            .first()
            .await?;
        pass_field.send_keys(&self.password).await?;

        // 4. Submit
        let submit_btn = driver
            .query(By::Css("button[type='submit']"))
            .first()
            .await?;
        submit_btn.click().await?;

        info!("DHL: Login submitted, waiting for dashboard...");
        sleep(Duration::from_secs(5)).await;

        Ok(())
    }
}

impl DeliveryProvider for DhlProvider {
    fn code(&self) -> &'static str {
        "dhl"
    }

    fn name(&self) -> &'static str {
        "DHL Geschäftskunden"
    }

    fn create_shipment<'a>(
        &'a self,
        req: &'a DeliveryRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<DeliveryResponse>> + Send + 'a>> {
        Box::pin(async move {
            info!("DHL: Starting shipment creation for {}", req.order_number);

            let driver = self.setup_driver().await?;

            if let Err(e) = self.login(&driver).await {
                let _ = driver.quit().await;
                return Err(anyhow::anyhow!("DHL Login failed: {}", e));
            }

            // TODO: Navigate to ShipmentDetails and fill form (Phase 8.3)
            let _ = driver.quit().await;

            Ok(DeliveryResponse {
                tracking_number: format!("DHL-DUMMY-{}", req.order_number),
                raw_response: serde_json::json!({
                    "status": "simulated",
                    "message": "Scraper initialized successfully"
                }),
            })
        })
    }

    fn fetch_recent_shipments(
        &self,
        _days: i32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<serde_json::Value>>> + Send + '_>> {
        Box::pin(async move {
            // TODO: Port CSV export parsing (Phase 8.3)
            Ok(vec![])
        })
    }
}
