package cn.dev33.satoken.golden;

import cn.dev33.satoken.SaManager;
import cn.dev33.satoken.config.SaTokenConfig;
import cn.dev33.satoken.stp.StpLogic;
import cn.dev33.satoken.serializer.impl.SaSerializerTemplateForJdkUseBase64;
import cn.dev33.satoken.serializer.impl.SaSerializerTemplateForJdkUseHex;
import cn.dev33.satoken.serializer.impl.SaSerializerTemplateForJdkUseISO_8859_1;
import cn.dev33.satoken.apikey.config.SaApiKeyConfig;
import cn.dev33.satoken.apikey.error.SaApiKeyErrorCode;
import cn.dev33.satoken.apikey.template.SaApiKeyTemplate;
import cn.dev33.satoken.serializer.SaSerializerForBase64UseEmoji;
import cn.dev33.satoken.serializer.SaSerializerForBase64UsePeriodicTable;
import cn.dev33.satoken.serializer.SaSerializerForBase64UseSpecialSymbols;
import cn.dev33.satoken.serializer.SaSerializerForBase64UseTianGan;
import cn.dev33.satoken.jwt.SaJwtUtil;
import cn.dev33.satoken.sign.config.SaSignConfig;
import cn.dev33.satoken.sign.template.SaSignTemplate;
import cn.dev33.satoken.oauth2.config.SaOAuth2OidcConfig;
import cn.dev33.satoken.oauth2.consts.GrantType;
import cn.dev33.satoken.oauth2.consts.SaOAuth2Consts;
import cn.dev33.satoken.oauth2.error.SaOAuth2ErrorCode;
import cn.dev33.satoken.oauth2.dao.SaOAuth2Dao;
import cn.dev33.satoken.oauth2.data.loader.SaOAuth2DataLoaderDefaultImpl;
import cn.dev33.satoken.sso.config.SaSsoClientConfig;
import cn.dev33.satoken.sso.config.SaSsoServerConfig;
import cn.dev33.satoken.sso.error.SaSsoErrorCode;
import cn.dev33.satoken.sso.name.ApiName;
import cn.dev33.satoken.sso.name.ParamName;
import cn.dev33.satoken.sso.SaSsoManager;
import cn.dev33.satoken.sso.template.SaSsoClientTemplate;
import cn.dev33.satoken.sso.template.SaSsoServerTemplate;
import cn.dev33.satoken.sso.util.SaSsoConsts;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.LinkedHashMap;
import java.util.Map;

/** Exports deterministic core values from the pinned Java baseline. */
public final class CoreGoldenExporter {

