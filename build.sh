#!/bin/bash -ex
# Build via Docker for both Arm64 and x86-64
# docker buildx build --platform linux/amd64,linux/arm64 --tag dacut/lambda-nop-build --file al2023-build.dockerfile .
# docker buildx build --platform linux/amd64 --tag dacut/lambda-nop-build-amd64 --load --file al2023-build.dockerfile .
# docker buildx build --platform linux/amd64 --tag dacut/lambda-nop-build-arm64 --load --file al2023-build.dockerfile .

docker buildx bake --file docker-bake.hcl 
