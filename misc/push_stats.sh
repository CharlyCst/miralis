#!/usr/bin/env bash
# This script pushes statistics from the current commit into a separate repository.
# It is meant to execute in the CI after a successful merge and assumes it runs on a GNU-flavored Linux system.

miralis_stats_repo_path="stats"
miralis_stats_file="stats.csv"
miralis_stats_csv_path="$miralis_stats_repo_path/$miralis_stats_file"
if [ ! -f "$miralis_stats_csv_path" ]; then
    echo "CSV stats file '$miralis_stats_csv_path' not found!"
    exit 1
fi

release_config="./config/test/qemu-virt-release.toml"
if [ ! -f "$release_config" ]; then
    echo "Config file '$release_config' not found!"
    exit 1
fi

# ———————————————————————————————— Git commit ———————————————————————————————— #

git_commit="$(git rev-parse HEAD)"

# ——————————————————————————————————— Date ——————————————————————————————————— #

current_date="$(date +"%Y-%m-%d")"

# ——————————————————————————————— Miralis size ——————————————————————————————— #

# First we build Miralis in release mode
start=`date +%s.%N`
miralis_img_path="$(just build $release_config | tail -n 1)"
end=`date +%s.%N`
build_time=$(echo $end - $start | bc)

# Then we get the size of the image
if [ "$(uname)" == "Darwin" ]; then
    # MacOS
    miralis_size="$(stat -f %z $miralis_img_path)"
else
    # Linux
    miralis_size="$(stat --format=%s $miralis_img_path)"
fi

# ———————————————————————————————— Get latency cycles ———————————————————————————————— #

file="cycles.txt"

# Run the benchmarks
cargo run -- run --firmware tracing_firmware --config ./config/test/spike-latency-benchmark.toml > $file
cargo run -- run --firmware tracing_firmware --config ./config/test/spike-latency-benchmark-protect-payload.toml >> $file
cargo run -- run --firmware tracing_firmware --config ./config/test/spike-latency-benchmark-offload.toml >> $file

# Extract the number after "firmware cost:"
firmware_cost=$(grep -i "Firmware cost default_policy :" "$file"  | sed -E 's/.*Firmware cost default_policy : ([0-9]+).*/\1/')
payload_cost=$(grep -i "Payload cost default_policy :" "$file" | sed -E 's/.*Payload cost default_policy : ([0-9]+).*/\1/')
firmware_cost_protect=$(grep -i "Firmware cost protect_payload :" "$file"  | sed -E 's/.*Firmware cost protect_payload : ([0-9]+).*/\1/')
payload_cost_protect=$(grep -i "Payload cost protect_payload :" "$file" | sed -E 's/.*Payload cost protect_payload : ([0-9]+).*/\1/')
misaligned_cost_protect=$(grep -i "Misaligned cost protect_payload :" "$file" | sed -E 's/.*Misaligned cost protect_payload : ([0-9]+).*/\1/')
set_time_cost=$(grep -i "Ecall cost to set time offload :" "$file" | sed -E 's/.*Ecall cost to set time offload : ([0-9]+).*/\1/')
read_time_cost=$(grep -i "CSRRS Cost to read time offload :" "$file" | sed -E 's/.CSRRS Cost to read time offload : ([0-9]+).*/\1/')

# ———————————————————————————————— Push stats ———————————————————————————————— #

echo "Commit: $git_commit"
echo "Current date: $current_date"
echo "Miralis size: $miralis_size bytes"
echo "Build time: $build_time"
echo "Miralis <--> Firmware latency in cycles: " $firmware_cost
echo "Payload <--> Firmware latency in cycles: " $payload_cost
echo "[PROTECT PAYLOAD] Miralis <--> Firmware latency in cycles: " $firmware_cost_protect
echo "[PROTECT PAYLOAD] Payload <--> Firmware latency in cycles: " $payload_cost_protect
echo "[PROTECT PAYLOAD] Cost of a misaligned emulation: " $misaligned_cost_protect
echo "[OFFLOAD] Latency to set timer interrupt: " $set_time_cost
echo "[OFFLOAD] Latency to read current time: " $read_time_cost

if [ "$1" = "--commit" ]; then
    csv_entry="$git_commit, $current_date, $miralis_size, $build_time, $firmware_cost, $payload_cost, $firmware_cost_protect, $payload_cost_protect, $misaligned_cost_protect, $set_time_cost, $read_time_cost"
    echo $csv_entry >> "$miralis_stats_csv_path"
    echo "Added CSV entry to $miralis_stats_csv_path"
fi
