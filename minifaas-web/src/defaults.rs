pub(crate) const fn default_max_runtime_secs() -> u64 {
    300
}
pub(crate) const fn default_timer_resolution_ms() -> i64 {
    1000
}
pub(crate) const fn default_no_runtime_threads() -> usize {
    10
}
pub(crate) fn default_db_name() -> String {
    "functions.db".to_owned()
}
pub(crate) fn default_env_path() -> String {
    "/tmp".to_owned()
}
