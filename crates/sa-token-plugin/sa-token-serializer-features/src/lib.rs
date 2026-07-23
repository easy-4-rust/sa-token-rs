//! Custom Base64 alphabets migrated from Java `sa-token-serializer-features`.

pub mod plugin;
pub mod serializer;

pub use plugin::SaTokenPluginForSerializerFeatures;
pub use serializer::{
    SaSerializerForBase64UseCustomCharacters, SaSerializerForBase64UseEmoji,
    SaSerializerForBase64UsePeriodicTable, SaSerializerForBase64UseSpecialSymbols,
    SaSerializerForBase64UseTianGan,
};
