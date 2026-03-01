pub(crate) const SERVER_RUNTIME_SPECIFIER: &str = "lux/runtime/server";

pub(crate) fn server_runtime_source() -> &'static str {
    include_str!("server.js")
}
