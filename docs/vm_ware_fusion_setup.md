# VM Ware Fusion

To setup the project in VM Ware Fusion on a Mac using M1/2 ARM processors follow these steps.

1. Download VM Ware Fusion -> https://www.vmware.com/products/fusion.html
2. Download Ubuntu ARM Server -> https://ubuntu.com/download/server/arm
3. Create a VM in VMWare with the following configs:
    1. 64gb of Disk Space
    2. 4 CPU
    3. 8 GB Ram
    4. Network Adapter 1 -> "Share with my Mac" (NAT)
    5. Network Adapter 2 -> "Private to my Mac"

    The first network adapter is for internet connectivity.
    The second network adapter is for SSH into the machine from the Mac. This has to be a private network so that it works even
    when there is no external network available to the Mac.

4. Proceed with installation.
    Ubuntu Server allows pre-installation of Docker and MicroK8s which will be useful later.

5. Boot into the VM and clone the project

6. cd fireworks && sh bootstrap.sh