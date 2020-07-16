## Ship mesatee-core-standalone on K8S

### mesate-core-standalone Intro

The [mesate-core-standalone](https://github.com/xuperdata/mesatee-core-standalone#run) is from project [incubator-tenclave](https://github.com/apache/incubator-teaclave), but is simplified to a standalone function framework, on which you can compose your distributed SGX apps quickly without worrying about the RA. more details can find [here](https://github.com/xuperdata/mesatee-core-standalone/blob/master/README.md).

### sgx-device-plugin Intro

The [sgx-device-plugin](https://github.com/AliyunContainerService/sgx-device-plugin) enable us to run an SGX-enabled app in k8s cluster combining both the advantages of cluster orchestration from k8s and application security enhancement from IntelSGX.  

The sgx-device-plugin provides a device plugin, which maps the devices /dev/isgx  and mounts /run/aesmd/aesm.socket into the container which running in a SGX server through a cloud-native way.

For implementation a device plugin in K8S,  we need to implement DevicePlugin [interface](https://kubernetes.io/docs/concepts/extend-kubernetes/compute-storage-net/device-plugins/):

```
service DevicePlugin {
      // ListAndWatch returns a stream of List of Devices
      // Whenever a Device state change or a Device disappears, ListAndWatch
      // returns the new list
      rpc ListAndWatch(Empty) returns (stream ListAndWatchResponse) {}

      // Allocate is called during container creation so that the Device
      // Plugin can run device specific operations and instruct Kubelet
      // of the steps to make the Device available in the container
      rpc Allocate(AllocateRequest) returns (AllocateResponse) {}
}
```

Launch a socket server and register it to KubeletSocket.  Finally, Deploy this plugin as a [DaemonSet](https://github.com/AliyunContainerService/sgx-device-plugin/tree/5f5b5efb8876ba911aa607dcf7c91712a3fa2fa4#deployment).   

In order to do SGX-based app load balancing,  this plugin collects the free EPC resources, and expose it as an appliable resource sgx_epc_MiBwhen creating the container.  you can set up this resource requirement to declare how much EPC sections you want to keep for your SGX app.

Notice that memory can not be reusable usually, but ERC is reusable here, which means this kind of resource is just abstracted for load-balance, not for isolation usage.

That's almost all it provides now.  We need to do something more to make it easy to land: 

1. RA integration
2. A SGX-LibOS to support multiple language runtimes 
3. Confidential storage

### Design

RA integration should consider security and efficiency at the meantime.  Now, mesate-core-standalone had a practical approach by doing the RA in the server-side, and the client just need to check the signer identity and signature of the server-side.  

For easy-to-compose an SGX app,  mesate-core-standalone now work as a framework to provide function interface to write a built-in function easily and also enable you to access the function via an SDK or CLI tool.  However,  we are limited to develop our secure app by Rust and C/CPP, but lots of native apps are wroten by Java or Golang, which need standard libc for their runtime. At present, we found two LibOS to help migrate the native app to secure container seamlessly

- [GrapheneSGX-Golang-Support-and-Enhancement](https://github.com/intel/GrapheneSGX-Golang-Support-and-Enhancement) 
- [occulm](https://github.com/occlum/occlum)

I will give more details about those projects in the following doc.

For Confidential storage, we have two approaches. The first one is to keep the secret in our blockchain by [Private Ledger](https://github.com/xuperdata/mesatee-core-standalone/blob/master/docs/xuperchain trusted ledger - chinese.md), which is ready now but sounds heavy due to maintaining of a blockchain network. The other one is to rewrite the FS operation, and then redirect the encrypted data into distributed network FS, with the help of our KMS. The later one is light, and transparent to end-user, I will give the design doc later.  

### Demo show

1. minikube installation

minikube is a single node K8S cluster builder.  It's easy to install minikube by the [official doc](https://kubernetes.io/docs/tasks/tools/install-minikube/) if you can access to gcr.io or by [this doc](https://developer.aliyun.com/article/221687). 

1. Then start the cluster and deploy the SGX plugin by DaemonSet.

```
minikube start --driver=docker
minikube kubectl -- apply -f https://github.com/duanbing/sgx-device-plugin/blob/master/deploy/sgx-device-plugin.yaml
```

1. Deploy the mesate-core-standalone by K8S.  

```
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fns-dp
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      app: fns
  template:
    metadata:
      labels:
        app: fns
    spec:
      containers:
      - name: fns
        image: xuperdata/fns-sgx-2.9.1:1.0
        imagePullPolicy: Always
        env:
          - name: IAS_KEY
          value: {{your key}}
          - name: IAS_SPID
          value: {{your spid}}
        - name: RUST_LOG
          value: debug
        resources:
          limits:
            cpu: 250m
            memory: 512Mi
            alibabacloud.com/sgx_epc_MiB: 2
        volumeMounts:
        - mountPath: /dev/sgx/enclave
          name: sgxdevice
        securityContext:
          privileged: true
      volumes:
      - hostPath:
          path: /dev/sgx/enclave
        name: sgxdevice
```

1. Check the status of mesatee-core-standalone. 

$ minikube kubectl -- get pod    NAME                         READY   STATUS    RESTARTS   AGE    fns-dp-5bdb5b8d78-2qnpr      1/1     Running   0          31s    fns-dp-5bdb5b8d78-rl89t      1/1     Running   0          25s    sgx-device-plugin-ds-crvjf   1/1     Running   0          6h51m

1. Add a NodePort service svc.yaml:

```
# svc.yaml
apiVersion: v1
kind: Service
metadata:
  name: fns-service
spec:
  type: NodePort
  selector:
    app: fns
  ports:
    - protocol: TCP
      targetPort: 8082
      port: 8082
      nodePort: 30007
```

and check if it's work:

```
$ minikube kubectl -- get node -o wide
NAME       STATUS   ROLES    AGE   VERSION   INTERNAL-IP   EXTERNAL-IP   OS-IMAGE       KERNEL-VERSION      CONTAINER-RUNTIME
minikube   Ready    master   16h   v1.18.3   172.17.0.3    <none>        Ubuntu 19.10   4.13.0-36-generic   docker://19.3.2

$ curl 172.17.0.3:30007
curl: (52) Empty reply from server
```

you also can test it by running the unit test of [TEESDK](https://github.com/xuperdata/teesdk)

### Next Plan

Our next work is to integrate a LibOS to run Golang or Java native application by K8S with security enhancement. 
