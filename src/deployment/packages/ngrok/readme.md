
# Ngrok

Ngrok is used to provide a tunnel by which the internal Kubernetes services are exposed to the outside world.

1. Install the helm chart
2. Install the ingress records to map routes to services
3. Ensure the API key for ngrok is loaded
4. Create a new subdomain on the account's domain

    feature_branch_xyz.project_name.centerpoint.consulting

    This approach allows each feature branch to operate independently with its own accessible subdomain, like foo in foo.example.com or foo.bar in foo.bar.example.com.
