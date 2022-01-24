//! Provides a basic const assert macro

#[macro_export]
macro_rules! const_assert {
    ($test:expr) => {
        const _TEST: bool = $test;
        const _ASSERT: [(); 1 - !_TEST as uszie] = [];
    }
}
