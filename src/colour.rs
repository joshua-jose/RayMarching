#[macro_export]
macro_rules! rgb {
    [$r:expr, $g:expr, $b:expr] => {
        Vec3::new(
             ($r as f32 / 255.0) * ($r as f32 / 255.0),
             ($g as f32 / 255.0) * ($g as f32 / 255.0),
             ($b as f32 / 255.0) * ($b as f32 / 255.0),
        )
    };
}
