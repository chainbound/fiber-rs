#!/bin/bash

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 start_slot num_samples"
    exit 1
fi

mkdir -p ./blocks

if ! command -v ethdo &> /dev/null
then
    echo "ethdo could not be found"
    echo "Please install ethdo by running: go get github.com/wealdtech/ethdo"
    exit
fi

start_slot=$1
num_samples=$2

for ((i=0; i<$num_samples; i++)); do
    slot=$((start_slot + i))

    ethdo block info --blockid $slot --ssz | xxd -r -p > ./blocks/block$slot.ssz
done
