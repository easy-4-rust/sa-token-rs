//! Base64 codec with a caller-provided 64-character alphabet.

use std::collections::{HashMap, HashSet};

use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::serializer::SaSerializerTemplate;
use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
use serde_json::Value;

/// Base64 serializer using a validated custom Unicode alphabet.
#[derive(Debug, Clone)]
pub struct SaSerializerForBase64UseCustomCharacters {
    alphabet: Vec<char>,
    reverse: HashMap<char, u8>,
    pad: char,
}

impl SaSerializerForBase64UseCustomCharacters {
    /// Creates a codec from exactly 64 distinct Unicode scalar values.
    ///
    /// # Errors
    /// Returns an error when the alphabet length is not 64, contains duplicate
    /// characters, or contains the padding character.
    pub fn new(custom_characters: &str, pad: char) -> SaResult<Self> {
        let alphabet: Vec<char> = custom_characters.chars().collect();
        if alphabet.len() != 64 {
            return Err(invalid("custom Base64 alphabet must contain 64 characters"));
        }
        let unique: HashSet<char> = alphabet.iter().copied().collect();
        if unique.len() != 64 {
            return Err(invalid("custom Base64 alphabet contains duplicates"));
        }
        if unique.contains(&pad) {
            return Err(invalid("padding character must not occur in the alphabet"));
        }
        Ok(Self::from_parts(alphabet, pad))
    }

    pub(crate) fn from_static(custom_characters: &str, pad: char) -> Self {
        Self::from_parts(custom_characters.chars().collect(), pad)
    }

    fn from_parts(alphabet: Vec<char>, pad: char) -> Self {
        let reverse = alphabet
            .iter()
            .copied()
            .enumerate()
            .map(|(index, character)| (character, index as u8))
            .collect();
        Self {
            alphabet,
            reverse,
            pad,
        }
    }

    /// Returns the configured alphabet.
    pub fn custom_characters(&self) -> String {
        self.alphabet.iter().collect()
    }

    /// Returns the padding character.
    pub fn pad_character(&self) -> char {
        self.pad
    }
}

impl SaSerializerTemplateForJdk for SaSerializerForBase64UseCustomCharacters {
    fn bytes_to_string(&self, bytes: &[u8]) -> SaResult<String> {
        let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let first = u32::from(chunk[0]);
            let second = chunk.get(1).copied().map(u32::from).unwrap_or(0);
            let third = chunk.get(2).copied().map(u32::from).unwrap_or(0);
            let combined = (first << 16) | (second << 8) | third;
            encoded.push(self.alphabet[((combined >> 18) & 0x3f) as usize]);
            encoded.push(self.alphabet[((combined >> 12) & 0x3f) as usize]);
            encoded.push(if chunk.len() >= 2 {
                self.alphabet[((combined >> 6) & 0x3f) as usize]
            } else {
                self.pad
            });
            encoded.push(if chunk.len() == 3 {
                self.alphabet[(combined & 0x3f) as usize]
            } else {
                self.pad
            });
        }
        Ok(encoded)
    }

    fn string_to_bytes(&self, value: &str) -> SaResult<Vec<u8>> {
        let characters: Vec<char> = value.chars().collect();
        if characters.len() % 4 != 0 {
            return Err(invalid(
                "custom Base64 input length must be divisible by four",
            ));
        }
        let mut decoded = Vec::with_capacity(characters.len() / 4 * 3);
        let group_count = characters.len() / 4;
        for (group_index, group) in characters.chunks_exact(4).enumerate() {
            let final_group = group_index + 1 == group_count;
            let pad_count = group.iter().rev().take_while(|&&c| c == self.pad).count();
            if pad_count > 2
                || (!final_group && pad_count != 0)
                || group[..4 - pad_count].contains(&self.pad)
            {
                return Err(invalid("custom Base64 padding is invalid"));
            }
            let mut indices = [0_u32; 4];
            for (index, character) in group.iter().copied().enumerate() {
                if character != self.pad {
                    indices[index] = u32::from(*self.reverse.get(&character).ok_or_else(|| {
                        invalid("custom Base64 input contains an unknown character")
                    })?);
                }
            }
            let combined = (indices[0] << 18) | (indices[1] << 12) | (indices[2] << 6) | indices[3];
            decoded.push(((combined >> 16) & 0xff) as u8);
            if pad_count < 2 {
                decoded.push(((combined >> 8) & 0xff) as u8);
            }
            if pad_count == 0 {
                decoded.push((combined & 0xff) as u8);
            }
        }
        Ok(decoded)
    }
}

impl SaSerializerTemplate for SaSerializerForBase64UseCustomCharacters {
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

fn invalid(message: impl Into<String>) -> SaTokenException {
    SaTokenException::Other {
        message: message.into(),
    }
}
