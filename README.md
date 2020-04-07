# mesatee-core-standalone
A standalone mesatee-core aims to provide a lightweight, efficient TCP-based RPC library with remote attestation integrated, ported from [incubator-teaclave](https://github.com/apache/incubator-teaclave) gracefully. 

# Quick Start

## Compile
```
git clone https://github.com/xuperdata/mesatee-core-standalone
cd mesatee-core-standalone
docker run --name core3 --net=host -v$(pwd):/teaclave -w /teaclave -it teaclave/teaclave-build-ubuntu-1804:latest bash
mkdir -p build && cd build
cmake -DTEST_MODE=ON .. && make
```

## Test
```
export IAS_SPID=xxxx
export IAS_KEY=xxx

```
TBD...

# TODO
0. add simple client
1. rename mesatee_services to example
2. remove runtime config

 
