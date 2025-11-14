#!/bin/bash

cp /root/game_master/game_master/target/release/game_master /root/game_master/game_master_release
chmod +x /root/game_master_release
cd /root/game_master

export DATA_SERVER_IP_ADDR=127.0.0.1

./game_master_release