    private CoreGoldenExporter() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("expected: <output-file> <source-commit>");
        }

        SaTokenConfig config = new SaTokenConfig();
        SaManager.setConfig(config);
        StpLogic logic = new StpLogic("login");
        byte[] codecInput = "SaToken".getBytes(StandardCharsets.ISO_8859_1);
        SaApiKeyConfig apiKeyConfig = new SaApiKeyConfig();
        SaApiKeyTemplate apiKeyTemplate = new SaApiKeyTemplate();
        Map<String, Object> jwtPayload = new LinkedHashMap<>();
        jwtPayload.put("loginType", "login");
        jwtPayload.put("loginId", 10001);
        jwtPayload.put("deviceType", "web");
        jwtPayload.put("eff", -1);
        jwtPayload.put("rnStr", "0123456789abcdef0123456789abcdef");
        String javaJwt = SaJwtUtil.createToken(jwtPayload, "java-golden-secret");
        Map<String, Object> signPayload = new LinkedHashMap<>();
        signPayload.put("b", "2");
        signPayload.put("a", "1");
        SaSignConfig signConfig = new SaSignConfig("secret");
        String javaSign = new SaSignTemplate(signConfig).createSign(signPayload);
        SaSsoClientConfig ssoClient = new SaSsoClientConfig();
        ssoClient.setServerUrl("https://sso.example").setClient("app-a");
        SaSsoManager.setClientConfig(ssoClient);
        SaSsoServerConfig ssoServer = new SaSsoServerConfig();
        SaSsoClientTemplate ssoClientTemplate = new SaSsoClientTemplate();
        SaSsoServerTemplate ssoServerTemplate = new SaSsoServerTemplate();
        SaOAuth2Dao oauth2Dao = new SaOAuth2Dao();
        SaOAuth2DataLoaderDefaultImpl oauth2Loader = new SaOAuth2DataLoaderDefaultImpl();
        String ssoServerAuthUrl = ssoClientTemplate.buildServerAuthUrl(
                "https://client.example/sso/login", "/home?a=1");
        ApiName ssoApi = new ApiName();
        ParamName ssoParam = new ParamName();
        String json = "{\n"
                + "  \"source_commit\": \"" + escape(args[1]) + "\",\n"
                + "  \"token_name\": \"" + escape(config.getTokenName()) + "\",\n"
                + "  \"timeout\": " + config.getTimeout() + ",\n"
                + "  \"active_timeout\": " + config.getActiveTimeout() + ",\n"
                + "  \"is_concurrent\": " + config.getIsConcurrent() + ",\n"
                + "  \"max_login_count\": " + config.getMaxLoginCount() + ",\n"
                + "  \"same_token_timeout\": " + config.getSameTokenTimeout() + ",\n"
                + "  \"token_session_check_login\": " + config.getTokenSessionCheckLogin() + ",\n"
                + "  \"auto_renew\": " + config.getAutoRenew() + ",\n"
                + "  \"token_key\": \"" + escape(logic.splicingKeyTokenValue("TOKEN")) + "\",\n"
                + "  \"session_key\": \"" + escape(logic.splicingKeySession("10001")) + "\",\n"
                + "  \"token_session_key\": \"" + escape(logic.splicingKeyTokenSession("TOKEN")) + "\",\n"
                + "  \"last_active_key\": \"" + escape(logic.splicingKeyLastActiveTime("TOKEN")) + "\",\n"
                + "  \"switch_key\": \"" + escape(logic.splicingKeySwitch()) + "\",\n"
                + "  \"disable_key\": \"" + escape(logic.splicingKeyDisable("10001", "login")) + "\",\n"
                + "  \"disable_service_key\": \"" + escape(logic.splicingKeyDisable("10001", "account")) + "\",\n"
                + "  \"safe_key\": \"" + escape(logic.splicingKeySafe("TOKEN", "")) + "\",\n"
                + "  \"safe_service_key\": \"" + escape(logic.splicingKeySafe("TOKEN", "payment")) + "\",\n"
                + "  \"serializer_base64\": \"" + escape(new SaSerializerTemplateForJdkUseBase64().bytesToString(codecInput)) + "\",\n"
                + "  \"serializer_hex\": \"" + escape(new SaSerializerTemplateForJdkUseHex().bytesToString(codecInput)) + "\",\n"
                + "  \"serializer_iso_8859_1\": \"" + escape(new SaSerializerTemplateForJdkUseISO_8859_1().bytesToString(codecInput)) + "\",\n"
                + "  \"serializer_emoji\": \"" + escape(new SaSerializerForBase64UseEmoji().bytesToString(codecInput)) + "\",\n"
                + "  \"serializer_periodic_table\": \"" + escape(new SaSerializerForBase64UsePeriodicTable().bytesToString(codecInput)) + "\",\n"
                + "  \"serializer_special_symbols\": \"" + escape(new SaSerializerForBase64UseSpecialSymbols().bytesToString(codecInput)) + "\",\n"
                + "  \"serializer_tian_gan\": \"" + escape(new SaSerializerForBase64UseTianGan().bytesToString(codecInput)) + "\",\n"
                + "  \"jwt_hs256_token\": \"" + escape(javaJwt) + "\",\n"
                + "  \"sign_default_timestamp_disparity\": " + new SaSignConfig().getTimestampDisparity() + ",\n"
                + "  \"sign_default_digest\": \"" + escape(new SaSignConfig().getDigestAlgo()) + "\",\n"
                + "  \"sign_md5\": \"" + escape(javaSign) + "\",\n"
                + "  \"sso_client_auth_url\": \"" + escape(ssoClient.getAuthUrl()) + "\",\n"
                + "  \"sso_client_is_http\": " + ssoClient.getIsHttp() + ",\n"
                + "  \"sso_client_is_slo\": " + ssoClient.getIsSlo() + ",\n"
                + "  \"sso_server_ticket_timeout\": " + ssoServer.getTicketTimeout() + ",\n"
                + "  \"sso_server_max_reg_client\": " + ssoServer.getMaxRegClient() + ",\n"
                + "  \"sso_api_check_ticket\": \"" + escape(ssoApi.ssoCheckTicket) + "\",\n"
                + "  \"sso_param_secretkey\": \"" + escape(ssoParam.secretkey) + "\",\n"
                + "  \"sso_client_wildcard\": \"" + escape(SaSsoConsts.CLIENT_WILDCARD) + "\",\n"
                + "  \"sso_last_error_code\": " + SaSsoErrorCode.CODE_30024 + ",\n"
                + "  \"sso_server_auth_url\": \"" + escape(ssoServerAuthUrl) + "\",\n"
                + "  \"sso_ticket_key\": \"" + escape(ssoServerTemplate.splicingTicketModelSaveKey("TICKET")) + "\",\n"
                + "  \"sso_ticket_index_key\": \"" + escape(ssoServerTemplate.splicingTicketIndexKey("", 10001)) + "\",\n"
                + "  \"sso_encoded_back_url\": \"" + escape(ssoServerTemplate.encodeBackParam("https://client.example/sso/login?back=/home?a=1")) + "\",\n"
                + "  \"oauth2_oidc_id_token_timeout\": " + new SaOAuth2OidcConfig().getIdTokenTimeout() + ",\n"
                + "  \"oauth2_grant_authorization_code\": \"" + escape(GrantType.authorization_code) + "\",\n"
                + "  \"oauth2_authorize_api\": \"" + escape(SaOAuth2Consts.Api.authorize) + "\",\n"
                + "  \"oauth2_finally_work_scope\": \"" + escape(SaOAuth2Consts._FINALLY_WORK_SCOPE) + "\",\n"
                + "  \"oauth2_last_error_code\": " + SaOAuth2ErrorCode.CODE_30191 + ",\n"
                + "  \"oauth2_code_key\": \"" + escape(oauth2Dao.splicingCodeSaveKey("C")) + "\",\n"
                + "  \"oauth2_code_index_key\": \"" + escape(oauth2Dao.splicingCodeIndexKey("app-a", 10001)) + "\",\n"
                + "  \"oauth2_access_token_key\": \"" + escape(oauth2Dao.splicingAccessTokenSaveKey("A")) + "\",\n"
                + "  \"oauth2_access_token_rsd\": \"" + escape(oauth2Dao.splicingAccessTokenRSDValue("app-a", 10001)) + "\",\n"
                + "  \"oauth2_refresh_token_key\": \"" + escape(oauth2Dao.splicingRefreshTokenSaveKey("R")) + "\",\n"
                + "  \"oauth2_client_token_key\": \"" + escape(oauth2Dao.splicingClientTokenSaveKey("CT")) + "\",\n"
                + "  \"oauth2_grant_scope_key\": \"" + escape(oauth2Dao.splicingGrantScopeKey("app-a", 10001)) + "\",\n"
                + "  \"oauth2_state_key\": \"" + escape(oauth2Dao.splicingStateSaveKey("S")) + "\",\n"
                + "  \"oauth2_nonce_key\": \"" + escape(oauth2Dao.splicingCodeNonceIndexSaveKey("C")) + "\",\n"
                + "  \"oauth2_openid\": \"" + escape(oauth2Loader.getOpenid("app-a", 10001)) + "\",\n"
                + "  \"oauth2_unionid\": \"" + escape(oauth2Loader.getUnionid("subject-a", 10001)) + "\",\n"
                + "  \"api_key_prefix\": \"" + escape(apiKeyConfig.getPrefix()) + "\",\n"
                + "  \"api_key_timeout\": " + apiKeyConfig.getTimeout() + ",\n"
                + "  \"api_key_record_index\": " + apiKeyConfig.getIsRecordIndex() + ",\n"
                + "  \"api_key_save_key\": \"" + escape(apiKeyTemplate.splicingApiKeySaveKey("AK-TEST")) + "\",\n"
                + "  \"api_key_invalid_code\": " + SaApiKeyErrorCode.CODE_12301 + ",\n"
                + "  \"api_key_scope_code\": " + SaApiKeyErrorCode.CODE_12311 + "\n"
                + "}\n";

        Path output = Paths.get(args[0]).toAbsolutePath().normalize();
        Files.createDirectories(output.getParent());
        Files.write(output, json.getBytes(StandardCharsets.UTF_8));
    }

    private static String escape(String value) {
        return value.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}
