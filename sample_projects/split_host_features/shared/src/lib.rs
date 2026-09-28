pub fn host_only() -> bool {
    cfg!(feature = "host-only")
}
