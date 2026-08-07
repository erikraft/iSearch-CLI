from playwright.sync_api import sync_playwright
import os

def run_cuj(page):
    abs_path = os.path.abspath("scripts/donation.html")
    page.set_viewport_size({"width": 1280, "height": 900})
    page.goto(f"file://{abs_path}")
    page.wait_for_timeout(1000)

    # Verify that the title is correct
    print("Page Title:", page.title())

    # Let's check if the city PERUIBE is absent from the page text
    body_text = page.locator("body").inner_text()
    if "PERUIBE" in body_text:
        print("[FAIL] PERUIBE is visible in the page body!")
    else:
        print("[PASS] PERUIBE is not visible in the page body.")

    # Let's select the R$ 10 donation button
    btn_10 = page.locator(".value-btn[data-value='10']")
    btn_10.click()
    page.wait_for_timeout(500)

    # Click the generate PIX QR Code button
    generate_btn = page.locator("#generate-pix-btn")
    generate_btn.click()
    page.wait_for_timeout(1000)

    # Check if the PIX copy and paste field is populated
    payload_input = page.locator("#pix-payload-input")
    payload_value = payload_input.input_value()
    print("Generated PIX Payload:", payload_value)

    # Ensure PERUIBE is present inside the payload string itself
    if "PERUIBE" in payload_value:
        print("[PASS] PERUIBE is correctly present inside the PIX payload string.")
    else:
        print("[FAIL] PERUIBE is missing from the PIX payload string!")

    # Click copy PIX code button
    copy_btn = page.locator("#copy-pix-btn")
    copy_btn.click()
    page.wait_for_timeout(1000)

    # Take screenshot at the final state
    page.screenshot(path="/home/jules/verification/screenshots/donation_flow.png")
    page.wait_for_timeout(1000)

if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(
            record_video_dir="/home/jules/verification/videos"
        )
        page = context.new_page()
        try:
            run_cuj(page)
        finally:
            context.close()
            browser.close()
