use grenadiers_core::{
    protocol::{RoomCode, RoomId},
    room::RoomMode,
};

use super::{CODE_ALPHABET, COLLISION_RETRIES, GameRooms, Registry, RoomError, RoomIdentity};

impl GameRooms {
    pub(super) fn generate_identity(
        &self,
        registry: &mut Registry,
        mode: RoomMode,
    ) -> Result<RoomIdentity, RoomError> {
        let prefix = match registry.prefix {
            Some(prefix) => prefix,
            None => {
                let mut bytes = [0; 8];
                (self.random_fill)(&mut bytes).map_err(|_| RoomError::CapacityReached)?;
                let prefix = u64::from_be_bytes(bytes);
                registry.prefix = Some(prefix);
                prefix
            }
        };
        let serial = registry.next_serial;
        let next_serial = serial.checked_add(1).ok_or(RoomError::CapacityReached)?;
        for _ in 0..COLLISION_RETRIES {
            let code = self.generate_code()?;
            if !registry.codes.contains_key(&code) {
                let id = RoomId::new(&format!("{prefix:016x}{serial:016x}"))
                    .map_err(|_| RoomError::CapacityReached)?;
                let private_cpu_seed = if matches!(mode, RoomMode::Solo(_)) {
                    let mut seed = [0; 32];
                    (self.random_fill)(&mut seed).map_err(|_| RoomError::CapacityReached)?;
                    Some(seed)
                } else {
                    None
                };
                registry.next_serial = next_serial;
                return Ok(RoomIdentity {
                    id,
                    code,
                    profile: self.profile,
                    mode,
                    registry_serial: serial,
                    private_cpu_seed,
                });
            }
        }
        Err(RoomError::CapacityReached)
    }

    fn generate_code(&self) -> Result<RoomCode, RoomError> {
        let mut bytes = [0; 5];
        (self.random_fill)(&mut bytes).map_err(|_| RoomError::CapacityReached)?;
        let bits = u64::from_be_bytes([0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]);
        let code = (0..8)
            .map(|shift| CODE_ALPHABET[((bits >> (35 - shift * 5)) & 31) as usize] as char)
            .collect::<String>();
        RoomCode::new(&code).map_err(|_| RoomError::CapacityReached)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Instant;

    use grenadiers_core::room::{CpuProfile, RoomMode};

    use super::*;

    #[test]
    fn failed_private_cpu_entropy_does_not_consume_registry_serial() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let rooms = GameRooms::with_random_fill(Instant::now(), move |bytes| {
            let call = observed.fetch_add(1, Ordering::Relaxed);
            if call == 2 {
                return Err(());
            }
            bytes.fill(7);
            Ok(())
        });
        let mut registry = rooms.lock();
        assert!(matches!(
            rooms.generate_identity(&mut registry, RoomMode::Solo(CpuProfile::INITIAL)),
            Err(RoomError::CapacityReached)
        ));
        assert_eq!(registry.next_serial, 1);
        assert!(registry.rooms.is_empty());
    }
}
