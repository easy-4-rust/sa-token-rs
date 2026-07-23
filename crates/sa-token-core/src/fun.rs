//! Fun 模块（对应 Java `cn.dev33.satoken.fun`）。

pub mod hooks;
pub mod strategy;

// 1:1 对齐 Java `cn.dev33.satoken.fun`（根包）
pub mod is_run_function;
pub mod sa_function;
pub mod sa_param_function;
pub mod sa_param_ret_function;
pub mod sa_ret_function;
pub mod sa_ret_generic_function;
pub mod sa_route_function;
pub mod sa_two_param_function;

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::is_run_function::IsRunFunction;
    use super::sa_function::SaFunction;
    use super::sa_param_function::SaParamFunction;
    use super::sa_param_ret_function::SaParamRetFunction;
    use super::sa_ret_function::{SaAnyValue, SaRetFunction};
    use super::sa_ret_generic_function::SaRetGenericFunction;
    use super::sa_two_param_function::SaTwoParamFunction;

    struct DynamicReturn;

    impl SaRetFunction for DynamicReturn {
        fn run(&self) -> SaAnyValue {
            Box::new("value".to_owned())
        }
    }

    #[test]
    fn is_run_function_selects_exactly_one_branch() {
        let mut value = 0;
        IsRunFunction::new(true)
            .exe(|| value += 1)
            .no_exe(|| value += 10);
        assert_eq!(value, 1);

        IsRunFunction::new(false)
            .exe(|| value += 100)
            .no_exe(|| value += 2);
        assert_eq!(value, 3);
    }

    #[test]
    fn function_traits_preserve_java_arity_and_return_contracts() {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        RUNS.store(0, Ordering::SeqCst);
        SaFunction::run(&|| {
            RUNS.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(RUNS.load(Ordering::SeqCst), 1);

        let captured = AtomicUsize::new(0);
        SaParamFunction::run(&|value| captured.store(value, Ordering::SeqCst), 7);
        assert_eq!(captured.load(Ordering::SeqCst), 7);

        let doubled = SaParamRetFunction::run(&|value| value * 2, 6);
        assert_eq!(doubled, 12);

        let pair_sum = AtomicUsize::new(0);
        SaTwoParamFunction::run(
            &|first: usize, second: usize| {
                pair_sum.store(first + second, Ordering::SeqCst);
            },
            4,
            5,
        );
        assert_eq!(pair_sum.load(Ordering::SeqCst), 9);

        let generic = SaRetGenericFunction::run(&|| "generic".to_owned());
        assert_eq!(generic, "generic");
        let dynamic = DynamicReturn.run();
        assert_eq!(
            dynamic.downcast_ref::<String>().expect("dynamic String"),
            "value"
        );
    }
}
