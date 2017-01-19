#[macro_export]
macro_rules! variadic2{
    ($f:path, $e1:expr) => {
        $e1
    };
    ($f:path, $e1:expr, $e2:expr) => {
        $f($e1, $e2)
    };
    ($f:path, $e1:expr, $e2:expr, $($rest:expr),*) => {
        $f(variadic2!($f, $e1, $e2), variadic2!($f, $($rest),*))
    };
}
