group "default" {
    targets = ["al2023-local", "al2023-runtime"]
}

target "al2023-local" {
    dockerfile = "al2023-build.dockerfile"
    platforms = ["linux/amd64", "linux/arm64"]
    tags = ["dacut/lambda-nop:al2023-build-latest"]
    target = "executable"
    output = ["type=local,dest=build/"]
}

target "al2023-runtime" {
    dockerfile = "al2023-build.dockerfile"
    platforms = ["linux/amd64", "linux/arm64"]
    tags = ["dacut/lambda-nop:al2023", "public.ecr.aws/kanga/lambda-nop:latest"]
    output = ["type=image,push=true"]
    target = "runtime"
}
