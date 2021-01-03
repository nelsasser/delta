#!~/bin/bash

cd ./core/delta-nodes && cargo build
cd ./nodes && cargo build
cd ./core/delta-core && cargo build