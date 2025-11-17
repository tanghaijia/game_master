#!/bin/bash

unset http_proxy
unset https_proxy

cd /root/frp/frp_0.65.0_linux_amd64
./frpc -c ./frpc.toml &
cd /root/game_master

export DATA_SERVER_IP_ADDR=192.168.8.88
export RUSTFS_REGION=cn-east-1
export RUSTFS_ACCESS_KEY_ID=2yv03sZrLW9iaAwKm8uO
export RUSTFS_SECRET_ACCESS_KEY=thj@13835720054
export RUSTFS_ENDPOINT_URL=http://192.168.8.168:9001/

chmod +x ./game_master
./game_master
