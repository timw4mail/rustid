#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "haiku", test))]
pub mod haiku;
