//! Base64 codec using 64 special symbols.

use super::SaSerializerForBase64UseCustomCharacters;

const ALPHABET: &str = concat!(
    "▲▼●◆■★▶◀",
    "♠♥♦♣▁▂▃▄",
    "▅▆▇█▏▎▍▌",
    "▋▊▉▬〓◤◥◣",
    "◢♩♪♫♬§〼↖",
    "↑↗←→↙↓↘☴",
    "☲☷☳☱☶☵☰◐",
    "◑☀☼▪•‥…∷"
);

/// Special-symbol custom Base64 serializer.
#[derive(Debug, Clone)]
pub struct SaSerializerForBase64UseSpecialSymbols {
    pub(super) inner: SaSerializerForBase64UseCustomCharacters,
}

impl Default for SaSerializerForBase64UseSpecialSymbols {
    fn default() -> Self {
        Self {
            inner: SaSerializerForBase64UseCustomCharacters::from_static(ALPHABET, '※'),
        }
    }
}

impl_custom_serializer!(SaSerializerForBase64UseSpecialSymbols);
