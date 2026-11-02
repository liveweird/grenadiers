use super::*;

#[test]
fn bounds_constants_and_defaults_are_explicit() {
    assert_eq!(
        BombCapacityDropMultiplier::new(1),
        Ok(BombCapacityDropMultiplier::LEGACY)
    );
    assert_eq!(
        BombCapacityDropMultiplier::new(5),
        Ok(BombCapacityDropMultiplier::DEFAULT)
    );
    assert_eq!(BombCapacityDropMultiplier::new(10).unwrap().value(), 10);
    assert_eq!(
        BombCapacityDropMultiplier::default(),
        BombCapacityDropMultiplier::DEFAULT
    );
}

#[test]
fn values_outside_the_closed_integer_domain_are_rejected() {
    for value in [0, 11, u64::MAX] {
        assert_eq!(
            BombCapacityDropMultiplier::new(value),
            Err(BombCapacityDropMultiplierError::InvalidValue(value))
        );
    }
}
