use anyhow::Result;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Browser Automation Benchmarking (Size Comparison) ---");

    #[cfg(feature = "feat_chromiumoxide")]
    {
        println!("Compiled with: chromiumoxide");
        let _ = bench_chromiumoxide().await;
    }

    #[cfg(feature = "feat_headless_chrome")]
    {
        println!("Compiled with: headless_chrome");
        let _ = bench_headless_chrome().await;
    }

    #[cfg(feature = "feat_fantoccini")]
    {
        println!("Compiled with: fantoccini");
        let _ = bench_fantoccini().await;
    }

    Ok(())
}

#[cfg(feature = "feat_chromiumoxide")]
async fn bench_chromiumoxide() -> Result<()> {
    use chromiumoxide::page::ScreenshotParams;
    use chromiumoxide::Browser;
    use futures::StreamExt;

    let config = chromiumoxide::BrowserConfig::builder()
        .build()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });
    let page = browser.new_page("https://kroki.io").await?;
    let _screenshot = page.screenshot(ScreenshotParams::builder().build()).await?;
    browser.close().await?;
    let _ = handle.await;
    Ok(())
}

#[cfg(feature = "feat_headless_chrome")]
async fn bench_headless_chrome() -> Result<()> {
    use headless_chrome::Browser as HeadlessBrowser;
    let browser = HeadlessBrowser::default().map_err(|e| anyhow::anyhow!("{}", e))?;
    let tab = browser.new_tab().map_err(|e| anyhow::anyhow!("{}", e))?;
    tab.navigate_to("https://kroki.io")
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    tab.wait_until_navigated()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let _screenshot = tab
        .capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

#[cfg(feature = "feat_fantoccini")]
async fn bench_fantoccini() -> Result<()> {
    // Basic fantoccini placeholder to trigger dependency compilation
    use fantoccini::{ClientBuilder, Locator};
    let _c = ClientBuilder::native();
    Ok(())
}
