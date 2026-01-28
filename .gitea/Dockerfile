FROM catthehacker/ubuntu:act-22.04

RUN apt update && \
    apt install -y \
        gcc-mingw-w64-x86-64 \
        protobuf-compiler \
        dotnet-sdk-8.0 && \
    rm -rf /var/lib/apt/lists/*

RUN dotnet tool install -g Microsoft.Web.LibraryManager.Cli

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- \
        -y \
        --default-toolchain nightly \
        --target x86_64-pc-windows-gnu,x86_64-unknown-linux-gnu \
        --profile minimal

ENV PATH="/root/.cargo/bin:/root/.dotnet/tools:${PATH}"