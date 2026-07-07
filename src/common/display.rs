use super::cache::{Cache, Level1Cache};
use super::constants::*;

use alloc::format;
use alloc::string::String;

use crate::common::CliFlags;
#[cfg(dos)]
use crate::println;

pub struct CpuDisplay {
    pub flags: CliFlags,
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
        if !self.flags.color {
            Self::raw_label(s)
        } else {
            format!("{}{s:>14}{}: ", Self::ansi(ANSI_GREEN), ANSI_RESET)
        }
    }

    pub fn sublabel(&self, s: &str) -> String {
        if !self.flags.color {
            Self::raw_sublabel(s)
        } else {
            format!(
                "{:>16}{}{s}{}: ",
                "",
                Self::ansi(ANSI_BRIGHT_BLUE),
                ANSI_RESET
            )
        }
    }

    pub fn inline_sublabel(&self, label: &str, sub: &str) -> String {
        if !self.flags.color {
            Self::raw_inline_sublabel(label, sub)
        } else {
            format!(
                "{}{label:>14}{}: {}{sub:1}{}: ",
                Self::ansi(ANSI_GREEN),
                ANSI_RESET,
                Self::ansi(ANSI_BRIGHT_BLUE),
                ANSI_RESET
            )
        }
    }

    pub fn ansi(code: &str) -> String {
        format!("\x1b[{code}m")
    }

    pub fn ansi_color(code: &str, s: &str) -> String {
        format!("{}{s}{ANSI_RESET}", Self::ansi(code))
    }

    pub fn simple_line(&self, l: &str, v: &str) {
        let l = self.label(l);
        println!("{}{}", l, v);

        #[cfg(not(dos))]
        println!();
    }

    pub fn newline() {
        #[cfg(not(dos))]
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

            Self::newline();
        }
    }

    /// Format the system name if it is a Mac, or other known string
    pub fn format_system_name(&self, raw: &str) -> String {
        // Based on <https://github.com/fastfetch-cli/fastfetch/blob/dev/src/detection/host/host_mac.c>
        let model = match raw {
            "MacBookPro18,3" | "MacBookPro18,4" => "MacBook Pro (14-inch, 2021)",
            "MacBookPro18,1" | "MacBookPro18,2" => "MacBook Pro (16-inch, 2021)",
            "MacBookPro17,1" => "MacBook Pro (13-inch, 2020)",
            "MacBookPro16,3" => "MacBook Pro (13-inch, 2020, Two Thunderbolt 3 ports)",
            "MacBookPro16,2" => "MacBook Pro (13-inch, 2020, Four Thunderbolt 3 ports)",
            "MacBookPro16,4" | "MacBookPro16,1" => "MacBook Pro (16-inch, 2019)",
            "MacBookPro15,4" => "MacBook Pro (13-inch, 2019, Two Thunderbolt 3 ports)",
            "MacBookPro15,3" => "MacBook Pro (15-inch, 2019)",
            "MacBookPro15,2" => "MacBook Pro (13-inch, 2018/2019, Four Thunderbolt 3 ports)",
            "MacBookPro15,1" => "MacBook Pro (15-inch, 2018/2019)",
            "MacBookPro14,3" => "MacBook Pro (15-inch, 2017)",
            "MacBookPro14,2" => "MacBook Pro (13-inch, 2017, Four Thunderbolt 3 ports)",
            "MacBookPro14,1" => "MacBook Pro (13-inch, 2017, Two Thunderbolt 3 ports)",
            "MacBookPro13,3" => "MacBook Pro (15-inch, 2016)",
            "MacBookPro13,2" => "MacBook Pro (13-inch, 2016, Four Thunderbolt 3 ports)",
            "MacBookPro13,1" => "MacBook Pro (13-inch, 2016, Two Thunderbolt 3 ports)",
            "MacBookPro12,1" => "MacBook Pro (Retina, 13-inch, Early 2015)",
            "MacBookPro11,4" | "MacBookPro11,5" => "MacBook Pro (Retina, 15-inch, Mid 2015)",
            "MacBookPro11,2" | "MacBookPro11,3" => {
                "MacBook Pro (Retina, 15-inch, Late 2013/Mid 2014)"
            }
            "MacBookPro11,1" => "MacBook Pro (Retina, 13-inch, Late 2013/Mid 2014)",
            "MacBookPro10,2" => "MacBook Pro (Retina, 13-inch, Late 2012/Early 2013)",
            "MacBookPro10,1" => "MacBook Pro (Retina, 15-inch, Mid 2012/Early 2013)",
            "MacBookPro9,2" => "MacBook Pro (13-inch, Mid 2012)",
            "MacBookPro9,1" => "MacBook Pro (15-inch, Mid 2012)",
            "MacBookPro8,3" => "MacBook Pro (17-inch, 2011)",
            "MacBookPro8,2" => "MacBook Pro (15-inch, 2011)",
            "MacBookPro8,1" => "MacBook Pro (13-inch, 2011)",
            "MacBookPro7,1" => "MacBook Pro (13-inch, Mid 2010)",
            "MacBookPro6,2" => "MacBook Pro (15-inch, Mid 2010)",
            "MacBookPro6,1" => "MacBook Pro (17-inch, Mid 2010)",
            "MacBookPro5,5" => "MacBook Pro (13-inch, Mid 2009)",
            "MacBookPro5,3" => "MacBook Pro (15-inch, Mid 2009)",
            "MacBookPro5,2" => "MacBook Pro (17-inch, Mid/Early 2009)",
            "MacBookPro5,1" => "MacBook Pro (15-inch, Late 2008)",
            "MacBookPro4,1" => "MacBook Pro (17/15-inch, Early 2008)",
            "MacBookAir10,1" => "MacBook Air (2020)",
            "MacBookAir9,1" => "MacBook Air (Retina, 13-inch, 2020)",
            "MacBookAir8,2" => "MacBook Air (Retina, 13-inch, 2019)",
            "MacBookAir8,1" => "MacBook Air (Retina, 13-inch, 2018)",
            "MacBookAir7,2" => "MacBook Air (13-inch, Early 2015/2017)",
            "MacBookAir7,1" => "MacBook Air (11-inch, Early 2015)",
            "MacBookAir6,2" => "MacBook Air (13-inch, Mid 2013/Early 2014)",
            "MacBookAir6,1" => "MacBook Air (11-inch, Mid 2013/Early 2014)",
            "MacBookAir5,2" => "MacBook Air (13-inch, Mid 2012)",
            "MacBookAir5,1" => "MacBook Air (11-inch, Mid 2012)",
            "MacBookAir4,2" => "MacBook Air (13-inch, Mid 2011)",
            "MacBookAir4,1" => "MacBook Air (11-inch, Mid 2011)",
            "MacBookAir3,2" => "MacBook Air (13-inch, Late 2010)",
            "MacBookAir3,1" => "MacBook Air (11-inch, Late 2010)",
            "MacBookAir2,1" => "MacBook Air (Mid 2009)",
            "Macmini9,1" => "Mac mini (2020)",
            "Macmini8,1" => "Mac mini (2018)",
            "Macmini7,1" => "Mac mini (Mid 2014)",
            "Macmini6,1" | "Macmini6,2" => "Mac mini (Late 2012)",
            "Macmini5,1" | "Macmini5,2" => "Mac mini (Mid 2011)",
            "Macmini4,1" => "Mac mini (Mid 2010)",
            "Macmini3,1" => "Mac mini (Early/Late 2009)",
            "MacBook10,1" => "MacBook (Retina, 12-inch, 2017)",
            "MacBook9,1" => "MacBook (Retina, 12-inch, Early 2016)",
            "MacBook8,1" => "MacBook (Retina, 12-inch, Early 2015)",
            "MacBook7,1" => "MacBook (13-inch, Mid 2010)",
            "MacBook6,1" => "MacBook (13-inch, Late 2009)",
            "MacBook5,2" => "MacBook (13-inch, Early/Mid 2009)",
            "MacPro7,1" => "Mac Pro (2019)",
            "MacPro6,1" => "Mac Pro (Late 2013)",
            "MacPro5,1" => "Mac Pro (Mid 2010 - Mid 2012)",
            "MacPro4,1" => "Mac Pro (Early 2009)",
            "Mac17,9" => "MacBook Pro (14-inch, 2026)",
            "Mac17,8" => "MacBook Pro (16-inch, 2026)",
            "Mac17,7" => "MacBook Pro (14-inch, 2026)",
            "Mac17,6" => "MacBook Pro (16-inch, 2026)",
            "Mac17,5" => "MacBook Neo (13-inch, 2026)",
            "Mac17,4" => "MacBook Air (15-inch, 2026)",
            "Mac17,3" => "MacBook Air (13-inch, 2026)",
            "Mac17,2" => "MacBook Pro (14-inch, 2025)",
            "Mac16,13" => "MacBook Air (15-inch, 2025)",
            "Mac16,12" => "MacBook Air (13-inch, 2025)",
            "Mac16,11" | "Mac16,10" => "Mac Mini (2024)",
            "Mac16,9" => "Mac Studio (2025)",
            "Mac16,3" => "iMac (24-inch, 2024, Four Thunderbolt / USB 4 ports)",
            "Mac16,2" => "iMac (24-inch, 2024, Two Thunderbolt / USB 4 ports)",
            "Mac16,1" => "MacBook Pro (14-inch, 2024, Three Thunderbolt 4 ports)",
            "Mac16,6" | "Mac16,8" => "MacBook Pro (14-inch, 2024, Three Thunderbolt 5 ports)",
            "Mac16,7" | "Mac16,5" => "MacBook Pro (16-inch, 2024, Three Thunderbolt 5 ports)",
            "Mac15,14" => "Mac Studio (2025)",
            "Mac15,13" => "MacBook Air (15-inch, 2024)",
            "Mac15,12" => "MacBook Air (13-inch, 2024)",
            "Mac15,3" => "MacBook Pro (14-inch, Nov 2023, Two Thunderbolt / USB 4 ports)",
            "Mac15,4" => "iMac (24-inch, 2023, Two Thunderbolt / USB 4 ports)",
            "Mac15,5" => "iMac (24-inch, 2023, Two Thunderbolt / USB 4 ports, Two USB 3 ports)",
            "Mac15,6" | "Mac15,8" | "Mac15,10" => {
                "MacBook Pro (14-inch, Nov 2023, Three Thunderbolt 4 ports)"
            }
            "Mac15,7" | "Mac15,9" | "Mac15,11" => {
                "MacBook Pro (16-inch, Nov 2023, Three Thunderbolt 4 ports)"
            }
            "Mac14,15" => "MacBook Air (15-inch, 2023)",
            "Mac14,14" => "Mac Studio (2023, Two Thunderbolt 4 front ports)",
            "Mac14,13" => "Mac Studio (2023, Two USB-C front ports)",
            "Mac14,8" => "Mac Pro (2023)",
            "Mac14,6" | "Mac14,10" => "MacBook Pro (16-inch, 2023)",
            "Mac14,5" | "Mac14,9" => "MacBook Pro (14-inch, 2023)",
            "Mac14,3" => "Mac mini (2023, Two Thunderbolt 4 ports)",
            "Mac14,12" => "Mac mini (2023, Four Thunderbolt 4 ports)",
            "Mac14,7" => "MacBook Pro (13-inch, 2022)",
            "Mac14,2" => "MacBook Air (2022)",
            "Mac13,1" => "Mac Studio (2022, Two USB-C front ports)",
            "Mac13,2" => "Mac Studio (2022, Two Thunderbolt 4 front ports)",
            "iMac21,1" => "iMac (24-inch, 2021, Two Thunderbolt / USB 4 ports, Two USB 3 ports)",
            "iMac21,2" => "iMac (24-inch, 2021, Two Thunderbolt / USB 4 ports)",
            "iMac20,1" | "iMac20,2" => "iMac (Retina 5K, 27-inch, 2020)",
            "iMac19,1" => "iMac (Retina 5K, 27-inch, 2019)",
            "iMac19,2" => "iMac (Retina 4K, 21.5-inch, 2019)",
            "iMacPro1,1" => "iMac Pro (2017)",
            "iMac18,3" => "iMac (Retina 5K, 27-inch, 2017)",
            "iMac18,2" => "iMac (Retina 4K, 21.5-inch, 2017)",
            "iMac18,1" => "iMac (21.5-inch, 2017)",
            "iMac17,1" => "iMac (Retina 5K, 27-inch, Late 2015)",
            "iMac16,2" => "iMac (Retina 4K, 21.5-inch, Late 2015)",
            "iMac16,1" => "iMac (21.5-inch, Late 2015)",
            "iMac15,1" => "iMac (Retina 5K, 27-inch, Late 2014 - Mid 2015)",
            "iMac14,4" => "iMac (21.5-inch, Mid 2014)",
            "iMac14,2" => "iMac (27-inch, Late 2013)",
            "iMac14,1" => "iMac (21.5-inch, Late 2013)",
            "iMac13,2" => "iMac (27-inch, Late 2012)",
            "iMac13,1" => "iMac (21.5-inch, Late 2012)",
            "iMac12,2" => "iMac (27-inch, Mid 2011)",
            "iMac12,1" => "iMac (21.5-inch, Mid 2011)",
            "iMac11,3" => "iMac (27-inch, Mid 2010)",
            "iMac11,2" => "iMac (21.5-inch, Mid 2010)",
            "iMac10,1" => "iMac (27/21.5-inch, Late 2009)",
            "iMac9,1" => "iMac (24/20-inch, Early 2009)",
            _ => raw,
        };

        String::from(model)
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
