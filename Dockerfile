FROM btwiuse/arch:rust

RUN curl -sL https://github.com/btwiuse/substrate-benchmark-machine/releases/download/v0.1.3/substrate-benchmark-machine-linux-amd64 > /usr/bin/substrate-benchmark-machine && chmod +x /usr/bin/substrate-benchmark-machine

RUN substrate-benchmark-machine -f || true

CMD substrate-benchmark-machine -f