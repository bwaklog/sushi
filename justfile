container_hostname := `echo $HOSTNAME`

help:
  @just -l

[unix, no-exit-message]
build tag:
  @if [ -f "/.dockerenv" ]; then \
    echo "Can't build an image, inside a container env"; \
  else \
    echo 'Building bwaklog/sushi:{{tag}}';\
    sudo docker build . \
    --platform linux/arm64 \
    -t bwaklog/sushi:{{tag}}; \
  fi \

[unix, no-exit-message]
run tag hostname network='sushi-test':
  @if [ -f "/.dockerenv" ]; then \
    echo "Can't run an image, inside a container env"; \
  else \
    echo 'Running bwaklog/sushi:{{tag}} on network {{network}}';\
    sudo docker run --rm -it \
    --network {{network}} \
    --platform linux/arm64 \
    --hostname {{hostname}} \
    --privileged \
    bwaklog/sushi:{{tag}}; \
  fi \

[private]
release:
  @echo 'Building sushi release 🍣'
  @cargo build --release

# start a vpn server
vpns:
  ./sushi -s -l {{container_hostname}}:8080

# start a bare metal server with cargo
bare_vpns server: release
  sudo ./target/release/sushi -s -l {{server}}:8080

# start a vpn client
vpnc tunip server:
  ./sushi -t {{tunip}} -l {{container_hostname}}:8080 -r {{server}}:8080

# start a bare metal client with cargo
bare_vpnc tunip local server: release
  sudo ./target/release/sushi -t {{tunip}} -l {{local}}:8080 -r {{server}}:8080

