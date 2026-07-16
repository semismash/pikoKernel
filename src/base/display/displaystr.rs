use crate::base::text::sysstr::{self, SysStr};
use crate::drivers::display::{ForegroundColor, BackgroundColor};

use core::ops::{Deref, DerefMut};

// formatted string type for display
struct DisplayString<const N: usize = STR_LENGTH_DEFAULT> {
    content: SysStr,
    fg_color: ForegroundColor,
    bg_color: BackgroundColor,
    blink: bool,
}

// initialization
impl<const N: usize> DisplayString<N> {

    pub const fn new() -> Self {
        Self {
            content: SysStr::<N>::new(),
            fg_color: ForegroundColor::default(),
            bg_color: BackgroundColor::default(),
            blink: false,   // default
        }
    }

    pub fn from_str<FG, BG, BL>(
        str_in: &str,
        fg_color: FG,
        bg_color: BG,
        blink: BL,
    ) -> Option<Self> 
    where
        FG: Into<Option<ForegroundColor>> + Copy,
        BG: Into<Option<BackgroundColor>> + Copy,
        BL: Into<Option<bool>> + Copy,
    {
        let new_str = SysStr::<N>::from_str(str_in)?;
        Some(Self {
            content: new_str,
            fg_color: fg_color.into().unwrap_or(ForegroundColor::default()),
            bg_color: bg_color.into().unwrap_or(BackgroundColor::default()),
            blink: blink.into().unwrap_or(false),
        })
    }

    pub fn from_str_default(str_in: &str) -> Option<Self> {
        Self::<N>::from_str(str_in, None, None, None)
    }

}

//access
impl<const N: usize> Deref for DisplayString {
    type Target = SysStr;

    fn deref(&self) -> &Self::Target {
        &self.content
    }

}

//mutation
impl<const N: usize> DerefMut for DisplayString {
    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }

}

// display string specific
impl DisplayString {

    pub fn get_fg_color(&self) -> ForegroundColor { self.fg_color }
    pub fn get_bg_color(&self) -> BackgroundColorColor { self.bg_color }
    pub fn get_if_blink(&self) -> bool { self.blink }

    pub fn set_fg_color(&mut self, new_val: ForegroundColor) { self.fg_color = new_val; }
    pub fn set_bg_color(&mut self, new_val: BackgroundColorColor) { self.bg_color = new_val; }
    pub fn set_blink(&mut self, new_val: bool) { self.blink = new_val; }

    pub fn set_attributes<FG, BG, BL>(
        &mut self,
        new_fg: FG,
        new_bg: BG,
        new_blink: BL,
    )
    where
        FG: Into<Option<ForegroundColor>> + Copy,
        BG: Into<Option<BackgroundColor>> + Copy,
        BL: Into<Option<bool>> + Copy,
    {
        if let Some(fg) = new_fg.into() { self.fg_color = fg; }
        if let Some(bg) = new_bg.into() { self.bl_color = bg; }
        if let Some(bl) = new_blink.into() { self.blink = bl; }
    }

}