macro_rules! with_item_spec {
    ($callback:ident) => {
        $callback! {
            (Kick, 0, KICK, contains_kick),
            (Punch, 1, PUNCH, contains_punch),
            (Bracelet, 2, BRACELET, contains_bracelet),
            (WallPass, 3, WALL_PASS, contains_wall_pass),
            (Heart, 4, HEART, contains_heart),
            (RemoteBomb, 5, REMOTE_BOMB, contains_remote_bomb),
            (PiercingBomb, 6, PIERCING_BOMB, contains_piercing_bomb),
            (PowerBomb, 7, POWER_BOMB, contains_power_bomb),
            (Landmine, 8, LANDMINE, contains_landmine),
            (Shield, 9, SHIELD, contains_shield),
            (Stopwatch, 10, STOPWATCH, contains_stopwatch),
            (InvisibilityCloak, 11, INVISIBILITY_CLOAK, contains_invisibility_cloak),
            (GoldenFlame, 12, GOLDEN_FLAME, contains_golden_flame),
            (ShortFuse, 13, SHORT_FUSE, contains_short_fuse),
            (Diarrhea, 14, DIARRHEA, contains_diarrhea),
            (SlowFoot, 15, SLOW_FOOT, contains_slow_foot),
            (ShortFlame, 16, SHORT_FLAME, contains_short_flame),
            (LongFuse, 17, LONG_FUSE, contains_long_fuse),
            (ReverseControls, 18, REVERSE_CONTROLS, contains_reverse_controls),
            (InvisibleBombs, 19, INVISIBLE_BOMBS, contains_invisible_bombs),
            (OilBarrel, 20, OIL_BARREL, contains_oil_barrel),
            (Teleporter, 21, TELEPORTER, contains_teleporter),
        }
    };
}

pub(crate) use with_item_spec;
