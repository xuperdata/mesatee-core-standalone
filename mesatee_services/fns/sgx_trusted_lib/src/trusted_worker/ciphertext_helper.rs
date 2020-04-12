#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CipherText {
    #[prost(uint32, tag="1")]
    pub svn: u32,
    #[prost(bytes, tag="2")]
    pub content: std::vec::Vec<u8>,
    /// 拥有者地址的hash
    #[prost(bytes, tag="3")]
    pub address: std::vec::Vec<u8>,
    #[prost(bytes, tag="4")]
    pub args_hash: std::vec::Vec<u8>,
}
