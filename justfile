container_ip := if os() == "linux" {`ip addr show eth0 | awk '/inet/ {print $2}' | cut -d '/' -f1`} else { '0.0.0.0' }

[unix, no-exit-message]
build tag:
  @if [ -f "/.dockerenv" ]; then \
    echo "Can't build an image, inside a container env"; \
  else \
    sudo docker build . -t sushi:{{tag}}; \
  fi \

[unix, no-exit-message]
run tag network='sushi-test':
  @if [ -f "/.dockerenv" ]; then \
    echo "Can't run an image, inside a container env"; \
  else \
    sudo docker run --rm -it --network {{network}} --privileged sushi:{{tag}}; \
  fi \

# start a vpn server
vpns:
  ./sushi -s -l {{container_ip}}:8080

# start a vpn client
vpnc tunip server:
  ./sushi -t {{tunip}} -l {{container_ip}}:8080 -r {{server}}:8080
