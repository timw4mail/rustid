//! Classic Macintosh OS-specific hardware and topology detection.

#[cfg(classic_macos)]
use alloc::string::ToString;
#[cfg(classic_macos)]
use crate::common::{DataSource, OS, SystemInfo, TDetect, TOSData, TopologyCount, TopologyTier};
#[cfg(classic_macos)]
use crate::m68k::gestalt;

#[cfg(classic_macos)]
impl TDetect for TopologyCount {
    fn detect() -> Self {
        let sockets = OS::get_socket_count();
        TopologyCount {
            sockets,
            cores: 1,
            threads: 1,
            source: DataSource::Gestalt("gestaltMachineType / gestaltProcClkSpeed"),
        }
    }
}

#[cfg(classic_macos)]
impl TOSData for OS {
    fn get_system_name() -> Option<SystemInfo> {
        let mut mach_id: i32 = 0;
        unsafe {
            gestalt_raw(gestalt::GESTALT_MACH_TYPE, &mut mach_id);
        }
        let model_name = gestalt::get_mac_model_name(mach_id as u32);
        Some(SystemInfo::new(
            Some("Apple Computer, Inc.".to_string()),
            DataSource::DefaultValue,
            Some(model_name.to_string()),
            DataSource::Gestalt("gestaltMachineType"),
        ))
    }

    fn get_socket_count() -> TopologyTier {
        // 'mpc ' selector checks for Multiprocessing API / dual CPU cards
        let mut mp_count: i32 = 0;
        let res = unsafe { gestalt_raw(0x6d70_6320, &mut mp_count) };
        if res == 0 && mp_count > 1 {
            return TopologyTier::new(mp_count as u32, DataSource::Gestalt("gestaltMultiprocessorCount"));
        }
        TopologyTier::new(1, DataSource::Gestalt("gestaltMachineType"))
    }
}

#[cfg(classic_macos)]
unsafe extern "C" {
    fn gestalt_raw(selector: u32, response: *mut i32) -> i16;
}
