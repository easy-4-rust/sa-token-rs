//! Base64 codec using the first 64 Chinese periodic-table element names.

use super::SaSerializerForBase64UseCustomCharacters;

const ALPHABET: &str = concat!(
    "氢氦锂铍硼碳氮氧",
    "氟氖钠镁铝硅磷硫",
    "氯氩钾钙钪钛钒铬",
    "锰铁钴镍铜锌镓锗",
    "砷硒溴氪铷锶钇锆",
    "铌钼锝钌铑钯银镉",
    "铟锡锑碲碘氙铯钡",
    "镧铈镨钕钷钐铕钆"
);

/// Periodic-table custom Base64 serializer.
#[derive(Debug, Clone)]
pub struct SaSerializerForBase64UsePeriodicTable {
    pub(super) inner: SaSerializerForBase64UseCustomCharacters,
}

impl Default for SaSerializerForBase64UsePeriodicTable {
    fn default() -> Self {
        Self {
            inner: SaSerializerForBase64UseCustomCharacters::from_static(ALPHABET, '鿫'),
        }
    }
}

impl_custom_serializer!(SaSerializerForBase64UsePeriodicTable);
