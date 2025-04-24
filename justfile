container_ip := if os() == "linux" {`ip addr show eth0 | awk '/inet/ {print $2}' | cut -d '/' -f1`} else { '0.0.0.0' }

help:
  @just -l

[unix, no-exit-message]
build tag:
  @if [ -f "/.dockerenv" ]; then \
    echo "Can't build an image, inside a container env"; \
  else \
    sudo docker build . --platform linux/arm64 -t sushi:{{tag}}; \
  fi \

[unix, no-exit-message]
run tag network='sushi-test':
  @if [ -f "/.dockerenv" ]; then \
    echo "Can't run an image, inside a container env"; \
  else \
    sudo docker run --rm -it --network {{network}} --platform linux/arm64 --privileged sushi:{{tag}}; \
  fi \

[private]
release:
  @echo 'Building sushi release 🍣'
  @cargo build --release

# start a vpn server
vpns:
  ./sushi -s -l {{container_ip}}:8080

# start a bare metal server with cargo
bare_vpns server: release
  sudo ./target/release/sushi -s -l {{server}}:8080

# start a vpn client
vpnc tunip server:
  ./sushi -t {{tunip}} -l {{container_ip}}:8080 -r {{server}}:8080

# start a bare metal client with cargo
bare_vpnc tunip local server: release
  sudo ./target/release/sushi -t {{tunip}} -l {{local}}:8080 -r {{server}}:8080

