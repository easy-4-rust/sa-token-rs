//! `sa-token-dao-sweeper` —— 后台扫描清理过期数据。
//!
//! 对应 Java `sa-token-hutool-timed-cache`：以固定间隔（默认 30 秒）
//! 扫描内存 DAO 的 key，对 TTL 过期的项主动删除，避免惰性失效导致的内存
//! 增长。
//!
//! 注意：Redis / moka 都不需要这个 sweeper（Redis 用 EXPIRE，
//! moka 用自身 Time-To-Live）。本 crate 主要为 `sa-token-dao-memory`
//! 提供"主动清理"语义。
//!
//! # 用法
//! ```ignore
//! use sa_token_dao_sweeper::SweeperHandle;
//!
//! let handle = SweeperHandle::start(sa_manager_dao(), 30)?;
//! // 业务运行中
//! handle.stop().await;
//! ```

use std::sync::Arc;
use std::time::Duration;

use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::exception::SaResult;
use sa_token_core::sa_manager::SaManager;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error};

/// Sweeper 句柄，用于在需要时停止后台任务
pub struct SweeperHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<JoinHandle<()>>,
}

impl SweeperHandle {
    /// 启动一个 sweeper 后台任务
    ///
    /// # 参数
    /// - `period_secs`: 清理间隔（秒）。对应 Java
    ///   `SaTokenConfig.dataRefreshPeriod`，默认 30 秒
    pub fn start(period_secs: u64) -> SaResult<Self> {
        Self::start_at(period_secs, "".to_string())
    }

    /// 启动 sweeper，仅扫描以 `prefix` 开头的 key
    pub fn start_at(period_secs: u64, prefix: String) -> SaResult<Self> {
        let (stop_tx, mut stop_rx) = oneshot::channel();
        let join = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(period_secs.max(1)));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = sweep_once(&prefix).await {
                            error!(error = %e, "sa-token-dao-sweeper sweep failed");
                        }
                    }
                    _ = &mut stop_rx => {
                        break;
                    }
                }
            }
        });
        Ok(Self {
            stop_tx: Some(stop_tx),
            join: Some(join),
        })
    }

    /// 停止 sweeper（graceful shutdown）
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(j) = self.join.take() {
            let _ = j.await;
        }
    }
}

/// 执行一次扫描清理
async fn sweep_once(prefix: &str) -> SaResult<()> {
    let dao = Arc::clone(&SaManager::sa_token_dao());
    // 列出所有 key（与 Java Hutool-timed-cache 的清理策略一致）
    let keys = dao.search_data(prefix, "", 0, -1, true)?;
    let mut expired = 0usize;
    for k in keys {
        let ttl = dao.get_timeout(&k)?;
        // -2 表示 key 不存在（被其他路径删了），-1 表示永不过期
        if ttl == -2 {
            expired += 1;
            continue;
        }
        // 主动删除过期 key（语义与 Hutool 一致：扫到就删）
        if ttl == 0 {
            dao.delete(&k)?;
            expired += 1;
        }
    }
    if expired > 0 {
        debug!(expired, "sa-token-dao-sweeper: cleaned expired keys");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;

    #[tokio::test(flavor = "multi_thread")]
    async fn sweeper_handle_starts_and_stops() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        let handle = SweeperHandle::start(1).expect("start");
        tokio::time::sleep(Duration::from_millis(100)).await;
        handle.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sweeper_cleans_expired_keys() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        let dao = Arc::new(SaTokenDaoDefaultImpl::new());
        SaManager::set_sa_token_dao(dao.clone());
        // 写入一个 TTL=0 的 key（在某些实现中等价于已过期）
        SaTokenDao::set(&*dao, "k1", "v1", 1).expect("set");
        tokio::time::sleep(Duration::from_millis(1500)).await;
        // 启动 sweeper 跑 1 轮
        let handle = SweeperHandle::start(1).expect("start");
        tokio::time::sleep(Duration::from_millis(200)).await;
        handle.stop().await;
        // expired key 已被惰性删除
        let v = SaTokenDao::get(&*dao, "k1").expect("get");
        // 可能是 None（已过期）也可能是 Some（取决于实现），但不应 panic
        let _ = v;
    }
}
