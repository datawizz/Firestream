import requests
import json

# Step 1: Replace these values with your own client ID and client secret from the Procore Developer Portal
client_id = '3f05ba59dce6d4f72a070fa8460656de6fcfa782882ff983668a4b0862c5114e'
client_secret = '796410523fecc93dd30095e0b05b0095fb196273e4f0cabf45b4c877472d6ec3'

# Step 2: Replace these values with your Procore credentials
username = 'justin@centerpoint.consulting'
password = 'Rm%HrQcyC#ELiu^iuqGnZIYIkLvhtu&@n*$B9!N7ra'

# Authenticate and get an access token
auth_url = 'https://api.procore.com/oauth/token'
auth_data = {
    'grant_type': 'authorization_code',
    'client_id': client_id,
    'client_secret': client_secret,
    # 'username': username,
    # 'password': password,
}

response = requests.post(auth_url, data=auth_data)
response_data = response.json()

print(response_data)

if response.status_code != 200:
    print("Authentication failed. Please check your credentials.")
    exit(1)

access_token = response_data['access_token']

# Make an API request to get data from the Procore API
api_url = 'https://api.procore.com/vapid/me'  # Replace this URL with the desired API endpoint
headers = {
    'Authorization': f'Bearer {access_token}',
    'Content-Type': 'application/json',
}

response = requests.get(api_url, headers=headers)
response_data = response.json()

if response.status_code != 200:
    print("API request failed. Please check the API endpoint.")
    exit(1)

# Print the data retrieved from the Procore API
print(json.dumps(response_data, indent=4))
