//! This build's facts, for the page's footer.

/// This build's facts as `key\tvalue` lines (see the module docs).
pub fn build_info() -> &'static str {
    static INFO: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    INFO.get_or_init(reva_vm::version::fields)
}

/// Where [`build_info`]'s text starts.
#[unsafe(no_mangle)]
pub extern "C" fn reva_build_info_ptr() -> *const u8 {
    build_info().as_ptr()
}

/// The length of [`build_info`]'s text in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn reva_build_info_len() -> u32 {
    build_info().len() as u32
}
