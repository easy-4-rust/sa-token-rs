//! Detailed error-code registry corresponding to Java `SaErrorCode`.

/// Namespace for stable Sa-Token detailed error codes.
pub struct SaErrorCode;

impl SaErrorCode {
    /// Java `CODE_UNDEFINED`.
    pub const CODE_UNDEFINED: i32 = -1;
    /// Java `CODE_10001`.
    pub const CODE_10001: i32 = 10001;
    /// Java `CODE_10002`.
    pub const CODE_10002: i32 = 10002;
    /// Java `CODE_10003`.
    pub const CODE_10003: i32 = 10003;
    /// Java `CODE_10004`.
    pub const CODE_10004: i32 = 10004;
    /// Java `CODE_10011`.
    pub const CODE_10011: i32 = 10011;
    /// Java `CODE_10021`.
    pub const CODE_10021: i32 = 10021;
    /// Java `CODE_10022`.
    pub const CODE_10022: i32 = 10022;
    /// Java `CODE_10031`.
    pub const CODE_10031: i32 = 10031;
    /// Java `CODE_10032`.
    pub const CODE_10032: i32 = 10032;
    /// Java `CODE_10301`.
    pub const CODE_10301: i32 = 10301;
    /// Java `CODE_10311`.
    pub const CODE_10311: i32 = 10311;
    /// Java `CODE_10312`.
    pub const CODE_10312: i32 = 10312;
    /// Java `CODE_10321`.
    pub const CODE_10321: i32 = 10321;
    /// Java `CODE_11001`.
    pub const CODE_11001: i32 = 11001;
    /// Java `CODE_11002`.
    pub const CODE_11002: i32 = 11002;
    /// Java `CODE_11003`.
    pub const CODE_11003: i32 = 11003;
    /// Java `CODE_11004`.
    pub const CODE_11004: i32 = 11004;
    /// Java `CODE_11011`.
    pub const CODE_11011: i32 = 11011;
    /// Java `CODE_11012`.
    pub const CODE_11012: i32 = 11012;
    /// Java `CODE_11013`.
    pub const CODE_11013: i32 = 11013;
    /// Java `CODE_11014`.
    pub const CODE_11014: i32 = 11014;
    /// Java `CODE_11015`.
    pub const CODE_11015: i32 = 11015;
    /// Java `CODE_11016`.
    pub const CODE_11016: i32 = 11016;
    /// Java `CODE_11017`.
    pub const CODE_11017: i32 = 11017;
    /// Java `CODE_11031`.
    pub const CODE_11031: i32 = 11031;
    /// Java `CODE_11041`.
    pub const CODE_11041: i32 = 11041;
    /// Java `CODE_11051`.
    pub const CODE_11051: i32 = 11051;
    /// Java `CODE_11061`.
    pub const CODE_11061: i32 = 11061;
    /// Java `CODE_11062`.
    pub const CODE_11062: i32 = 11062;
    /// Java `CODE_11063`.
    pub const CODE_11063: i32 = 11063;
    /// Java `CODE_11064`.
    pub const CODE_11064: i32 = 11064;
    /// Java `CODE_11071`.
    pub const CODE_11071: i32 = 11071;
    /// Java `CODE_11072`.
    pub const CODE_11072: i32 = 11072;
    /// Java `CODE_11073`.
    pub const CODE_11073: i32 = 11073;
    /// Java `CODE_11074`.
    pub const CODE_11074: i32 = 11074;
    /// Java `CODE_12001`.
    pub const CODE_12001: i32 = 12001;
    /// Java `CODE_12002`.
    pub const CODE_12002: i32 = 12002;
    /// Java `CODE_12003`.
    pub const CODE_12003: i32 = 12003;
    /// Java `CODE_12101`.
    pub const CODE_12101: i32 = 12101;
    /// Java `CODE_12102`.
    pub const CODE_12102: i32 = 12102;
    /// Java `CODE_12103`.
    pub const CODE_12103: i32 = 12103;
    /// Java `CODE_12104`.
    pub const CODE_12104: i32 = 12104;
    /// Java `CODE_12111`.
    pub const CODE_12111: i32 = 12111;
    /// Java `CODE_12112`.
    pub const CODE_12112: i32 = 12112;
    /// Java `CODE_12113`.
    pub const CODE_12113: i32 = 12113;
    /// Java `CODE_121131`.
    pub const CODE_121131: i32 = 121131;
    /// Java `CODE_121132`.
    pub const CODE_121132: i32 = 121132;
    /// Java `CODE_12114`.
    pub const CODE_12114: i32 = 12114;
    /// Java `CODE_12115`.
    pub const CODE_12115: i32 = 12115;
    /// Java `CODE_12116`.
    pub const CODE_12116: i32 = 12116;
    /// Java `CODE_12117`.
    pub const CODE_12117: i32 = 12117;
    /// Java `CODE_12118`.
    pub const CODE_12118: i32 = 12118;
    /// Java `CODE_12119`.
    pub const CODE_12119: i32 = 12119;
    /// Java `CODE_12401`.
    pub const CODE_12401: i32 = 12401;
}

#[cfg(test)]
mod tests {
    use super::SaErrorCode;

    #[test]
    fn representative_codes_match_java_baseline() {
        assert_eq!(SaErrorCode::CODE_UNDEFINED, -1);
        assert_eq!(SaErrorCode::CODE_11001, 11001);
        assert_eq!(SaErrorCode::CODE_121131, 121131);
        assert_eq!(SaErrorCode::CODE_12401, 12401);
    }
}
