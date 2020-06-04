### TEE是什么？ 

TEE的全称是可信计算环境， MesaTEE提供了一种内存安全的编程模式，结合Intel SGX实现内存安全的可信计算，相对MPC，HM等基于计算复杂性理论的算法，TEE具备性能高，编程复杂度低，进而更加容易实现多种复杂的计算任务，例如SQL分析，机器学习等。

### 超级链可信账本

超级链基于TEE技术实现合约数据加密存储和密文计算，我们称这个产品为超级链可信账本。目前我们开源了核心的组件：

1. TEE-SDK: https://github.com/xuperdata/teesdk

   负责实现超级链SDK，超级链跟TEE环境的通信。

2. 超级链SDK:  https://github.com/xuperdata/xuper-sdk-go

   负责交易的封装，加密和解密

3. xuperchain: https://github.com/xuperchain/xuperchain

   超级链开源代码

4. mesatee-core: https://github.com/xuperdata/mesatee-core-standalone

   可以说是一个TEE Enclave App的开发框架。结合[SGX RUST SDK](https://github.com/apache/incubator-teaclave-sgx-sdk), 以及[SGX Crates](https://github.com/universal-secure-computing-community/crates-sgx) 可以快速实现可信账本。

当然，没有开源的商业组件包括：

1. 秘钥管理（KMS）：我们提供了高安全，可更新，可分发的秘钥托管系统，保证账本的安全性，可以安全抵抗选择明文攻击。
2. 基本算子以及加解密功能：这部分实现非常简单，但是目前因为跟KMS有耦合，后面会进行解耦，并且陆续开源，这里也是大家可以参与的部分
3. 链上数据协同商业化解决方案： 我们针对不同的政务场景，金融场景等提供了数据可信上链，多方安全计算等解决方案。

### 超级链可信账本可以干什么？

基于超级链和mesatee实现链上合约数据加密存储以及基本运算：

- 加密存储： 数据加密存储在合约里面，并且数据提供者具备对数据的**所有权**。同时支持密文数据所有权共享。

- 基本运算： 例如密文加法，减法以及乘法，比较运算等，我们统一称之为**隐私计算**算子。同时可以使用mesatee-core快速扩展算子，实现包括支持图灵完备计算，以及多方安全计算。

  实际上上面提到的基本功能几乎可以让你实现任何复杂的功能。

### 应用场景

1. 秘钥托管

   利用TEE托管私钥，是目前市场上最普遍的做法；

2. 链上隐私计算

   [XuperData](https://xchain.baidu.com/n/case/xuperdata) 提供了支持复杂运算的链下可信计算, 可信账本可以让你实现链上密文存储和计算。例如， 姚氏百万富翁问题, 安全ID求交（数据重合度分析）等。在政务数据共享领域，经常碰到数据安全交换的需求，希望实现所谓的”可以分享数据，但是不能篡改数据“，本质上就是如何保证数据的所有权的前提下，挖掘数据的价值，放心的让其他方使用你的数据；

3. 可信随机数

   利用TEE可以生成可信随机数。利用TEE生成随机数，然后联盟节点之间的不同TEE正对各自生成的随机数进行共识，有了可信随机数，那么我们就可以支持多种竞猜类的Dapp，是的区块链上的玩法大大增加。

### 怎么试用？

如果要在非SGX服务器上面试用，请在编译mesatee-core的时候，开启模拟模式。

#### 流程和原理

1. 部署过程

   1. 下载  https://github.com/xuperdata/mesatee-core-standalone

   2. 部署你自己实现的app，并且部署到mesatee_services/fns/sgx_trusted_lib

   3. 按照步骤2的文档进行编译，然后启动。

      ```
      export IAS_SPID=your IAS SPID
      export IAS_KEY=your IAS KEY
      export RUST_LOG=debug  //可选
      ```

   注意FNS的默认端口是8082.

   4. 编译TEESDK

      ```
      git clone https://github.com/xuperdata/teesdk
      cd teesdk/mesatee
      cp $HOME/mesatee-core-standalone/release/lib/libmesatee_sdk_c.so lib/
      cd ../
      bash build.sh
      ```

      编译之后会在build目录产出libmesateesdk.so.0.0.1， 然后将这个文件和mesatee/xchain-plugin/teeconfig.conf拷贝到xchain的pluginPath配置的目录下面，

   5. 拉取超级链最新代码： https://github.com/xuperchain/xuperchain , 注意编译的时候把 makefile的 **-mod=vendor**去掉，编译超级链，并且在xchain.conf增加如下配置：

      ```
      # 块广播模式
      blockBroadcaseMode: 0
      ...
      #可信环境的入口, optional
      wasm:
       driver: "xvm"
       enableUpgrade: false
       teeConfig:
         enable: on
         pluginPath: "/root/private-ledger-go-api/xchain_plugin/libmesateesdk.so.0.0.1"
         configPath: "/root/private-ledger-go-api/xchain_plugin/teeconfig.conf"
       xvm:
         optLevel: 0
         
      #是否开启默认的XEndorser背书服务
      enableXEndorser: true
      ```

   6. 拉取超级链SDK最新的代码。配置sdk.yaml.tee

      ```
      tfConfig:
        teeConfig:
          svn: 0
          enable: on
          tmsport: 8082
          uid: "uid1"
          token: "token1"
          auditors:
            -
              publicder: /root/mesatee-core-standalone/release/services/auditors/godzilla/godzilla.public.der
              sign: /root/mesatee-core-standalone/release/services/auditors/godzilla/godzilla.sign.sha256
              enclaveinfoconfig: /root/mesatee-core-standalone/release/services/enclave_info.toml
      paillierConfig:
        enable: off
      ```

      

2. 测试

可信应用开发参考合约[trustops](https://github.com/xuperchain/xuperchain/tree/master/core/contractsdk/cpp/example/trustops)；可信合约相关测试参考[trust_counter](https://github.com/xuperdata/xuper-sdk-go/blob/master/example/main_trust_counter.go)；mesatee-core服务相关测试参考[teesdk_test](https://github.com/xuperdata/teesdk/blob/master/mesatee/teesdk_test.go)。

可信合约的执行流程和原理如下：data_auth合约中的方法使用了TrustOperators可信算子，TrustOperators会通过tfcall调用外部SDK，这时会调用到我们提前注册好的teesdk。teesdk通过cgo实现了链的go代码对mesatee-core-standalone的c_sdk的调用，最后实现了mesatee_service的TEE服务调用。

#### 开发智能合约

```
以下部分不是全部开源。
```
tee通过kds和合约参数派生出的对称秘钥对密文数据进行加密，tee目前支持kds的初始化、派生运算和升级，具体方法如下：

| 方法名称  | 入参              | 处理过程          | 返回          |
| :-------- | :-------------- | :----------------- | :------------- |
| init     | kds, svn    | 根据给定的svn初始化对应的kds    | svn，即当前的svn |
| mint | bds, svn  | 根据bds(即根kds)和给定的svn，派生出对应的kds  | kds，即派生得到的kds |
| inc     | kds, svn | 升级kds到给定的svn版本  | svn，即当前的svn|

隐私应用目前支持密文的二元运算(加法、减法、乘法)、数据授权(所属权、使用权)，具体可以参考可信算子的实现trust_operators. 

在data_auth合约里，我们定义了一个表，用于存储隐私数据。
每一行指定一个数据id、拥有者、密文内容、过期时间、user地址、授权信息。
owner使用自己数据时不需要commitment，所以当owner=user时，commitment字段为空。

| dataid | owner | content | exp_time | user  | commitment  |
| :----- | :---- | :------ | :------- | :---- | :---------- |
| 111    | owner | @#%!^@  | 20201010 | owner |             |
| 111    | owner | @#%!^@  | 20201010 | user1 | commiment1  |
| 111    | owner | @#%!^@  | 20201010 | user2 | commitment2 |

合约中定义了9个方法，包括数据增删改查、数据所有权和使用权授予、数据的二元运算。主要方法如下：

| 方法名称  | 入参                                   | 处理过程                                                     | 返回                 |
| :-------- | :------------------------------------- | :----------------------------------------------------------- | :------------------- |
| store     | dataid, content, expire,auth           | 插入记录，user是自己的地址                                   | “done” / "failed..." |
| authorize | dataid, user, pubkey, signature        | 调用算子计算给user的commitment，添加新的一行数据             | "done" / "failed..." |
| share     | dataid, addr, newid, pubkey, signature | 利用算子重新加密数据，将数据newid插入表中并赋予新的owner     | "done" / "failed..." |
| add/sub/mul | dataid1, dataid2, newid                | 取出两个密文和对应的commitment，调用可信算子add/sub/mul方法，返回密文后添加新的一行数据 | "done" / "failed..." |

用户可以在表中存储自己的加密数据，需要授权时调用authorize方法，对user进行权限授予，授权信息会存在表中。用户可以使用表中的加密数据，调用add、sub、mul方法进行隐私计算，得到的结果也会加密存到表中。

```
以此为参考，用户可根据自身需要开发可信应用。  
```

更全的指令设计正在进行中。参考： [Trusted Ledger Instruction Initiative](https://github.com/xuperdata/mesatee-core-standalone/wiki/Proposal:-Trusted-Ledger-Instruction)
