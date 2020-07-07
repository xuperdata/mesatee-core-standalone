# Mesatee-core-standalone Docker

This directory contains the docker infrastructure for build and runtime
environment. Note that you must mount SGX device to use SGX feature. 
SGX-2.9.1 is required to run the service. Follow the [instructions](docs/SGX2.9.1升级指南.md) 
to install SGX driver before getting started.

## Build

```
$ cd mesatee-core-standalnoe/docker
$ cp -r ../release/services/* ./
$ docker build -t fns-sgx-2.9.1 .
```

## Run

```
$ docker run --net=host --device /dev/sgx/enclave --name my-fns --env-file ias -d fns-sgx-2.9.1
```
or (depends on your device)
```
$ docker run --net=host --device /dev/isgx --name my-fns --env-file ias -d fns-sgx-2.9.1
```
