use std::{fmt, fs::File, io::Read, path::Path, sync::Arc};

const SEED_BYTES: usize = 32;
const HEX_SEED_BYTES: usize = SEED_BYTES * 2;
const MAX_FILE_BYTES: u64 = (HEX_SEED_BYTES + 2) as u64;

type EntropyFill = dyn Fn(&mut [u8]) -> Result<(), ()> + Send + Sync;

#[derive(Clone)]
pub(crate) struct RoundSeedSource {
    source: Source,
}

#[derive(Clone)]
enum Source {
    OsEntropy(Arc<EntropyFill>),
    Fixed([u8; SEED_BYTES]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RoundSeedError {
    FileRead,
    InvalidFile,
    EntropyUnavailable,
}

impl RoundSeedSource {
    pub(crate) fn from_optional_path(path: Option<&Path>) -> Result<Self, RoundSeedError> {
        match path {
            Some(path) => Self::from_file(path),
            None => Ok(Self::with_entropy(|bytes| {
                getrandom::fill(bytes).map_err(|_| ())
            })),
        }
    }

    pub(crate) fn with_entropy(
        entropy_fill: impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            source: Source::OsEntropy(Arc::new(entropy_fill)),
        }
    }

    pub(crate) fn next_seed(&self) -> Result<[u8; SEED_BYTES], RoundSeedError> {
        match &self.source {
            Source::OsEntropy(entropy_fill) => {
                let mut seed = [0; SEED_BYTES];
                entropy_fill(&mut seed).map_err(|()| RoundSeedError::EntropyUnavailable)?;
                Ok(seed)
            }
            Source::Fixed(seed) => Ok(*seed),
        }
    }

    fn from_file(path: &Path) -> Result<Self, RoundSeedError> {
        let mut bytes = Vec::with_capacity(HEX_SEED_BYTES + 1);
        File::open(path)
            .map_err(|_| RoundSeedError::FileRead)?
            .take(MAX_FILE_BYTES)
            .read_to_end(&mut bytes)
            .map_err(|_| RoundSeedError::FileRead)?;
        let seed = decode_seed_file(&bytes)?;
        Ok(Self {
            source: Source::Fixed(seed),
        })
    }
}

impl fmt::Debug for RoundSeedSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = match self.source {
            Source::OsEntropy(_) => "OsEntropy",
            Source::Fixed(_) => "Fixed(<redacted>)",
        };
        formatter
            .debug_struct("RoundSeedSource")
            .field("source", &mode)
            .finish()
    }
}

impl fmt::Display for RoundSeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::FileRead => "round seed file could not be read",
            Self::InvalidFile => "round seed file has an invalid format",
            Self::EntropyUnavailable => "round seed entropy is unavailable",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RoundSeedError {}

fn decode_seed_file(bytes: &[u8]) -> Result<[u8; SEED_BYTES], RoundSeedError> {
    let hexadecimal = match bytes {
        bytes if bytes.len() == HEX_SEED_BYTES => bytes,
        bytes if bytes.len() == HEX_SEED_BYTES + 1 && bytes.last() == Some(&b'\n') => {
            &bytes[..HEX_SEED_BYTES]
        }
        _ => return Err(RoundSeedError::InvalidFile),
    };

    let mut seed = [0; SEED_BYTES];
    for (index, output) in seed.iter_mut().enumerate() {
        let offset = index * 2;
        *output = (hex_value(hexadecimal[offset])? << 4) | hex_value(hexadecimal[offset + 1])?;
    }
    Ok(seed)
}

fn hex_value(byte: u8) -> Result<u8, RoundSeedError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(RoundSeedError::InvalidFile),
    }
}
