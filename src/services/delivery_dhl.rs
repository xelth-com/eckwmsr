use regex::Regex;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::time::sleep;
use tracing::{info, warn};

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

        WebDriver::new("http://localhost:9515", caps)
            .await
            .map_err(|e| anyhow::anyhow!(e))
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

        // 2. Click Login trigger
        if let Ok(login_trigger) = driver
            .find(By::XPath("//button[contains(text(), 'Anmelden')]"))
            .await
        {
            let _ = login_trigger.click().await;
            sleep(Duration::from_secs(2)).await;
        }

        // 3. Fill credentials
        if let Ok(email_field) = driver.query(By::Css("input[type='email']")).first().await {
            email_field.send_keys(&self.username).await?;
            if let Ok(pass_field) = driver.query(By::Css("input[type='password']")).first().await {
                pass_field.send_keys(&self.password).await?;
            }
            if let Ok(submit_btn) = driver.query(By::Css("button[type='submit']")).first().await {
                submit_btn.click().await?;
                info!("DHL: Login submitted, waiting for dashboard...");
                sleep(Duration::from_secs(5)).await;
            }
        }

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

            // Navigate to Shipment Details
            let details_url = format!("{}/content/vls/vc/ShipmentDetails", self.url);
            driver.goto(&details_url).await?;
            sleep(Duration::from_secs(3)).await;

            // Helper macro for safe field filling
            macro_rules! fill_field {
                ($selector:expr, $value:expr) => {
                    if let Ok(el) = driver.query(By::Css($selector)).first().await {
                        let _ = el.send_keys($value).await;
                    }
                };
            }

            // Fill Receiver Address
            fill_field!("input[id='receiver.name1']", &req.receiver_address.name1);
            fill_field!(
                "input[id='receiver.street']",
                &req.receiver_address.street
            );
            fill_field!(
                "input[id='receiver.streetNumber']",
                &req.receiver_address.house_number
            );
            fill_field!("input[id='receiver.plz']", &req.receiver_address.zip);
            fill_field!("input[id='receiver.city']", &req.receiver_address.city);

            // Fill Shipment Data (Weight with comma for German locale)
            let weight_str = req.weight.to_string().replace('.', ",");
            fill_field!("input[id='shipment-weight']", &weight_str);

            // Submit Form
            if let Ok(submit_btn) = driver
                .query(By::XPath(
                    "//button[contains(text(), 'Versenden') or contains(text(), 'Drucken')]",
                ))
                .first()
                .await
            {
                submit_btn.click().await?;
                info!("DHL: Form submitted");
                sleep(Duration::from_secs(5)).await;
            } else {
                let _ = driver.quit().await;
                return Err(anyhow::anyhow!("DHL: Submit button not found"));
            }

            // Extract Tracking Number from success page
            let body = driver.query(By::Css("body")).first().await?;
            let body_text = body.text().await.unwrap_or_default();

            let re = Regex::new(
                r"(?i)(?:Sendungsnummer|Tracking|Paketnummer)[:\s]+(\d{10,20})",
            )
            .unwrap();
            let tracking_number = if let Some(caps) = re.captures(&body_text) {
                caps[1].to_string()
            } else {
                warn!("DHL: Could not parse tracking number from response.");
                format!("DHL-UNKNOWN-{}", req.order_number)
            };

            let _ = driver.quit().await;

            Ok(DeliveryResponse {
                tracking_number,
                raw_response: serde_json::json!({"status": "created", "provider": "dhl"}),
            })
        })
    }

    fn fetch_recent_shipments(
        &self,
        _days: i32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<serde_json::Value>>> + Send + '_>> {
        Box::pin(async move { Ok(vec![]) })
    }
}
