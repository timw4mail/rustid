use crate::common::{DataSource, OS, TOSData};

impl TOSData for OS {
    fn get_socket_count() -> (u32, DataSource) {
        // @TODO: Get cpu socket count from windows
        (1, DataSource::DefaultValue)
    }
}
