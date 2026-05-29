FROM debian:latest

RUN apt-get update && \
    apt-get install -y gpg wget lsb-release software-properties-common ninja-build

RUN set -ex &&\
    echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-16 main" > /etc/apt/sources.list.d/apt.llvm.org.list &&\
    wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key |  tee /etc/apt/trusted.gpg.d/apt.llvm.org.asc &&\
    apt update &&\
    apt-get install -y clang-16 lldb-16 lld-16 clangd-16 clang-tidy-16 clang-format-16 clang-tools-16 llvm-16-dev lld-16 lldb-16 llvm-16-tools libomp-16-dev libc++-16-dev libc++abi-16-dev libclang-common-16-dev libclang-16-dev libclang-cpp16-dev libunwind-16-dev libclang-rt-16-dev libpolly-16-dev

RUN wget https://apt.llvm.org/llvm.sh && chmod +x llvm.sh && ./llvm.sh 16

RUN apt-get update && apt-get install -y cmake libnuma-dev ninja-build clang-format protobuf-compiler curl

ENV PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/lib/llvm-16/bin:/root/.cargo/bin

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN mkdir /root/lore
WORKDIR /root/lore

COPY . .

RUN ./build-linux.sh

# CMD sleep 100000 # to interact with the image
