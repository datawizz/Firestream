

# Poor man's DNS server accomplished by updating the hosts file inside the container.
echo "172.16.16.240 nginx.example.com" | sudo tee -a /etc/hosts

wget https://github.com/txn2/kubefwd/releases/download/1.22.4/kubefwd_Linux_x86_64.tar.gz

