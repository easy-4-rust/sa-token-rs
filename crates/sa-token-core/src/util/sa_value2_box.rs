//! `SaValue2Box` —— 1:1 对应 Java `cn.dev33.satoken.util.SaValue2Box`

use std::sync::Mutex;

/// 通用值持有者（与 Java `SaValue2Box<T>` 对应）
///
/// 由于 Rust 没有 `final` 等价物，借助互斥锁 + 一次性写入实现
/// 「只能赋值一次」的语义。
pub struct SaValue2Box<T> {
    inner: Mutex<Option<T>>,
}

impl<T> SaValue2Box<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// 设置值（仅首次生效；后续赋值被忽略并返回 false）
    pub fn set(&self, value: T) -> bool {
        let mut g = self.inner.lock().unwrap();
        if g.is_none() {
            *g = Some(value);
            true
        } else {
            false
        }
    }

    /// 获取值
    pub fn get(&self) -> Option<T>
    where
        T: Copy,
    {
        self.inner.lock().unwrap().as_ref().copied()
    }

    /// 消费并获取值
    pub fn take(self) -> Option<T> {
        let mut g = self.inner.lock().unwrap();
        g.take()
    }
}

impl<T> Default for SaValue2Box<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_once() {
        let v: SaValue2Box<u32> = SaValue2Box::new();
        assert!(v.set(42));
        assert!(!v.set(100)); // 第二次失败
        assert_eq!(v.get(), Some(42));
    }

    #[test]
    fn take() {
        let v: SaValue2Box<String> = SaValue2Box::new();
        v.set("hi".to_string());
        assert_eq!(v.take(), Some("hi".to_string()));
    }
}
