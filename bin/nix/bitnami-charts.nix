# Fetches a specific Git repository at a precise commit
{ fetchgit }:

fetchgit {
  name = "bitnami-charts";
  url = "https://github.com/bitnami/charts.git";  # Replace with your repository URL
  rev = "9bc801b4caa0b2fff6ae3392f6b417877a056965";  # Replace with your specific commit hash
  sha256 = "8No+rUyEmugs26c7XYo1SAwlafG8sKrhsk6FnaJwL/U=";    # Replace with the hash of the repository at that commit

  # Optional configurations
  fetchSubmodules = false;  # Set to true if you need submodules
  deepClone = false;        # Keep this false for better performance
}
