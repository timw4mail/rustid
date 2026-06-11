use crate::common::{DataSource, OS, TOSData, TopologyTier};

impl TOSData for OS {
    fn get_socket_count() -> TopologyTier {
        // @TODO: Get cpu socket count from windows
        TopologyTier::new(1, DataSource::DefaultValue)
    }
}
