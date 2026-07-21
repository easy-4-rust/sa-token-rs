# Sa-Token Java 完整目录树

> 基线：Sa-Token Java `dev` 分支 `89e47c12`
> 用 code-review-graph + 文件系统扫描生成，覆盖 `sa-token-core/src/main/java/cn/dev33/satoken` 全部目录与文件。
> 仅记录 Java 核心库（`sa-token-core`），不含 `sa-token-plugin` / `sa-token-starter` / `sa-token-demo`。

---

## 目录树

```text
sa-token-core/src/main/java/cn/dev33/satoken/
├── SaManager.java
│
├── annotation/
│   ├── SaCheckDisable.java
│   ├── SaCheckHttpBasic.java
│   ├── SaCheckHttpDigest.java
│   ├── SaCheckLogin.java
│   ├── SaCheckOr.java
│   ├── SaCheckPermission.java
│   ├── SaCheckRole.java
│   ├── SaCheckSafe.java
│   ├── SaIgnore.java
│   ├── SaMode.java
│   └── handler/
│       ├── SaAnnotationHandlerInterface.java
│       ├── SaCheckDisableHandler.java
│       ├── SaCheckHttpBasicHandler.java
│       ├── SaCheckHttpDigestHandler.java
│       ├── SaCheckLoginHandler.java
│       ├── SaCheckOrHandler.java
│       ├── SaCheckPermissionHandler.java
│       ├── SaCheckRoleHandler.java
│       ├── SaCheckSafeHandler.java
│       └── SaIgnoreHandler.java
│
├── application/
│   ├── ApplicationInfo.java
│   ├── SaApplication.java
│   ├── SaGetValueInterface.java
│   └── SaSetValueInterface.java
│
├── config/
│   ├── SaCookieConfig.java
│   ├── SaTokenConfig.java
│   └── SaTokenConfigFactory.java
│
├── context/
│   ├── SaHolder.java
│   ├── SaTokenContext.java
│   ├── SaTokenContextDefaultImpl.java
│   ├── SaTokenContextForReadOnly.java
│   ├── SaTokenContextForThreadLocal.java
│   ├── SaTokenContextForThreadLocalStaff.java
│   ├── mock/
│   │   ├── SaRequestForMock.java
│   │   ├── SaResponseForMock.java
│   │   ├── SaStorageForMock.java
│   │   └── SaTokenContextMockUtil.java
│   └── model/
│       ├── package-info.java
│       ├── SaCookie.java
│       ├── SaRequest.java
│       ├── SaResponse.java
│       ├── SaStorage.java
│       └── SaTokenContextModelBox.java
│
├── dao/
│   ├── SaTokenDao.java
│   ├── SaTokenDaoDefaultImpl.java
│   ├── auto/
│   │   ├── SaTokenDaoByObjectFollowString.java
│   │   ├── SaTokenDaoBySessionFollowObject.java
│   │   └── SaTokenDaoByStringFollowObject.java
│   └── timedcache/
│       ├── SaMapPackage.java
│       ├── SaMapPackageForConcurrentHashMap.java
│       └── SaTimedCache.java
│
├── error/
│   └── SaErrorCode.java
│
├── exception/
│   ├── ApiDisabledException.java
│   ├── BackResultException.java
│   ├── DisableServiceException.java
│   ├── FirewallCheckException.java
│   ├── InvalidContextException.java
│   ├── NotHttpBasicAuthException.java
│   ├── NotHttpDigestAuthException.java
│   ├── NotImplException.java
│   ├── NotLoginException.java
│   ├── NotPermissionException.java
│   ├── NotRoleException.java
│   ├── NotSafeException.java
│   ├── NotWebContextException.java
│   ├── RequestPathInvalidException.java
│   ├── SaJsonConvertException.java
│   ├── SameTokenInvalidException.java
│   ├── SaTokenContextException.java
│   ├── SaTokenException.java
│   ├── SaTokenPluginException.java
│   ├── StopMatchException.java
│   └── TotpAuthException.java
│
├── filter/
│   ├── SaFilter.java
│   ├── SaFilterAuthStrategy.java
│   └── SaFilterErrorStrategy.java
│
├── fun/
│   ├── IsRunFunction.java
│   ├── SaFunction.java
│   ├── SaParamFunction.java
│   ├── SaParamRetFunction.java
│   ├── SaRetFunction.java
│   ├── SaRetGenericFunction.java
│   ├── SaRouteFunction.java
│   ├── SaTwoParamFunction.java
│   ├── hooks/
│   │   └── SaTokenPluginHookFunction.java
│   └── strategy/
│       ├── SaAutoRenewFunction.java
│       ├── SaCheckElementAnnotationFunction.java
│       ├── SaCheckELRootMapExtendFunction.java
│       ├── SaCheckMethodAnnotationFunction.java
│       ├── SaCheckOrAnnotationFunction.java
│       ├── SaCorsHandleFunction.java
│       ├── SaCreateSessionFunction.java
│       ├── SaCreateStpLogicFunction.java
│       ├── SaCreateTokenFunction.java
│       ├── SaFirewallCheckFailHandleFunction.java
│       ├── SaFirewallCheckFunction.java
│       ├── SaGenerateUniqueTokenFunction.java
│       ├── SaGetAnnotationFunction.java
│       ├── SaHasElementFunction.java
│       ├── SaIsAnnotationPresentFunction.java
│       └── SaRouteMatchFunction.java
│
├── http/
│   ├── SaHttpTemplate.java
│   ├── SaHttpTemplateDefaultImpl.java
│   └── SaHttpUtil.java
│
├── httpauth/
│   ├── basic/
│   │   ├── SaHttpBasicAccount.java
│   │   ├── SaHttpBasicTemplate.java
│   │   └── SaHttpBasicUtil.java
│   └── digest/
│       ├── SaHttpDigestModel.java
│       ├── SaHttpDigestTemplate.java
│       └── SaHttpDigestUtil.java
│
├── json/
│   ├── SaJsonTemplate.java
│   └── SaJsonTemplateDefaultImpl.java
│
├── listener/
│   ├── SaTokenEventCenter.java
│   ├── SaTokenListener.java
│   ├── SaTokenListenerForLog.java
│   └── SaTokenListenerForSimple.java
│
├── log/
│   ├── SaLog.java
│   └── SaLogForConsole.java
│
├── model/
│   └── wrapperInfo/
│       └── SaDisableWrapperInfo.java
│
├── plugin/
│   ├── SaTokenPlugin.java
│   ├── SaTokenPluginHolder.java
│   └── SaTokenPluginHookModel.java
│
├── router/
│   ├── SaHttpMethod.java
│   ├── SaRouter.java
│   └── SaRouterStaff.java
│
├── same/
│   ├── SaSameTemplate.java
│   └── SaSameUtil.java
│
├── secure/
│   ├── BCrypt.java
│   ├── SaBase32Util.java
│   ├── SaBase64Util.java
│   ├── SaSecureUtil.java
│   └── totp/
│       ├── SaTotpTemplate.java
│       └── SaTotpUtil.java
│
├── serializer/
│   ├── SaSerializerTemplate.java
│   └── impl/
│       ├── SaSerializerTemplateForJdk.java
│       ├── SaSerializerTemplateForJdkUseBase64.java
│       ├── SaSerializerTemplateForJdkUseHex.java
│       ├── SaSerializerTemplateForJdkUseISO_8859_1.java
│       └── SaSerializerTemplateForJson.java
│
├── session/
│   ├── SaSession.java
│   ├── SaSessionCustomUtil.java
│   ├── SaTerminalInfo.java
│   └── raw/
│       ├── SaRawSessionDelegator.java
│       └── SaRawSessionUtil.java
│
├── stp/
│   ├── SaLoginConfig.java
│   ├── SaLoginModel.java
│   ├── SaTokenInfo.java
│   ├── StpInterface.java
│   ├── StpInterfaceDefaultImpl.java
│   ├── StpLogic.java
│   ├── StpUtil.java
│   └── parameter/
│       ├── SaLoginParameter.java
│       ├── SaLogoutParameter.java
│       └── enums/
│           ├── SaLogoutMode.java
│           ├── SaLogoutRange.java
│           ├── SaReplacedLoginExitMode.java
│           └── SaReplacedRange.java
│
├── strategy/
│   ├── SaAnnotationStrategy.java
│   ├── SaFirewallStrategy.java
│   ├── SaStrategy.java
│   └── hooks/
│       ├── SaFirewallCheckHook.java
│       ├── SaFirewallCheckHookForBlackPath.java
│       ├── SaFirewallCheckHookForDirectoryTraversal.java
│       ├── SaFirewallCheckHookForHeader.java
│       ├── SaFirewallCheckHookForHost.java
│       ├── SaFirewallCheckHookForHttpMethod.java
│       ├── SaFirewallCheckHookForParameter.java
│       ├── SaFirewallCheckHookForPathBannedCharacter.java
│       ├── SaFirewallCheckHookForPathDangerCharacter.java
│       └── SaFirewallCheckHookForWhitePath.java
│
├── temp/
│   ├── SaTempTemplate.java
│   └── SaTempUtil.java
│
└── util/
    ├── SaFoxUtil.java
    ├── SaHexUtil.java
    ├── SaResult.java
    ├── SaSugar.java
    ├── SaTokenConsts.java
    ├── SaTtlMethods.java
    ├── SaValue2Box.java
    └── StrFormatter.java
```

---

## 统计

| 指标 | 数值 |
|---|---|
| 一级目录数 | 22 |
| 顶层 Java 文件 | 1 (SaManager.java) |
| 总 Java 文件数 | ~155 |
| 注解类 | 10 |
| 异常类 | 20 |
| 函数式接口 | 15+ |
| 策略 Hook | 9 |
| 序列化实现 | 5 |
| 核心类（StpLogic/StpUtil/SaSession 等） | 8 |

---

## 与 Rust 版对照

参见 [rust-tree-full.md](./rust-tree-full.md) 和 [project-tree-diff.md](./project-tree-diff.md)。
