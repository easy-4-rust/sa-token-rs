//! Unpadded Base64 codec using emoji U+1F600 through U+1F63F.

use std::collections::HashMap;

use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::serializer::SaSerializerTemplate;
use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
use serde_json::Value;

/// Emoji custom Base64 serializer matching the Java bit-stream algorithm.
#[derive(Debug, Clone)]
pub struct SaSerializerForBase64UseEmoji {
    alphabet: Vec<char>,
    reverse: HashMap<char, u8>,
}

impl Default for SaSerializerForBase64UseEmoji {
    fn default() -> Self {
        let alphabet: Vec<char> = (0x1f600..=0x1f63f).filter_map(char::from_u32).collect();
        let reverse = alphabet
            .iter()
            .copied()
            .enumerate()
            .map(|(index, character)| (character, index as u8))
            .collect();
        Self { alphabet, reverse }
    }
}

impl SaSerializerTemplateForJdk for SaSerializerForBase64UseEmoji {
    fn bytes_to_string(&self, bytes: &[u8]) -> SaResult<String> {
        let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
        let mut accumulator = 0_u32;
        let mut bit_count = 0_u8;
        for byte in bytes {
            accumulator = (accumulator << 8) | u32::from(*byte);
            bit_count += 8;
            while bit_count >= 6 {
                bit_count -= 6;
                result.push(self.alphabet[((accumulator >> bit_count) & 0x3f) as usize]);
            }
        }
        if bit_count > 0 {
            result.push(self.alphabet[((accumulator << (6 - bit_count)) & 0x3f) as usize]);
        }
        Ok(result)
    }

    fn string_to_bytes(&self, value: &str) -> SaResult<Vec<u8>> {
        let mut result = Vec::with_capacity(value.chars().count() * 6 / 8);
        let mut accumulator = 0_u32;
        let mut bit_count = 0_u8;
        for character in value.chars() {
            let index =
                self.reverse
                    .get(&character)
                    .copied()
                    .ok_or_else(|| SaTokenException::Other {
                        message: format!("invalid emoji in custom Base64 input: {character}"),
                    })?;
            accumulator = (accumulator << 6) | u32::from(index);
            bit_count += 6;
            if bit_count >= 8 {
                bit_count -= 8;
                result.push(((accumulator >> bit_count) & 0xff) as u8);
            }
        }
        Ok(result)
    }
}

impl SaSerializerTemplate for SaSerializerForBase64UseEmoji {
    fn object_to_string(&self, object: Option<&Value>) -> SaResult<Option<String>> {
        self.native_object_to_string(object)
    }

    fn string_to_object(&self, value: Option<&str>) -> SaResult<Option<Value>> {
        self.native_string_to_object(value)
    }

    fn object_to_bytes(&self, object: Option<&Value>) -> SaResult<Option<Vec<u8>>> {
        self.native_object_to_bytes(object)
    }

    fn bytes_to_object(&self, bytes: Option<&[u8]>) -> SaResult<Option<Value>> {
        self.native_bytes_to_object(bytes)
    }
}
