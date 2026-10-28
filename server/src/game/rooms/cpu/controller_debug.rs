use std::fmt;

use super::CpuController;

impl fmt::Debug for CpuController {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CpuController")
            .field("room_instance", &self.room_instance)
            .field("private_seed", &"<redacted>")
            .field("world", &self.world)
            .field("request_serial", &self.request_serial)
            .field("pending", &self.pending.is_some())
            .field("escaping", &self.escaping)
            .finish()
    }
}
