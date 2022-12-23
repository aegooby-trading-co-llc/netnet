#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ping {
    #[prost(uint32, tag = "1")]
    pub port: u32,
    #[prost(string, tag = "2")]
    pub uuid: ::prost::alloc::string::String,
}
