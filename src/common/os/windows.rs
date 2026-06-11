use crate::common::DataSource;

pub fn get_socket_count() -> (u32, DataSource) {
    // @TODO: Get cpu socket count from windows
    (1, DataSource::DefaultValue)
}
