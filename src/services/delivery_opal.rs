use regex::Regex;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::time::sleep;
use tracing::{info, warn};

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

        if let Ok(user_field) = driver
            .query(By::Css("input[name='username']"))
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

            // 1. Navigate to "Neuer Auftrag" using the header frame (optop)
            let optop_frame = driver.query(By::Css("frame[name='optop']")).first().await?;
            optop_frame.enter_frame().await?;

            if let Ok(new_order_link) = driver
                .query(By::XPath(
                    "//a[contains(text(), 'Neuer Auftrag') or contains(@href, 'new')]",
                ))
                .first()
                .await
            {
                new_order_link.click().await?;
                info!("OPAL: Clicked 'Neuer Auftrag'");
                sleep(Duration::from_secs(2)).await;
            }
            driver.enter_default_frame().await?;

            // 2. Switch to main content frame (opmain) to fill the form
            let opmain_frame = driver.query(By::Css("frame[name='opmain']")).first().await?;
            opmain_frame.enter_frame().await?;

            // Helper macro to fill array-based inputs by index (0 = Pickup, 1 = Delivery)
            macro_rules! fill_array_field {
                ($selector:expr, $index:expr, $value:expr) => {
                    if let Ok(elements) = driver.query(By::Css($selector)).all().await {
                        if elements.len() > $index {
                            let _ = elements[$index].send_keys($value).await;
                        }
                    }
                };
            }

            // Pickup address (index 0)
            fill_array_field!("input[name='address_name1[]']", 0, &req.sender_address.name1);
            fill_array_field!("input[name='address_str[]']", 0, &req.sender_address.street);
            fill_array_field!("input[name='address_plz[]']", 0, &req.sender_address.zip);
            fill_array_field!("input[name='address_ort[]']", 0, &req.sender_address.city);

            // Delivery address (index 1)
            fill_array_field!("input[name='address_name1[]']", 1, &req.receiver_address.name1);
            fill_array_field!("input[name='address_str[]']", 1, &req.receiver_address.street);
            fill_array_field!("input[name='address_plz[]']", 1, &req.receiver_address.zip);
            fill_array_field!("input[name='address_ort[]']", 1, &req.receiver_address.city);

            // Package Details
            if let Ok(weight_input) = driver.query(By::Css("input#segewicht")).first().await {
                let weight_str = req.weight.to_string().replace('.', ",");
                let _ = weight_input.send_keys(&weight_str).await;
            }
            if let Ok(ref_input) = driver.query(By::Css("input#seclref")).first().await {
                let _ = ref_input.send_keys(&req.ref_number).await;
            }

            // Submit Order
            if let Ok(submit_btn) = driver
                .query(By::Css("input[type='submit'], button[type='submit']"))
                .first()
                .await
            {
                submit_btn.click().await?;
                info!("OPAL: Form submitted");
                sleep(Duration::from_secs(4)).await;
            } else {
                let _ = driver.quit().await;
                return Err(anyhow::anyhow!("OPAL: Submit button not found"));
            }

            // Extract Tracking Number
            let body = driver.query(By::Css("body")).first().await?;
            let body_text = body.text().await.unwrap_or_default();

            let re = Regex::new(r"(?i)Sendungsnummer[:\s]*([A-Z0-9-]+)").unwrap();
            let tracking_number = if let Some(caps) = re.captures(&body_text) {
                caps[1].to_string()
            } else {
                warn!("OPAL: Could not parse tracking number from response.");
                format!("OPAL-UNKNOWN-{}", req.order_number)
            };

            let _ = driver.quit().await;

            Ok(DeliveryResponse {
                tracking_number,
                raw_response: serde_json::json!({"status": "created", "provider": "opal"}),
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
