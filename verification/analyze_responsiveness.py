import os
from playwright.sync_api import sync_playwright

def run_cuj(page, width, name):
    abs_path = os.path.abspath("scripts/index.html")
    page.set_viewport_size({"width": width, "height": 800})
    page.goto(f"file://{abs_path}")
    page.wait_for_timeout(1000)

    # Hide preloader
    page.evaluate("document.querySelector('.site-loader')?.classList.add('hidden')")
    page.wait_for_timeout(500)

    # Scroll to #termux section
    termux_section = page.locator("#termux")
    termux_section.scroll_into_view_if_needed()
    page.wait_for_timeout(500)

    # Capture screenshot
    page.screenshot(path=f"verification/screenshots/termux_{name}.png")

    # Check for horizontal scroll / overflow
    has_overflow = page.evaluate("document.documentElement.scrollWidth > window.innerWidth")
    print(f"Viewport width {width}px ({name}): Has horizontal scroll? {has_overflow}")

if __name__ == "__main__":
    widths = {
        320: "320px",
        360: "360px",
        375: "375px",
        390: "390px",
        414: "414px"
    }
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context()
        for w, name in widths.items():
            page = context.new_page()
            try:
                run_cuj(page, w, name)
            finally:
                page.close()
        context.close()
        browser.close()
