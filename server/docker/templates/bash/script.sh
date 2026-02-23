#!/bin/bash

read -r input_string

output=$(./run "$input_string")

echo "$output"
