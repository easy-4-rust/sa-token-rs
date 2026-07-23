//! Base64 codec using Chinese heavenly stems and related characters.

use super::SaSerializerForBase64UseCustomCharacters;

const ALPHABET: &str = concat!(
    "甲乙丙丁戊己庚辛",
    "壬癸子丑寅卯辰巳",
    "午未申酉戌亥乾坤",
    "震巽坎离艮兑金木",
    "水火土天地日月山",
    "石田风雷电霜雾露",
    "东南西北中信谷岚",
    "宇宙羽泰铭安鹤纤"
);

/// Heavenly-stems custom Base64 serializer.
#[derive(Debug, Clone)]
pub struct SaSerializerForBase64UseTianGan {
    pub(super) inner: SaSerializerForBase64UseCustomCharacters,
}

impl Default for SaSerializerForBase64UseTianGan {
    fn default() -> Self {
        Self {
            inner: SaSerializerForBase64UseCustomCharacters::from_static(ALPHABET, '口'),
        }
    }
}

impl_custom_serializer!(SaSerializerForBase64UseTianGan);
