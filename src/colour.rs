#[macro_export]
macro_rules! rgb {
    [$r:expr, $g:expr, $b:expr] => {
        [
            ($r as f32 / 255.0).powi(2),
            ($g as f32 / 255.0).powi(2),
            ($b as f32 / 255.0).powi(2),
        ]
    };
}
