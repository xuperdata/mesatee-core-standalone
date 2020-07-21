# mesatee-core-standalone
---
[![Build Status](https://travis-ci.com/xuperdata/mesatee-core-standalone.svg?branch=master)](https://travis-ci.com/xuperdata/mesatee-core-standalone)

A standalone mesatee-core, with a lightweight, efficient TCP-based RPC library with remote attestation integrated, ported from [incubator-teaclave](https://github.com/apache/incubator-teaclave), enable your to compose distributed SGX apps quickly.

Now we provide a very easy-to-use approach for you to write your own [Trusted DApp](docs/xuperchain%20trusted%20ledger%20-%20chinese.md) based on [TEESDK](https://github.com/xuperdata/teesdk), [xuper-sdk-go](https://github.com/xuperdata/xuper-sdk-go) and [XuperChain](https://github.com/xuperchain/xuperchain). 

## Quick Start
Note that you must mount SGX device to use SGX feature. SGX-2.9.1 is required to run the service. Follow the [instructions](docs/SGX2.9.1%20update%20instructions.md) to install SGX driver before getting started.

## Compile
You can compile the project by yourself:
```
$ git clone https://github.com/xuperdata/mesatee-core-standalone
$ cd mesatee-core-standalone
$ docker run --name fns --net=host -v$(pwd):/teaclave -w /teaclave -it teaclave/teaclave-build-ubuntu-1804-sgx-2.9.1 bash
# mkdir -p build && cd build
# cmake -DTEST_MODE=ON .. && make
```
If you want to test it in Non-SGX server, add `-DSGX_SIM_MODE` when compiling.

## Run
Run tee service in docker:
```
# cd /teaclave/release/services
# export IAS_SPID=xxxx
# export IAS_KEY=xxx
# ./fns
```
Notice that the default port of fns is 8082.
## Run with pre-built docker image

You can run service with pre-built docker image:
 ```
$ docker run --net=host --device /dev/sgx/enclave --name fns --env-file ias -d xuperdata/fns-sgx-2.9.1:1.0
```
or (depends on your device)
 ```
$ docker run --net=host --device /dev/isgx --name fns --env-file ias -d xuperdata/fns-sgx-2.9.1:1.0
```

`ias` is the file to set environment variables:
```
IAS_SPID=xxx
IAS_KEY=xxx
```
If you want to build your own docker image, check [docker](./docker) a subdirectory for more information.

## Have a try 

After compilation, you can find an excutable file  `quickstart` in release/examples directory. This is a simple example to get started, and you can try this as follows:
```
# cd /teaclave/release/examples
# ./quickstart echo -e ./enclave_info.toml  -m "hello world"
```

## Tests 

After compilation, you can run tests as follows:
```
# cd /teaclave/build
# make sgx-test
```
Refer to [tests](./tests) for more information

## Simulation Mode

By default, the outcome is targeting a platform with SGX hardware.  In order to
switch to SGX simulation target, please set `-DSGX_SIM_MODE=ON` when running `cmake`.
```
# cd /teaclave/build
# cmake -DSGX_SIM_MODE=ON ..
# make
```
In simulation mode, mesatee-core-standalone won't really connect to IAS to fetch reports, nor perform remote attestation during the TLS communications.
So basically it enables you to freely run on arbitrary platforms to test the functionalities.

## Community 
This library is maintained by members from XuperChain team and Mesatee team collaboratively. This lib will largely enable the multiple-parties confidential computing on blockchain. 
