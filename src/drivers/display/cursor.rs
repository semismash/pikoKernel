pub struct CursorFn {
    enable: Option<fn(u8, u8)>,
    disable: Option<fn()>,
    update: Option<fn(usize, usize)>,
}

impl CursorFn {
    
    pub const fn define_cursor_fn(
        fn_en: Option<fn(u8, u8)>,
        fn_dis: Option<fn()>,
        fn_updt: Option<fn(usize, usize)>,
    ) -> Self {
        Self {
            enable: fn_en,
            disable: fn_dis,
            update: fn_updt,
        }
    }

}