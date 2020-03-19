# mesatee-core-standalone
A standalone mesatee-core aims to provide a lightweight, efficient TCP-based RPC library with remote attestation integrated, ported from [incubator-teaclave](https://github.com/apache/incubator-teaclave) gracefully. 

Ensure `IAS_ROOT_CA_CERT_PATH` being setted before import this lib by  
```
export IAS_SPID=xxxx
export IAS_KEY=xxx
export IAS_ROOT_CA_CERT_PATH = /path/to/root/cert or /var/mesatee/ias_root_ca_cert.pem default
```
 
