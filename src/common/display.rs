use super::cache::{Cache, Level1Cache};

use alloc::format;
use alloc::string::String;

#[cfg(target_os = "none")]
use crate::println;

pub struct CpuDisplay {
    pub color: bool,
}

impl CpuDisplay {
    pub fn raw_label(s: &str) -> String {
        format!("{:>14}: ", s)
    }

    pub fn raw_sublabel(s: &str) -> String {
        format!("{:>16}{}: ", "", s)
    }

    pub fn raw_inline_sublabel(label: &str, sub: &str) -> String {
        format!("{:>14}: {:1}: ", label, sub)
    }

    pub fn label(&self, s: &str) -> String {
        if !self.color {
            Self::raw_label(s)
        } else {
            format!("\x1b[32m{:>14}\x1b[0m: ", s)
        }
    }

    pub fn sublabel(&self, s: &str) -> String {
        if !self.color {
            Self::raw_sublabel(s)
        } else {
            format!("\x1b[94m{:>16}{}\x1b[0m: ", "", s)
        }
    }

    pub fn inline_sublabel(&self, label: &str, sub: &str) -> String {
        if !self.color {
            Self::raw_inline_sublabel(label, sub)
        } else {
            format!("\x1b[32m{:>14}\x1b[0m: \x1b[94m{:1}\x1b[0m: ", label, sub)
        }
    }

    pub fn simple_line(&self, l: &str, v: &str) {
        let l = self.label(l);
        println!("{}{}", l, v);

        #[cfg(not(target_os = "none"))]
        println!();
    }

    pub fn newline() {
        #[cfg(not(target_os = "none"))]
        println!();
    }

    pub fn format_frequency(mhz: impl Into<u64>) -> String {
        let mhz = mhz.into();
        if mhz >= 1000 {
            let whole = mhz / 1000;
            let fract = (mhz % 1000) / 10;
            format!("{}.{:02} GHz", whole, fract)
        } else {
            format!("{}.00 MHz", mhz)
        }
    }

    pub fn cache_count(share_count: u32, core_count: u32) -> String {
        if share_count == 0 || (core_count / share_count) <= 1 {
            String::new()
        } else {
            format!("{}x ", core_count / share_count)
        }
    }

    pub fn display_cache(
        &self,
        cache: Option<Cache>,
        cache_count: &dyn Fn(u32) -> String,
        l3_socket_count: u32,
    ) {
        if let Some(cache) = cache {
            match cache.l1 {
                Level1Cache::Unified(l1) => {
                    println!("{}L1: Unified {:>4} KB", self.label("Cache"), l1.size);
                }
                Level1Cache::Split { data, instruction } => {
                    let data_count: String = cache_count(data.share_count);
                    let instruction_count: String = cache_count(instruction.share_count);

                    if data.assoc > 0 {
                        println!(
                            "{}{}{} KB, {}-way",
                            self.inline_sublabel("Cache", "L1d"),
                            &data_count,
                            data.size / 1024,
                            data.assoc
                        );
                    } else {
                        println!(
                            "{}{}{} KB",
                            self.inline_sublabel("Cache", "L1d"),
                            &data_count,
                            data.size / 1024
                        );
                    }

                    if instruction.assoc > 0 {
                        println!(
                            "{}{}{} KB, {}-way",
                            self.sublabel("L1i"),
                            &instruction_count,
                            instruction.size / 1024,
                            instruction.assoc
                        );
                    } else {
                        println!(
                            "{}{}{} KB",
                            self.sublabel("L1i"),
                            &instruction_count,
                            instruction.size / 1024,
                        );
                    }
                }
            }

            if let Some(l2) = cache.l2 {
                let count = cache_count(l2.share_count);
                let (num, unit) = Self::cache_size(l2.size);

                if l2.assoc > 0 {
                    println!(
                        "{} {}{} {}, {}-way",
                        self.sublabel("L2"),
                        &count,
                        num,
                        unit,
                        l2.assoc
                    );
                } else {
                    println!("{} {}{} {}", self.sublabel("L2"), &count, num, unit);
                }
            }

            if let Some(l3) = cache.l3 {
                let (num, unit) = Self::cache_size(l3.size);
                let count: String = if l3_socket_count > 1 {
                    format!("{}x ", l3_socket_count)
                } else {
                    cache_count(l3.share_count)
                };

                if l3.assoc > 0 {
                    println!(
                        "{} {}{} {}, {}-way",
                        self.sublabel("L3"),
                        &count,
                        num,
                        unit,
                        l3.assoc
                    );
                } else {
                    println!("{} {}{} {}", self.sublabel("L3"), &count, num, unit);
                }
            }
        }
        Self::newline();
    }

    #[inline]
    fn cache_size(raw_size: u32) -> (u32, &'static str) {
        let mut num = raw_size / 1024;
        let unit = if num >= 1024 { "MB" } else { "KB" };

        if num >= 1024 {
            num /= 1024;
        }

        (num, unit)
    }
}
