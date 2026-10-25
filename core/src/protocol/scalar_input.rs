use std::fmt;

use serde::Deserializer;
use serde::de::{self, Visitor};

pub(crate) fn deserialize_bounded<'de, D>(
    deserializer: D,
    maximum: u64,
    expectation: &'static str,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedVisitor {
        maximum: u64,
        expectation: &'static str,
    }
    impl Visitor<'_> for BoundedVisitor {
        type Value = u64;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.expectation)
        }
        fn visit_u64<E: de::Error>(self, value: u64) -> Result<u64, E> {
            self.validate(value)
        }
        fn visit_i64<E: de::Error>(self, value: i64) -> Result<u64, E> {
            u64::try_from(value)
                .map_err(|_| E::custom(self.expectation))
                .and_then(|value| self.validate(value))
        }
        fn visit_f64<E: de::Error>(self, value: f64) -> Result<u64, E> {
            if value.is_finite()
                && value.fract() == 0.0
                && value >= 0.0
                && value <= self.maximum as f64
            {
                Ok(value as u64)
            } else {
                Err(E::custom(self.expectation))
            }
        }
    }
    impl BoundedVisitor {
        fn validate<E: de::Error>(self, value: u64) -> Result<u64, E> {
            if value <= self.maximum {
                Ok(value)
            } else {
                Err(E::custom(self.expectation))
            }
        }
    }
    deserializer.deserialize_any(BoundedVisitor {
        maximum,
        expectation,
    })
}

#[cfg(test)]
mod tests {
    use serde::de::DeserializeOwned;

    use crate::protocol::v6::RoundCount;
    use crate::protocol::v7::{EffectView, ItemMask, Wins};

    fn accepts<T: DeserializeOwned>(inputs: &[&str]) {
        for input in inputs {
            assert!(
                serde_json::from_str::<T>(input).is_ok(),
                "expected {input} to be accepted"
            );
        }
    }

    fn rejects<T: DeserializeOwned>(inputs: &[&str], message: &str) {
        for input in inputs {
            let error = serde_json::from_str::<T>(input)
                .err()
                .unwrap_or_else(|| panic!("expected {input} to be rejected"));
            assert!(error.to_string().contains(message), "{input}: {error}");
        }
    }

    #[test]
    fn shared_number_input_preserves_real_consumer_boundaries() {
        accepts::<Wins>(&["0", "9", "9.0", "9e0"]);
        rejects::<Wins>(
            &["-1", "1.5", "10", "1e100", "null", r#""9""#],
            "invalid wins",
        );

        accepts::<RoundCount>(&["0", "2147483647", "2147483647.0", "1e3"]);
        rejects::<RoundCount>(
            &["-1", "0.5", "2147483648", "1e100", "false", "[]"],
            "invalid round count",
        );

        accepts::<ItemMask>(&["0", "4194303", "4194303.0", "1e3"]);
        rejects::<ItemMask>(
            &["-1", "0.5", "4194304", "1e100", "null", "{}"],
            "item mask contains unknown bits",
        );

        for ticks in ["1", "600", "600.0", "6e2"] {
            let input = format!(r#"{{"kind":"slowFoot","remainingTicks":{ticks}}}"#);
            assert!(
                serde_json::from_str::<EffectView>(&input).is_ok(),
                "{input}"
            );
        }
        for ticks in ["-1", "0.5", "601", "1e100", "null", r#""1""#] {
            let input = format!(r#"{{"kind":"slowFoot","remainingTicks":{ticks}}}"#);
            let error = serde_json::from_str::<EffectView>(&input).unwrap_err();
            assert!(error.to_string().contains("invalid effect duration"));
        }
    }
}
