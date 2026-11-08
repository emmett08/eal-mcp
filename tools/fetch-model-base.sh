#!/bin/sh
# Fetch the immutable base artifacts used by the optional eal-model-host.
set -eu

base_dir=${1:-models/base}
base_revision=7ae557604adf67be50417f59c2c2f167def9a775
base_url="https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/$base_revision"
mkdir -p "$base_dir"

fetch_one() {
    artifact_name=$1
    expected_sha=$2
    artifact_path="$base_dir/$artifact_name"
    if [ -f "$artifact_path" ]; then
        actual_sha=$(shasum -a 256 "$artifact_path" | cut -d ' ' -f 1)
        if [ "$actual_sha" = "$expected_sha" ]; then
            echo "Verified $artifact_path"
            return
        fi
        echo "Existing $artifact_path has a different SHA256. Move it aside before fetching." >&2
        exit 1
    fi
    temporary_path="$artifact_path.download"
    trap 'rm -f "$temporary_path"' EXIT HUP INT TERM
    curl --fail --location --retry 2 --proto '=https' --proto-redir '=https' \
        --tlsv1.2 --output "$temporary_path" "$base_url/$artifact_name"
    actual_sha=$(shasum -a 256 "$temporary_path" | cut -d ' ' -f 1)
    if [ "$actual_sha" != "$expected_sha" ]; then
        echo "SHA256 mismatch for $artifact_name" >&2
        exit 1
    fi
    mv "$temporary_path" "$artifact_path"
    trap - EXIT HUP INT TERM
    echo "Verified $artifact_path"
}

fetch_one config.json 18e18afcaccafade98daf13a54092927904649e1dd4eba8299ab717d5d94ff45
fetch_one generation_config.json e558847a8b4402616f1273797b015104dc266fe4b520056fca88823ba8f8ebe6
fetch_one model.safetensors fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe
