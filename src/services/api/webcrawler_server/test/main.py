import time
import sys
from selenium import webdriver
from selenium.webdriver.chrome.options import Options


def save_html(url):
    options = Options()
    options.headless = True
    options.add_argument("--no-sandbox")
    options.add_argument("--disable-dev-shm-usage")
    options.binary_location = "/usr/bin/chromium"

    driver = webdriver.Chrome(executable_path="chromedriver", options=options)
    driver.get(url)


    # Wait for the page to load
    time.sleep(3)

    # Dismiss the cookie consent popup if it exists
    try:
        close_button = WebDriverWait(driver, 5).until(
            EC.element_to_be_clickable((By.CSS_SELECTOR, "button.close"))
        )
        close_button.click()
    except:
        print("No cookie consent popup found or failed to close it.")

    # Switch to the iframe containing the content you want
    try:
        # Adjust the selector to match the iframe on the website
        iframe = WebDriverWait(driver, 5).until(
            EC.presence_of_element_located((By.CSS_SELECTOR, "iframe"))
        )
        driver.switch_to.frame(iframe)
    except:
        print("No iframe found or failed to switch to it.")

    #TODO update dynamically
    output_file = "output.html" 

    with open(output_file, "w") as f:
        f.write(driver.page_source)

    driver.quit()

if __name__ == "__main__":
    url = "https://procore.github.io/documentation/development-environments"  # Replace with the desired URL
    output_file = "output.html"  # Replace with the desired output file name
    save_html(url)
