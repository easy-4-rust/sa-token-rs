//! Custom Base64 serializer implementations.

macro_rules! impl_custom_serializer {
    ($type:ty) => {
        impl sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk for $type {
            fn bytes_to_string(&self, bytes: &[u8]) -> sa_token_core::exception::SaResult<String> {
                self.inner.bytes_to_string(bytes)
            }

            fn string_to_bytes(&self, value: &str) -> sa_token_core::exception::SaResult<Vec<u8>> {
                self.inner.string_to_bytes(value)
            }
        }

        impl sa_token_core::serializer::SaSerializerTemplate for $type {
            fn object_to_string(
                &self,
                object: Option<&serde_json::Value>,
            ) -> sa_token_core::exception::SaResult<Option<String>> {
                use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
                self.native_object_to_string(object)
            }

            fn string_to_object(
                &self,
                value: Option<&str>,
            ) -> sa_token_core::exception::SaResult<Option<serde_json::Value>> {
                use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
                self.native_string_to_object(value)
            }

            fn object_to_bytes(
                &self,
                object: Option<&serde_json::Value>,
            ) -> sa_token_core::exception::SaResult<Option<Vec<u8>>> {
                use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
                self.native_object_to_bytes(object)
            }

            fn bytes_to_object(
                &self,
                bytes: Option<&[u8]>,
            ) -> sa_token_core::exception::SaResult<Option<serde_json::Value>> {
                use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
                self.native_bytes_to_object(bytes)
            }
        }
    };
}

pub mod sa_serializer_for_base64_use_custom_characters;
pub mod sa_serializer_for_base64_use_emoji;
pub mod sa_serializer_for_base64_use_periodic_table;
pub mod sa_serializer_for_base64_use_special_symbols;
pub mod sa_serializer_for_base64_use_tian_gan;

pub use sa_serializer_for_base64_use_custom_characters::SaSerializerForBase64UseCustomCharacters;
pub use sa_serializer_for_base64_use_emoji::SaSerializerForBase64UseEmoji;
pub use sa_serializer_for_base64_use_periodic_table::SaSerializerForBase64UsePeriodicTable;
pub use sa_serializer_for_base64_use_special_symbols::SaSerializerForBase64UseSpecialSymbols;
pub use sa_serializer_for_base64_use_tian_gan::SaSerializerForBase64UseTianGan;
