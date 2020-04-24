# mesatee-core-standalone
A standalone mesatee-core aims to provide a lightweight, efficient TCP-based RPC library with remote attestation integrated, ported from [incubator-teaclave](https://github.com/apache/incubator-teaclave).

Now we provide a very easy-to-use approach for you to write your own [trusted DApp](apps_zh.md) based on [TEESDK](https://github.com/xuperdata/teesdk), [xuper-sdk-go](https://github.com/xuperdata/xuper-sdk-go) and [XuperChain](https://github.com/xuperchain/xuperchain). 

# Quick Start

## Compile
```
git clone https://github.com/xuperdata/mesatee-core-standalone
cd mesatee-core-standalone
docker run --name core3 --net=host -v$(pwd):/teaclave -w /teaclave -it teaclave/teaclave-build-ubuntu-1804:latest bash
mkdir -p build && cd build
cmake -DTEST_MODE=ON .. && make
```
If you want to test it in Non-SGX server, add `-DSGX_SIM_MODE` when compiling.

## Have a try 

There is a `quickstart` in mesa_services directory. That's how we write a network enclave application. After compalation, you can get 2 bianries: `fns` and `quickstart`. 

```
export IAS_SPID=xxxx
export IAS_KEY=xxx

cd mesatee-core-standalone/release/services
export CARGO_PKG_NAME=fns
./fns
```
Notice that the default port of fns is 8082.

Open another terminal, and run
```
cd mesatee-core-standalone/release/examples
./quickstart echo -e ./enclave_info.toml  -m "hello, world"
```
 
## Community 
This library is maintained by members from XuperChain team and Mesatee team collaboratively. This lib will largely enable the multiple-parties confidential computing on blockchain. 
