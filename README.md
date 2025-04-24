# sushi 🍣

## Setting up the repository

### setting up with `justfile`

```bash
# build the docker image
just build latest

# start a container (optional docker network name)
just run latest

# start a VPN server, inside the container
just vpns

# start a VPN client (IP for the tun interface and the server)
just vpnc 10.0.0.2 192.168.107.2
```

### building the docker image

```bash
sudo docker build . -t sushi`

# creating a docker network
docker network create sushi-test

# building the docker image
docker run --rm -it --network sushi-test --privledged sushi

# running the binary 
./sushi -h
```
