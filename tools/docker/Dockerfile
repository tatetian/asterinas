# SPDX-License-Identifier: MPL-2.0

#= Install packages for Docker building ====================================

FROM ubuntu:22.04 AS build-base

SHELL ["/bin/bash", "-c"]

ARG DEBIAN_FRONTEND=noninteractive

# Please keep the list sorted by name
RUN apt update && apt-get install -y --no-install-recommends \
    apache2-utils \
    build-essential \
    ca-certificates \
    clang \
    cmake \
    curl \
    git-core \
    gnupg \
    libevent-dev \
    libslirp-dev \
    libssl-dev \
    jq \
    python3-pip \
    python-is-python3 \
    tcl-dev \
    unzip \
    wget \
    zip
RUN pip3 install yq tomli

#= Download dependency =====================================================

WORKDIR /opt/linux_binary_cache
RUN wget https://raw.githubusercontent.com/asterinas/linux_vdso/2a6d2db/vdso64.so -O vdso64.so
RUN wget https://raw.githubusercontent.com/asterinas/linux_binary_cache/14598b6/vmlinuz-5.15.0-105 \
        -O vmlinuz

#= Build benchmark =========================================================

FROM build-base AS build-benchmarks

# Download the source files of benchmarks
WORKDIR /root
RUN apt install -y automake \
    libtool \
    pkg-config \
    libntirpc-dev

RUN wget https://github.com/akopytov/sysbench/archive/1.0.20.tar.gz \
    && tar -zxvf 1.0.20.tar.gz \
    && rm 1.0.20.tar.gz
RUN git clone https://github.com/nicktehrany/membench.git
RUN git clone https://github.com/esnet/iperf.git
RUN git clone https://github.com/kdlucas/byte-unixbench.git
RUN git clone https://github.com/asterinas/lmbench.git
RUN wget https://www.iozone.org/src/current/iozone3_506.tar
RUN tar -x -f iozone3_506.tar
RUN git clone -b fio-3.37 https://github.com/axboe/fio.git
RUN git clone https://git.kernel.org/pub/scm/linux/kernel/git/clrkwllms/rt-tests.git
RUN git clone https://git.kernel.org/pub/scm/linux/kernel/git/mason/schbench.git
RUN wget https://launchpad.net/libmemcached/1.0/1.0.18/+download/libmemcached-1.0.18.tar.gz \
    && tar -zxvf libmemcached-1.0.18.tar.gz \
    && rm libmemcached-1.0.18.tar.gz

# Build sysbench
WORKDIR /root/sysbench-1.0.20
RUN ./autogen.sh \
    && ./configure --without-mysql --prefix=/usr/local/benchmark/sysbench \
    && make -j \
    && make install

# Build membench
WORKDIR /root/membench
RUN make -j \
    && mkdir /usr/local/benchmark/membench \
    && cp membench /usr/local/benchmark/membench/

# Build iperf
WORKDIR /root/iperf
RUN ./configure --prefix=/usr/local/benchmark/iperf \
    && make -j \
    && make install

