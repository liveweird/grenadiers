use super::random_words::RandomWords;
use crate::game::{EnabledItems, PickupKind};

const SAMPLE_LIMIT: u32 = 4_294_967_200;
const MAX_DRAWS_PER_SAMPLE: usize = 16;

pub(super) fn sample_pickup(
    source: &mut impl RandomWords,
    enabled: EnabledItems,
) -> Option<PickupKind> {
    let heart = u32::from(enabled.contains_heart());
    let bound = 100 + heart * 25 + (enabled.bits().count_ones() - heart) * 15;
    let value = if bound == 100 {
        sample_percent(source)?
    } else {
        sample_below(source, bound)?
    };
    let heart_end = 100 + u32::from(enabled.contains_heart()) * 25;
    let shield_end = heart_end + u32::from(enabled.contains_shield()) * 15;
    let golden_end = shield_end + u32::from(enabled.contains_golden_flame()) * 15;
    let short_fuse_end = golden_end + u32::from(enabled.contains_short_fuse()) * 15;
    let short_flame_end = short_fuse_end + u32::from(enabled.contains_short_flame()) * 15;
    let long_fuse_end = short_flame_end + u32::from(enabled.contains_long_fuse()) * 15;
    let reverse_controls_end = long_fuse_end + u32::from(enabled.contains_reverse_controls()) * 15;
    let diarrhea_end = reverse_controls_end + u32::from(enabled.contains_diarrhea()) * 15;
    let slow_foot_end = diarrhea_end + u32::from(enabled.contains_slow_foot()) * 15;
    let piercing_end = slow_foot_end + u32::from(enabled.contains_piercing_bomb()) * 15;
    let power_end = piercing_end + u32::from(enabled.contains_power_bomb()) * 15;
    let remote_end = power_end + u32::from(enabled.contains_remote_bomb()) * 15;
    let landmine_end = remote_end + u32::from(enabled.contains_landmine()) * 15;
    let cloak_end = landmine_end + u32::from(enabled.contains_invisibility_cloak()) * 15;
    let stopwatch_end = cloak_end + u32::from(enabled.contains_stopwatch()) * 15;
    let invisible_bombs_end = stopwatch_end + u32::from(enabled.contains_invisible_bombs()) * 15;
    let teleporter_end = invisible_bombs_end + u32::from(enabled.contains_teleporter()) * 15;
    let wall_pass_end = teleporter_end + u32::from(enabled.contains_wall_pass()) * 15;
    let kick_end = wall_pass_end + u32::from(enabled.contains_kick()) * 15;
    let punch_end = kick_end + u32::from(enabled.contains_punch()) * 15;
    let bracelet_end = punch_end + u32::from(enabled.contains_bracelet()) * 15;
    let oil_end = bracelet_end + u32::from(enabled.contains_oil_barrel()) * 15;
    Some(match value {
        0..40 => PickupKind::BombCapacity,
        40..80 => PickupKind::BlastRange,
        80..100 => PickupKind::MovementSpeed,
        value if value < heart_end => PickupKind::Heart,
        value if value < shield_end => PickupKind::Shield,
        value if value < golden_end => PickupKind::GoldenFlame,
        value if value < short_fuse_end => PickupKind::ShortFuse,
        value if value < short_flame_end => PickupKind::ShortFlame,
        value if value < long_fuse_end => PickupKind::LongFuse,
        value if value < reverse_controls_end => PickupKind::ReverseControls,
        value if value < diarrhea_end => PickupKind::Diarrhea,
        value if value < slow_foot_end => PickupKind::SlowFoot,
        value if value < piercing_end => PickupKind::PiercingBomb,
        value if value < power_end => PickupKind::PowerBomb,
        value if value < remote_end => PickupKind::RemoteBomb,
        value if value < landmine_end => PickupKind::Landmine,
        value if value < cloak_end => PickupKind::InvisibilityCloak,
        value if value < stopwatch_end => PickupKind::Stopwatch,
        value if value < invisible_bombs_end => PickupKind::InvisibleBombs,
        value if value < teleporter_end => PickupKind::Teleporter,
        value if value < wall_pass_end => PickupKind::WallPass,
        value if value < kick_end => PickupKind::Kick,
        value if value < punch_end => PickupKind::Punch,
        value if value < bracelet_end => PickupKind::Bracelet,
        value if value < oil_end => PickupKind::OilBarrel,
        _ => unreachable!("sample value is bounded by the enabled item pool"),
    })
}

fn sample_below(source: &mut impl RandomWords, bound: u32) -> Option<u32> {
    let limit = u32::MAX - u32::MAX % bound;
    for _ in 0..MAX_DRAWS_PER_SAMPLE {
        let value = source.next_u32()?;
        if value < limit {
            return Some(value % bound);
        }
    }
    None
}

pub(super) fn sample_percent(source: &mut impl RandomWords) -> Option<u32> {
    for _ in 0..MAX_DRAWS_PER_SAMPLE {
        let value = source.next_u32()?;
        if value < SAMPLE_LIMIT {
            return Some(value % 100);
        }
    }
    None
}
