
# Webcrawler Service

Implements a REST endpoint for Selenium.

The service implements a REST endpoint which returns the fully rendered HTML a URL provided in the payload.
It does this using Selenium and the Chrome driver by default. The compiled and rendered HTML is then returned via the POST request.

# TODO

1. The Dockerfile needs to be updated to actually work.
1. There needs to be a Helm chart.
1. There needs to be a Kafka topic that stores scraped HTML files.

1. There is a bug in the /scrape endpoint which does not pull iframes, even if those are valid. This could be fixed in the parses logic.
2. Scraping a single webpage and returning the result is good but it should also have the option to follow links, and scrape those pages as well.
3. ELT Lib and DataModels need to be used here.
4. Firefox is less detectable for scraping (according to the internet?)

5. How can I use Google OAuth to sign into a site? Some sites don't support username and password. The SSO sign in with Google is often used. Google's EULA does not allow using Sign in With Google with automated services. So we have a arms race.