# Build lmbench
WORKDIR /root/lmbench
RUN make -j \
    && mv bin/x86_64-linux-gnu /usr/local/benchmark/lmbench \
    && rm /usr/local/benchmark/lmbench/*.o \
    && rm /usr/local/benchmark/lmbench/*.a

# Build unixbench
WORKDIR /root/byte-unixbench/UnixBench
RUN make UB_GCC_OPTIONS=-mno-sse2 -j && mv pgms /usr/local/benchmark/unixbench

# Build iozone
WORKDIR /root/iozone3_506/src/current
RUN CFLAGS=-static make linux-AMD64 \
    && cp iozone /usr/local/benchmark/

# Build fio
WORKDIR /root/fio
RUN ./configure --disable-shm --prefix=/usr/local/benchmark/fio \
    # Remove this when we support syscall timerfd_create and fadvise
    && sed -i -e '/#define CONFIG_HAVE_TIMERFD_CREATE/d' -e '/#define CONFIG_POSIX_FADVISE/d' config-host.h \
    && make -j \
    && make install

# Build hackbench
WORKDIR /root/rt-tests/src/hackbench
RUN make hackbench \
    && cp hackbench /usr/local/benchmark

# Build schbench
WORKDIR /root/schbench
RUN make \
    && cp schbench /usr/local/benchmark/

# Build memaslap for memcached
WORKDIR /root/libmemcached-1.0.18
RUN LDFLAGS='-lpthread' CPPFLAGS='-fcommon -fpermissive' CFLAGS='-fpermissive -fcommon' \
    ./configure --enable-memaslap --prefix=/usr/local/benchmark/libmemcached \
    && CPPFLAGS='-fcommon' make -j \
    && make install

# Clear cached files
WORKDIR /root
RUN rm -rf sysbench-1.0.20 \
    membench \
    iperf \
    lmbench \
    byte-unixbench \
    iozone3_506.tar \
    iozone3_506 \
    fio \
    hackbench \
    schbench \
    libmemcached-1.0.18

#= Install applications =======================================================

FROM build-base AS build-applications

# Install SQLite
WORKDIR /root
RUN apt-get install -y sqlite sqlite3
RUN wget https://www.sqlite.org/2024/sqlite-amalgamation-3460100.zip \
    && unzip sqlite-amalgamation-3460100.zip \
    && cd sqlite-amalgamation-3460100 \
    && gcc -g -shared -fPIC -c sqlite3.c \
    && gcc -g -shared -fPIC -o libsqlite3.so sqlite3.o \
    && mv ./libsqlite3.so /lib/x86_64-linux-gnu/ \
    && mv ./sqlite3.h /usr/include/x86_64-linux-gnu/ \
    && mv ./sqlite3ext.h /usr/include/x86_64-linux-gnu/

RUN rm -rf sqlite-amalgamation-3460100.zip \
    sqlite-amalgamation-3460100

# Install SQLite-speedtest
WORKDIR /root
RUN git clone --branch version-3.46.1 https://github.com/sqlite/sqlite.git
RUN cd sqlite \
    && mkdir bld \
    && cd bld \
    && ../configure --enable-all \
    && make speedtest1 \
    && cp ./speedtest1 /usr/local

RUN rm -rf sqlite

# Instal LevelDB 1.23
WORKDIR /root
RUN mkdir -p /usr/local/leveldb/benchmark/
RUN git clone -b 1.23 --recurse-submodules https://github.com/google/leveldb.git \
    && cd leveldb \
    && mkdir -p build \
    && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. \
    && cmake --build . \
    && make install \
    && mv ./db_bench /usr/local/leveldb/benchmark/ \
    && mv ./db_bench_sqlite3 /usr/local/leveldb/benchmark/

RUN rm -rf 1.23.tar.gz \
    leveldb-1.23

# Install Redis-7.0.15
WORKDIR /root
RUN wget https://download.redis.io/releases/redis-7.0.15.tar.gz \
    && tar -xzvf redis-7.0.15.tar.gz \
    && cd redis-7.0.15 \
    && make -j \
    && make PREFIX=/usr/local/redis install

RUN rm -rf redis-7.0.15.tar.gz \
    redis-7.0.15

# Install Nginx only with http enabled
WORKDIR /root
RUN wget https://nginx.org/download/nginx-1.26.2.tar.gz \
    && tar -xzvf nginx-1.26.2.tar.gz \
    && cd nginx-1.26.2 \
    && ./configure --with-cc-opt="-O2" --with-ld-opt="-static" --without-select_module --without-poll_module \
    --without-quic_bpf_module --without-http_charset_module --without-http_gzip_module --without-http_ssi_module \
    --without-http_userid_module --without-http_access_module --without-http_auth_basic_module --without-http_mirror_module \
    --without-http_geo_module --without-http_map_module --without-http_split_clients_module --without-http_referer_module \
    --without-http_rewrite_module --without-http_proxy_module --without-http_fastcgi_module --without-http_uwsgi_module \
    --without-http_scgi_module --without-http_grpc_module --without-http_memcached_module --without-http_limit_conn_module \
    --without-http_limit_req_module --without-http_empty_gif_module --without-http_browser_module --without-http_upstream_hash_module \
    --without-http_upstream_ip_hash_module --without-http_upstream_least_conn_module --without-http_upstream_random_module \
    --without-http_upstream_keepalive_module --without-http_upstream_zone_module --without-http-cache --without-mail_pop3_module \
    --without-mail_imap_module --without-mail_smtp_module --without-stream_limit_conn_module --without-stream_access_module \
    --without-stream_geo_module --without-stream_map_module --without-stream_split_clients_module --without-stream_return_module \
    --without-stream_pass_module --without-stream_set_module --without-stream_upstream_hash_module --without-stream_upstream_least_conn_module \
    --without-stream_upstream_random_module --without-stream_upstream_zone_module --without-pcre --without-pcre2

WORKDIR /root/nginx-1.26.2
RUN make -j \
    && make install

WORKDIR /root
RUN rm -rf nginx-1.26.2.tar.gz \
    nginx-1.26.2

# Install Memcached v1.6.32
WORKDIR /root
RUN wget https://www.memcached.org/files/memcached-1.6.32.tar.gz \
    && tar -xzvf memcached-1.6.32.tar.gz \
    && cd memcached-1.6.32 \
    && ./configure --prefix=/usr/local/memcached \
    && make -j \
    && make install

RUN rm -rf memcached-1.6.32.tar.gz \
    memcached-1.6.32

# Install Apache httpd v2.4.62
WORKDIR /root
RUN apt-get install -y --no-install-recommends libapr1-dev libaprutil1-dev libpcre2-dev
RUN wget https://dlcdn.apache.org/httpd/httpd-2.4.63.tar.bz2 \
    && tar xvf httpd-2.4.63.tar.bz2 \
    && cd httpd-2.4.63 \
    && ./configure \
    --disable-authn-file \
    --disable-authz-groupfile \
    --disable-authz-user \
    --disable-access-compat \
    --disable-auth-basic \
    --disable-reqtimeout \
    --disable-filter \
    --disable-charset-lite \
    --disable-mime \
    --disable-log-config \
    --disable-env \
    --disable-headers \
    --disable-setenvif \
    --disable-version \
    --disable-status \
    --disable-autoindex \
    --disable-dir \
    --disable-alias \
    && make -j \
    && make install

RUN rm -rf httpd-2.4.63.tar.bz2 \
    httpd-2.4.63

# Install wrk
WORKDIR /root
RUN git clone https://github.com/wg/wrk.git \
    && cd wrk \
    && make -j \
    && mkdir -p /usr/local/wrk \
    && cp wrk /usr/local/wrk

RUN rm -rf wrk

#= Build syscall test =========================================================

FROM build-base AS build-bazel

# Install bazel, which is required by the system call test suite from Gvisor project
RUN mkdir -p /root/bazel
COPY tools/install_bazel.sh /root/bazel/
WORKDIR /root/bazel
RUN ./install_bazel.sh
RUN apt clean && rm -rf /var/lib/apt/lists/*
WORKDIR /root
RUN rm -rf bazel

FROM build-bazel AS syscall_test

# Build the syscall test binaries
COPY test/syscall_test /root/syscall_test
WORKDIR /root/syscall_test
RUN export BUILD_DIR=build && \
    make ${BUILD_DIR}/syscall_test_bins

#= Build QEMU =================================================================

FROM build-base AS build-qemu

RUN apt update && apt-get install -y --no-install-recommends \
    libgcrypt-dev   `# optional build dependency` \
    libglib2.0-dev  `# build dependency` \
    libpixman-1-dev `# build dependency` \
    libusb-dev      `# optional build dependency` \
    meson \
    ninja-build
RUN apt clean && rm -rf /var/lib/apt/lists/*

FROM build-qemu AS qemu

# Fetch and install QEMU from the official source
#
# The QEMU version in the Ubuntu 22.04 repository is 6.*, which has a bug to cause OVMF debug to fail.
# The libslirp dependency is for QEMU's network backend.
WORKDIR /root
RUN wget -O qemu.tar.xz https://download.qemu.org/qemu-9.1.0.tar.xz \
    && mkdir /root/qemu \
    && tar xf qemu.tar.xz --strip-components=1 -C /root/qemu \
    && rm qemu.tar.xz
WORKDIR /root/qemu
RUN ./configure --target-list=x86_64-softmmu --prefix=/usr/local/qemu --enable-slirp \
    && make -j \
    && make install
WORKDIR /root
RUN rm -rf /root/qemu

#= Build OVMF =================================================================

FROM build-base AS build-ovmf

RUN apt update && apt-get install -y --no-install-recommends \
    bison \
    flex \
    iasl \
    nasm \
    uuid-dev
RUN apt clean && rm -rf /var/lib/apt/lists/*

FROM build-ovmf AS ovmf

# Fetch and build OVMF from the EDK2 official source
WORKDIR /root
RUN git clone --depth 1 --branch stable/202408 --recurse-submodules --shallow-submodules https://github.com/tianocore/edk2.git
WORKDIR /root/edk2
RUN /bin/bash -c "source ./edksetup.sh \
    && make -C BaseTools \
    && build -a X64 -t GCC5 -b DEBUG -p OvmfPkg/OvmfPkgX64.dsc -D DEBUG_ON_SERIAL_PORT \
    && build -a X64 -t GCC5 -b RELEASE -p OvmfPkg/OvmfPkgX64.dsc"

#= Build GRUB =================================================================

FROM build-base AS build-grub

RUN apt update && apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    autopoint \
    bison \
    flex \
    gawk \
    gettext \
    libfreetype6-dev \
    pkg-config
RUN apt clean && rm -rf /var/lib/apt/lists/*

FROM build-grub AS grub

# Fetch and install GRUB from the GNU official source
#
# We have installed grub-efi-amd64-bin just for the unicode.pf2 file, which is not included
# in the GRUB release. The Ubuntu release notoriously modifies the GRUB source code and enforce
# EFI handover boot, which is deprecated. So we have to build GRUB from source.
WORKDIR /root
# See also: https://github.com/asterinas/asterinas/pull/1710
RUN git clone --single-branch -b asterinas/2.12 https://github.com/asterinas/grub.git \
    && git -C grub checkout 0633bc8
# Fetch and install the Unicode font data for grub.
RUN wget -O unifont.pcf.gz https://unifoundry.com/pub/unifont/unifont-15.1.04/font-builds/unifont-15.1.04.pcf.gz \
    && mkdir -pv /usr/share/fonts/unifont \
    && gunzip -c unifont.pcf.gz > /usr/share/fonts/unifont/unifont.pcf \
    && rm unifont.pcf.gz
WORKDIR /root/grub
RUN echo depends bli part_gpt > grub-core/extra_deps.lst \
    && ./bootstrap \
    && ./configure \
        --target=x86_64 \
        --disable-efiemu \
        --with-platform=efi \
        --enable-grub-mkfont \
        --prefix=/usr/local/grub \
        --disable-werror \
    && make -j \
    && make install
WORKDIR /root
RUN rm -rf /root/grub

#= Build busybox ==============================================================

FROM build-base AS build-busybox

RUN apt clean && rm -rf /var/lib/apt/lists/*

FROM build-busybox AS busybox

WORKDIR /root
RUN wget -O busybox.tar.bz2 https://busybox.net/downloads/busybox-1.35.0.tar.bz2 \
    && mkdir /root/busybox \
    && tar xf busybox.tar.bz2 --strip-components=1 -C /root/busybox \
    && rm busybox.tar.bz2
WORKDIR /root/busybox
RUN make defconfig \
    && sed -i "s/# CONFIG_STATIC is not set/CONFIG_STATIC=y/g" .config \
    && sed -i "s/# CONFIG_FEATURE_SH_STANDALONE is not set/CONFIG_FEATURE_SH_STANDALONE=y/g" .config \
    && make -j

#= The final stages to produce the Asterinas development image ====================

FROM build-base AS rust

# Install Rust with both nightly and stable
ENV PATH="/root/.cargo/bin:${PATH}"
ARG ASTER_RUST_VERSION
RUN curl https://sh.rustup.rs -sSf | \
        sh -s -- --default-toolchain ${ASTER_RUST_VERSION} -y \
    && rustup toolchain install stable \
    && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git \
    && cargo -V \
    && rustup component add rust-src rustc-dev llvm-tools-preview

# Install cargo tools
RUN cargo install \
    cargo-binutils \
    mdbook \
    typos-cli

FROM rust

# Install all Asterinas dependent packages
RUN apt update && apt-get install -y --no-install-recommends \
    bridge-utils \
    clang-format       `# formatting general tests` \
    cpio \
    cpuid \
    exfatprogs \
    file \
    gdb \
    grub-efi-amd64 \
    grub-efi-amd64-bin \
    grub-efi-amd64-dbg \
    iptables \
    iproute2 \
    libnl-3-dev         `# dependency for netlink socket` \
    libnl-route-3-dev   `# dependency for netlink route socket` \
    libpixman-1-dev     `# running dependency for QEMU` \
    mtools              `# used by grub-mkrescue` \
    net-tools \
    openssh-server \
    pkg-config \
    socat \
    strace \
    sudo \
    unzip \
    vim \
    xorriso \
    zip
# Clean apt cache
RUN apt clean && rm -rf /var/lib/apt/lists/*

# Prepare the system call test suite
COPY --from=syscall_test /root/syscall_test/build/syscall_test_bins /root/syscall_test_bins
ENV ASTER_PREBUILT_SYSCALL_TEST=/root/syscall_test_bins

# Install QEMU built from the previous stages
COPY --from=qemu /usr/local/qemu /usr/local/qemu
ENV PATH="/usr/local/qemu/bin:${PATH}"
ENV LD_LIBRARY_PATH="/usr/local/qemu/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH}"

# Install OVMF built from the previous stages
COPY --from=ovmf /root/edk2/Build/OvmfX64/DEBUG_GCC5/FV/ /root/ovmf/debug
COPY --from=ovmf /root/edk2/Build/OvmfX64/RELEASE_GCC5/FV/ /root/ovmf/release

# Install GRUB built from the previous stages
COPY --from=grub /usr/local/grub /usr/local/grub
ENV PATH="/usr/local/grub/bin:${PATH}"
# Make a symbolic link for `unicode.pf2` from Ubuntu 22.04 package
RUN ln -sf /usr/share/grub/unicode.pf2 /usr/local/grub/share/grub/unicode.pf2

# Install Busybox built from the previous stages
COPY --from=busybox /root/busybox/busybox /bin/busybox

# Install benchmarks built from the previous stages
COPY --from=build-benchmarks /usr/local/benchmark /usr/local/benchmark

# Install applications built from the previous stages
COPY --from=build-applications /usr/local/redis /usr/local/redis
COPY --from=build-applications /usr/local/nginx /usr/local/nginx
COPY --from=build-applications /usr/local/leveldb /usr/local/leveldb
COPY --from=build-applications /usr/local/speedtest1 /usr/local/benchmark/sqlite-speedtest1
COPY --from=build-applications /lib/x86_64-linux-gnu/libsqlite3.so /lib/x86_64-linux-gnu/libsqlite3.so
COPY --from=build-applications /usr/local/memcached /usr/local/memcached
COPY --from=build-applications /usr/local/apache2 /usr/local/apache2
COPY --from=build-applications /usr/local/wrk /usr/local/wrk

# Add the path of Asterinas tools
ENV PATH="/root/asterinas/target/bin:${PATH}"

VOLUME [ "/root/asterinas" ]

WORKDIR /root/asterinas
