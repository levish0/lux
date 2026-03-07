pub(crate) const SERVER_RUNTIME_SPECIFIER: &str = "lux/runtime/server";
pub(crate) const CLIENT_RUNTIME_SPECIFIER: &str = "lux/runtime/client";

pub(crate) fn server_runtime_source() -> &'static str {
    include_str!("server.js")
}

pub(crate) fn client_runtime_source() -> &'static str {
    include_str!("client.js")
}
