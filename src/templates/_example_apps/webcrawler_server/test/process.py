from bs4 import BeautifulSoup

def process_html(file_path):
    with open(file_path, 'r') as file:
        html_content = file.read()

    soup = BeautifulSoup(html_content, 'lxml')
    
    # Extract all visible text content
    texts = soup.stripped_strings
    for text in texts:
        print(text)

if __name__ == "__main__":
    input_file = "/workspaces/firestream/src/services/python/web_crawler/output.html"  # Replace with the desired input file name
    process_html(input_file)
