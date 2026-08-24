use super::cache::{Cache, Level1Cache};
use super::constants::*;

use alloc::format;
use alloc::string::String;

use crate::common::CliFlags;
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

    /// Outputs a formatted label and value with an additional newline if flags.compact is false
    pub fn simple_line(&self, l: &str, v: &str) {
        self.section_line(l, v);

        self.newline();
    }

    /// Outputs a formatted label and value
    pub fn section_line(&self, l: &str, v: &str) {
        let l = self.label(l);
        println!("{}{}", l, v);
    }

    pub fn newline(&self) {
        #[cfg(not(any(dos, dos32a)))]
        if !self.flags.compact {
            println!();
        }
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
        self.display_cache_ext(cache, cache_count, l3_socket_count, None);
    }

    pub fn display_cache_ext(
        &self,
        cache: Option<Cache>,
        cache_count: &dyn Fn(u32) -> String,
        l3_socket_count: u32,
        l3_override: Option<&str>,
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
                            data_count,
                            data.size / 1024,
                            data.assoc
                        );
                    } else {
                        println!(
                            "{}{}{} KB",
                            self.inline_sublabel("Cache", "L1d"),
                            data_count,
                            data.size / 1024
                        );
                    }

                    if instruction.assoc > 0 {
                        println!(
                            "{}{}{} KB, {}-way",
                            self.sublabel("L1i"),
                            instruction_count,
                            instruction.size / 1024,
                            instruction.assoc
                        );
                    } else {
                        println!(
                            "{}{}{} KB",
                            self.sublabel("L1i"),
                            instruction_count,
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
                        count,
                        num,
                        unit,
                        l2.assoc
                    );
                } else {
                    println!("{} {}{} {}", self.sublabel("L2"), count, num, unit);
                }
            }

            if let Some(l3) = cache.l3 {
                if let Some(override_str) = l3_override {
                    println!("{} {}", self.sublabel("L3"), override_str);
                } else {
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
                            count,
                            num,
                            unit,
                            l3.assoc
                        );
                    } else {
                        println!("{} {}{} {}", self.sublabel("L3"), count, num, unit);
                    }
                }
            }

            self.newline();
        }
    }

    #[cfg(not(dos_os))]
    pub fn display_system(&self, system: &str, flags: CliFlags) {
        let formatted = &self.format_system_name(system);

        if flags.verbose && system != formatted {
            self.section_line("System", formatted);
            self.section_line("System (raw)", system);
            self.newline();
        } else {
            self.simple_line("System", formatted);
        }
    }

    /// Format the system name if it is a Mac, or other known string
    pub fn format_system_name(&self, raw: &str) -> String {
        // Based on <https://github.com/fastfetch-cli/fastfetch/blob/dev/src/detection/host/host_mac.c>
        // Additional models based on information in MacTracker App
        let model = match raw {
            // Arm64 (Apple Silicon)
            "MacBookPro18,3" | "MacBookPro18,4" => "MacBook Pro (14-inch, 2021)",
            "MacBookPro18,1" | "MacBookPro18,2" => "MacBook Pro (16-inch, 2021)",
            "MacBookPro17,1" => "MacBook Pro (13-inch, M1, 2020)",
            "MacBookAir10,1" => "MacBook Air (M1, 2020)",
            "Macmini9,1" => "Mac mini (M1, 2020)",
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

            // Intel
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
            "MacBookPro4,1" => "MacBook Pro (15/17-inch, Early 2008)",
            "MacBookPro3,1" => "MacBook Pro (15/17-inch Mid/Late 2007)",
            "MacBookPro2,2" => "MacBook Pro (15-inch Core 2 Duo)",
            "MacBookPro2,1" => "MacBook Pro (17-inch Core 2 Duo)",
            "MacBookPro1,2" => "MacBook Pro (17-inch)",
            "MacBookPro1,1" => "MacBook Pro",
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
            "MacBookAir2,1" => "MacBook Air (Late 2008/Mid 2009)",
            "MacBookAir1,1" => "MacBook Air",
            "Macmini8,1" => "Mac mini (2018)",
            "Macmini7,1" => "Mac mini (Mid 2014)",
            "Macmini6,1" | "Macmini6,2" => "Mac mini (Late 2012)",
            "Macmini5,1" | "Macmini5,2" => "Mac mini (Mid 2011)",
            "Macmini4,1" => "Mac mini (Mid 2010)",
            "Macmini3,1" => "Mac mini (Early/Late 2009)",
            "Macmini2,1" => "Mac mini (Mid 2007)",
            "Macmini1,1" => "Mac mini (Early/Late 2006)",
            "MacBook10,1" => "MacBook (Retina, 12-inch, 2017)",
            "MacBook9,1" => "MacBook (Retina, 12-inch, Early 2016)",
            "MacBook8,1" => "MacBook (Retina, 12-inch, Early 2015)",
            "MacBook7,1" => "MacBook (13-inch, Mid 2010)",
            "MacBook6,1" => "MacBook (13-inch, Late 2009)",
            "MacBook5,2" => "MacBook (13-inch, Early/Mid 2009)",
            "MacBook5,1" => "MacBook (13-inch, Aluminum, Late 2008)",
            "MacBook4,1" => "MacBook (13-inch, 2008)",
            "MacBook3,1" => "MacBook (13-inch, Late 2007)",
            "MacBook2,1" => "MacBook (13-inch, Late 2006/Mid 2007)",
            "MacBook1,1" => "MacBook (13-inch)",
            "MacPro7,1" => "Mac Pro (2019)",
            "MacPro6,1" => "Mac Pro (Late 2013)",
            "MacPro5,1" => "Mac Pro (Mid 2010 - Mid 2012)",
            "MacPro4,1" => "Mac Pro (Early 2009)",
            "MacPro3,1" => "Mac Pro (Early 2008)",
            "MacPro2,1" => "Mac Pro (8-core)",
            "MacPro1,1" => "Mac Pro",
            "iMacPro1,1" => "iMac Pro (2017)",
            "iMac20,1" | "iMac20,2" => "iMac (Retina 5K, 27-inch, 2020)",
            "iMac19,1" => "iMac (Retina 5K, 27-inch, 2019)",
            "iMac19,2" => "iMac (Retina 4K, 21.5-inch, 2019)",
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
            "iMac11,1" => "iMac (27-inch, Late 2009)",
            "iMac10,1" => "iMac (27/21.5-inch, Late 2009)",
            "iMac9,1" => "iMac (24/20-inch, Early/Mid 2009)",
            "iMac8,1" => "iMach (20/24-inch Early 2008)",
            "iMac7,1" => "iMac (20/24-inch Mid 2007)",
            "iMac6,1" => "iMac (24-inch)",
            "iMac5,2" => "iMac (17-inch Late 2006 CD)",
            "iMac5,1" => "iMac (Late 2006)",
            "iMac4,2" => "iMac (Mid 2006 17-inch)",
            "iMac4,1" => "iMac (Early 2006)",
            "Xserve3,1" => "XServe (Early 2009)",
            "Xserve2,1" => "XServe (Early 2008)",
            "Xserve1,1" => "XServe (Late 2006)",

            // PowerPC
            "iMac,1" => "iMac (Original, 5 Flavors)",
            "PowerMac12,1" => "iMac G5 (17/20-inch iSight)",
            "PowerMac11,2" => "Power Mac G5 (Late 2005)",
            "PowerMac10,2" => "Mac mini (Late 2005)",
            "PowerMac10,1" => "Mac mini",
            "PowerMac9,1" => "Power Mac G5 (Late 2004)",
            "PowerMac8,2" => "iMac G5 (Ambient Light Sensor)",
            "PowerMac8,1" => "iMac G5 (17/20-inch)",
            "PowerMac7,3" => "Power Mac G5 (June 2004 | Early 2005)",
            "PowerMac7,2" => "Power Mac G5 (June 2003 | Early 2005)",
            "PowerMac6,4" => "eMac (USB 2.0 | 2005)",
            "PowerMac6,3" => "iMac (15/17/20-inch USB 2.0)",
            "PowerMac6,1" => "iMac (15/17-inch Flat Panel, 1GHz/USB 2.0)",
            "PowerMac5,1" => "Power Mac G4 Cube",
            "PowerMac4,5" => "iMac (17-inch Flat Panel)",
            "PowerMac4,4" => "eMac",
            "PowerMac4,2" => "iMac (15-inch Flat Panel)",
            "PowerMac4,1" => "iMac (Early/Summer 2001)",
            "PowerMac3,6" => "Power Mac G4 (FW 800 | Mirrored Drive Doors)",
            "PowerMac3,5" => "Power Mac G4 (Quicksilver)",
            "PowerMac3,4" => "Power Mac G4 (Digital Audio)",
            "PowerMac3,3" => "Power Mac G4 (Gigabit Ethernet)",
            "PowerMac3,1" => "Power Mac G4 (AGP Graphics)",
            "PowerMac2,2" => "iMac/iMac DV/iMac DV+/iMac DV SE (Summer 2000)",
            "PowerMac2,1" => "iMac/iMac DV/iMac DV SE (Slot Loading)",
            "PowerMac1,2" => "Power Mac G4 (PCI Graphics)",
            "PowerMac1,1" => "Power Macintosh G3 (Blue and White)",
            "PowerBook6,8" => "PowerBook G4 (12-inch 1.5GHz)",
            "PowerBook6,7" => "iBook G4 (Mid 2005)",
            "PowerBook6,5" => "iBook G4 (2004)",
            "PowerBook6,4" => "PowerBook G4 (12-inch 1.33GHz)",
            "PowerBook6,3" => "iBook G4",
            "PowerBook6,2" => "PowerBook G4 (12-inch DVI)",
            "PowerBook6,1" => "PowerBook G4 (12-inch)",
            "PowerBook5,9" => "PowerBook G4 (17-inch Double-Layer SD)",
            "PowerBook5,8" => "PowerBook G4 (15-inch Double-Layer SD)",
            "PowerBook5,7" => "PowerBook G4 (17-inch 1.67GHz)",
            "PowerBook5,6" => "PowerBook G4 (15-inch 1.66/1.5GHz)",
            "PowerBook5,5" => "PowerBook G4 (17-inch 1.5GHz)",
            "PowerBook5,4" => "PowerBook G4 (15-inch 1.5/1.33GHz)",
            "PowerBook5,3" => "PowerBook G4 (17-inch 1.33GHz)",
            "PowerBook5,2" => "PowerBook G4 (15-inch FW 800)",
            "PowerBook5,1" => "PowerBook G4 (17-inch)",
            "PowerBook4,3" => "iBook (14.1 LCD 16 VRAM | Opaque 16 VRAM | 32 VRAM)",
            "PowerBook4,2" => "iBook (14.1 LCD)",
            "PowerBook4,1" => "iBook (Dual USB | late 2001)",
            "PowerBook3,5" => "PowerBook G4 (1GHz/867MHz)",
            "PowerBook3,4" => "PowerBook G4 (DVI)",
            "PowerBook3,3" => "PowerBook G4 (Gigabit Ethernet)",
            "PowerBook3,2" => "PowerBook G4 (Titanium)",
            "PowerBook3,1" => "PowerBook (Firewire)",
            "PowerBook2,2" => "iBook (FireWire)",
            "PowerBook2,1" => "iBook",
            "PowerBook1,1" => "PowerBook G3 (Bronze Keyboard)",
            "RackMac3,1" => "XServe G5",
            "RackMac1,2" => "XServe (Slot Load | Cluster Node)",
            "RackMac1,1" => "XServe",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_label() {
        assert_eq!(CpuDisplay::raw_label("Test"), "          Test: ");
    }

    #[test]
    fn test_raw_sublabel() {
        assert_eq!(CpuDisplay::raw_sublabel("Sub"), "                Sub: ");
    }

    #[test]
    fn test_raw_inline_sublabel() {
        assert_eq!(
            CpuDisplay::raw_inline_sublabel("Cache", "L1d"),
            "         Cache: L1d: "
        );
    }

    #[test]
    fn test_label_no_color() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(disp.label("Model"), "         Model: ");
    }

    #[test]
    fn test_sublabel_no_color() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(disp.sublabel("L1d"), "                L1d: ");
    }

    #[test]
    fn test_inline_sublabel_no_color() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(
            disp.inline_sublabel("Cache", "L1d"),
            "         Cache: L1d: "
        );
    }

    #[test]
    fn test_format_frequency_mhz() {
        assert_eq!(CpuDisplay::format_frequency(800u32), "800.00 MHz");
    }

    #[test]
    fn test_format_frequency_ghz_exact() {
        assert_eq!(CpuDisplay::format_frequency(3000u32), "3.00 GHz");
    }

    #[test]
    fn test_format_frequency_ghz_fraction() {
        assert_eq!(CpuDisplay::format_frequency(3500u32), "3.50 GHz");
    }

    #[test]
    fn test_format_frequency_ghz_precise() {
        assert_eq!(CpuDisplay::format_frequency(2496u32), "2.49 GHz");
    }

    #[test]
    fn test_format_frequency_zero() {
        assert_eq!(CpuDisplay::format_frequency(0u32), "0.00 MHz");
    }

    #[test]
    fn test_format_frequency_u64() {
        assert_eq!(CpuDisplay::format_frequency(2400u64), "2.40 GHz");
    }

    #[test]
    fn test_label_with_color() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: true,
                compact: false,
                verbose: false,
            },
        };
        let lbl = disp.label("Vendor");
        assert!(lbl.starts_with("\x1b["));
        assert!(lbl.contains("Vendor"));
    }

    #[test]
    fn test_ansi_format() {
        let s = CpuDisplay::ansi("31");
        assert_eq!(s, "\x1b[31m");
    }

    #[test]
    fn test_ansi_color() {
        let s = CpuDisplay::ansi_color("32", "green");
        assert_eq!(s, "\x1b[32mgreen\x1b[m");
    }

    #[test]
    fn test_cache_count_zero_share() {
        assert_eq!(CpuDisplay::cache_count(0, 4), "");
    }

    #[test]
    fn test_cache_count_single_core() {
        assert_eq!(CpuDisplay::cache_count(4, 1), "");
    }

    #[test]
    fn test_cache_count_multi() {
        assert_eq!(CpuDisplay::cache_count(4, 8), "2x ");
    }

    #[test]
    fn test_cache_count_exact() {
        assert_eq!(CpuDisplay::cache_count(2, 2), "");
    }

    #[test]
    fn test_cache_size_kb() {
        let (num, unit) = CpuDisplay::cache_size(65536);
        assert_eq!(num, 64);
        assert_eq!(unit, "KB");
    }

    #[test]
    fn test_cache_size_mb() {
        let (num, unit) = CpuDisplay::cache_size(8 * 1024 * 1024);
        assert_eq!(num, 8);
        assert_eq!(unit, "MB");
    }

    #[test]
    fn test_cache_size_zero() {
        let (num, unit) = CpuDisplay::cache_size(0);
        assert_eq!(num, 0);
        assert_eq!(unit, "KB");
    }

    #[test]
    fn test_format_system_name_macbook_air_m1() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(
            disp.format_system_name("MacBookAir10,1"),
            "MacBook Air (M1, 2020)"
        );
    }

    #[test]
    fn test_format_system_name_unknown() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(disp.format_system_name("CustomPC"), "CustomPC");
    }

    #[test]
    fn test_format_system_name_mac_pro() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(disp.format_system_name("MacPro7,1"), "Mac Pro (2019)");
    }

    #[test]
    fn test_format_system_name_powerpc() {
        let disp = CpuDisplay {
            flags: CliFlags {
                color: false,
                compact: false,
                verbose: false,
            },
        };
        assert_eq!(
            disp.format_system_name("PowerMac11,2"),
            "Power Mac G5 (Late 2005)"
        );
    }
}
