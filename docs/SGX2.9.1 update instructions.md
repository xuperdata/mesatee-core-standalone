# 安装sgx2.9.1操作指南

## 准备工作

1、Ubuntu16.04机器（sgx2.9.1需要ubuntu18.04环境，这里将sgx安装到docker）

2、从intel [官网](https://download.01.org/intel-sgx/sgx-linux/2.9.1/distro/ubuntu18.04-server/) 下载driver安装包

3、暂停已有的aesm服务并卸载/opt/intel下的SGX driver、SGX sdk


## 一、在宿主机安装driver

1、在安装包所在目录执行以下命令安装SGX driver

```
./sgx_linux_x64_driver_1.33.bin
```

在/opt/intel可以找到SGX driver目录，即表示安装成功


## 二、安装enclave-common到docker

1、创建容器

```
docker run --net=host --device /dev/sgx/enclave --name core5 -v /root/sgx-2.9.1-ubuntu18:/deps -w /deps -it teaclave/teaclave-build-ubuntu-1804-sgx-2.9.1 bash
```

-v参数自定义，注意要将安装包映射进来。如需将mesatee-core服务也映射进来，可加入-v /root/mesatee-core-standalone:/teaclave，如下：

```
docker run --net=host --device /dev/sgx/enclave --name core5 -v /root/repo/intel-sgx-deps/sgx-2.9.1-ubuntu18:/deps -v /root/mesatee-core-standalone:/teaclave -w /deps -it teaclave/teaclave-build-ubuntu-1804-sgx-2.9.1 bash
```

步骤2-4全部在容器里面进行。

2、安装依赖环境

```
apt-get update
apt-get install -y gnupg2 apt-transport-https ca-certificates curl software-properties-common
curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add -
add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main"
apt-get update
curl -sL https://deb.nodesource.com/setup_12.x | bash -
apt install nodejs -y
```

3、安装sgx-enclave-common

```
apt-get install -y libsgx-aesm-ecdsa-plugin-dbgsym libsgx-aesm-launch-plugin libsgx-ae-pce libsgx-dcap-pccs libsgx-ae-qe3 libsgx-dcap-default-qpl-dbgsym \
libsgx-ae-qve libsgx-quote-ex libsgx-aesm-pce-plugin libsgx-epid-dev libsgx-quote-ex-dev libsgx-enclave-common-dev libsgx-aesm-ecdsa-plugin \
libsgx-aesm-epid-plugin-dbgsym libsgx-dcap-ql-dev libsgx-epid-dbgsym libsgx-aesm-quote-ex-plugin-dbgsym libsgx-aesm-launch-plugin-dbgsym \
libsgx-ae-le libsgx-epid libsgx-urts-dbgsym libsgx-enclave-common-dbgsym libsgx-aesm-epid-plugin libsgx-aesm-quote-ex-plugin libsgx-enclave-common \
libsgx-launch libsgx-launch-dbgsym libsgx-dcap-ql-dbgsym libsgx-dcap-default-qpl-dev libsgx-ae-epid libsgx-uae-service-dbgsym libsgx-launch-dev \
libsgx-aesm-pce-plugin-dbgsym libsgx-dcap-default-qpl libsgx-quote-ex-dbgsym libsgx-uae-service libsgx-urts libsgx-dcap-ql
```

4、启动aesm服务

```
LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm /opt/intel/sgx-aesm-service/aesm/aesm_service
```

### 其他
如果要在Ubuntu20.04，需要考虑降低protobuf的版本17到10.  [下载地址](https://debian.pkgs.org/9/debian-main-amd64/libprotobuf10_3.0.0-9_amd64.deb.html)。   
构建开发环境还需要安装如下工具链：
```
apt-get install build-essential
apt install libtool
apt-get install autoconf
```
