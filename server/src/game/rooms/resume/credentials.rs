use std::fmt;

use grenadiers_core::protocol::v8::ResumeToken;

const CREDENTIAL_BYTES: usize = 32;
const MAX_COLLISION_ATTEMPTS: usize = 3;

#[derive(Clone, PartialEq, Eq)]
pub(in crate::game::rooms) struct ResumeCredential {
    token: ResumeToken,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ResumeCredentialError {
    EntropyUnavailable,
    CollisionExhausted,
}

impl ResumeCredential {
    pub(in crate::game::rooms) fn mint(
        entropy_fill: &(impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + ?Sized),
        other_seat: Option<&Self>,
    ) -> Result<Self, ResumeCredentialError> {
        for _ in 0..MAX_COLLISION_ATTEMPTS {
            let mut bytes = [0_u8; CREDENTIAL_BYTES];
            entropy_fill(&mut bytes).map_err(|()| ResumeCredentialError::EntropyUnavailable)?;
            let candidate = Self {
                token: encode(bytes)?,
            };
            if other_seat.is_none_or(|other| candidate != *other) {
                return Ok(candidate);
            }
        }
        Err(ResumeCredentialError::CollisionExhausted)
    }

    pub(in crate::game::rooms) fn token(&self) -> ResumeToken {
        self.token.clone()
    }

    pub(in crate::game::rooms) fn matches(&self, supplied: &ResumeToken) -> bool {
        self.token
            .as_str()
            .bytes()
            .zip(supplied.as_str().bytes())
            .fold(0_u8, |difference, (expected, actual)| {
                difference | (expected ^ actual)
            })
            == 0
    }
}

impl fmt::Debug for ResumeCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResumeCredential([REDACTED])")
    }
}

fn encode(bytes: [u8; CREDENTIAL_BYTES]) -> Result<ResumeToken, ResumeCredentialError> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(CREDENTIAL_BYTES * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    ResumeToken::new(encoded).map_err(|_| ResumeCredentialError::EntropyUnavailable)
}
