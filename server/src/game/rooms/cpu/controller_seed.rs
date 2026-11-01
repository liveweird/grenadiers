pub(super) fn derive_round_seed(mut seed: [u8; 32], physical_round: u64) -> [u8; 32] {
    let round = physical_round.to_le_bytes();
    for (index, value) in seed.iter_mut().enumerate() {
        *value ^= round[index % round.len()].wrapping_add((index as u8).wrapping_mul(0x5d));
        *value = value.rotate_left((index % 7 + 1) as u32);
    }
    seed
}
