//! Integration contract against a real Redis process.

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use sa_token_core::dao::AsyncSaTokenDao;
use sa_token_dao_redis::SaTokenDaoRedis;

struct RedisProcess {
    child: Option<Child>,
    data_dir: Option<PathBuf>,
    url: String,
}

impl RedisProcess {
    fn start() -> Self {
        if let Ok(url) = std::env::var("SA_TOKEN_REDIS_URL") {
            return Self {
                child: None,
                data_dir: None,
                url,
            };
        }

        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve Redis test port");
        let port = listener.local_addr().expect("read Redis test port").port();
        drop(listener);

        let data_dir = std::env::temp_dir().join(format!(
            "sa-token-redis-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&data_dir).expect("create Redis test data directory");
        let child = Command::new("redis-server")
            .args([
                "--bind",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--save",
                "",
                "--appendonly",
                "no",
                "--dir",
                data_dir.to_str().expect("UTF-8 Redis data directory"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start redis-server; set SA_TOKEN_REDIS_URL to use an external service");
        Self {
            child: Some(child),
            data_dir: Some(data_dir),
            url: format!("redis://127.0.0.1:{port}/"),
        }
    }

    async fn connect(&self) -> SaTokenDaoRedis {
        let mut last_error = None;
        for _ in 0..50 {
            let client = redis::Client::open(self.url.clone()).expect("valid Redis test URL");
            match SaTokenDaoRedis::connect(client).await {
                Ok(dao) => return dao,
                Err(error) => last_error = Some(error),
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("Redis did not become ready: {last_error:?}");
    }
}

impl Drop for RedisProcess {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(data_dir) = &self.data_dir {
            let _ = std::fs::remove_dir_all(data_dir);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_redis_preserves_ttl_scans_pages_and_propagates_errors() {
    let redis = RedisProcess::start();
    let dao = Arc::new(redis.connect().await);
    let namespace = format!("satoken:test:{}", uuid::Uuid::new_v4().simple());

    let ttl_key = format!("{namespace}:ttl");
    dao.set(&ttl_key, "before", 30).await.expect("set TTL key");
    let ttl_before = dao.get_timeout(&ttl_key).await.expect("read initial TTL");
    dao.update(&ttl_key, "after")
        .await
        .expect("atomic update preserving TTL");
    let ttl_after = dao.get_timeout(&ttl_key).await.expect("read updated TTL");
    assert!((1..=30).contains(&ttl_after));
    assert!(ttl_after <= ttl_before);
    assert_eq!(
        dao.get(&ttl_key).await.expect("read updated value"),
        Some("after".to_string())
    );

    for index in 0..35 {
        dao.set(&format!("{namespace}:scan:{index:02}"), "value", 60)
            .await
            .expect("seed SCAN key");
    }
    let page = dao
        .search_data(&format!("{namespace}:scan:"), "", 5, 10, true)
        .await
        .expect("SCAN page");
    assert_eq!(page.len(), 10);
    assert!(page.windows(2).all(|pair| pair[0] < pair[1]));

    let corrupt_session_key = format!("{namespace}:corrupt-session");
    dao.set(&corrupt_session_key, "not-json", 60)
        .await
        .expect("seed corrupt session");
    assert!(dao.get_session(&corrupt_session_key).await.is_err());

    let mut tasks = Vec::new();
    for index in 0..16 {
        let dao = Arc::clone(&dao);
        let key = format!("{namespace}:concurrent:{index}");
        tasks.push(tokio::spawn(async move {
            dao.set(&key, "ok", 60).await?;
            dao.get(&key).await
        }));
    }
    for task in tasks {
        assert_eq!(
            task.await
                .expect("concurrent Redis task")
                .expect("concurrent Redis operation"),
            Some("ok".to_string())
        );
    }

    let unused = TcpListener::bind("127.0.0.1:0").expect("reserve unavailable port");
    let unavailable_url = format!(
        "redis://{}/",
        unused.local_addr().expect("read unavailable port")
    );
    let unavailable = redis::Client::open(unavailable_url).expect("valid unavailable URL");
    assert!(SaTokenDaoRedis::connect(unavailable).await.is_err());

    for key in dao
        .search_data(&namespace, "", 0, -1, true)
        .await
        .expect("find cleanup keys")
    {
        dao.delete(&key).await.expect("delete cleanup key");
    }
}
