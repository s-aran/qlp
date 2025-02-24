use crate::error::Error;

use windows::Win32::{
    Foundation::{HANDLE, HGLOBAL, HWND},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
            GetClipboardFormatNameW, IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
        },
        Ole::{CF_OEMTEXT, CF_UNICODETEXT, CLIPBOARD_FORMAT},
    },
};

pub struct WinClipboard {
    opened: bool,
    clipboard_format: CLIPBOARD_FORMAT,
}

impl WinClipboard {
    pub fn new(format: CLIPBOARD_FORMAT) -> Self {
        WinClipboard {
            opened: false,
            clipboard_format: format,
        }
    }

    pub fn new_with_unicode_text() -> Self {
        WinClipboard {
            opened: false,
            clipboard_format: CLIPBOARD_FORMAT(CF_UNICODETEXT.0),
        }
    }

    pub fn type_of(&self) -> bool {
        unsafe { IsClipboardFormatAvailable(self.clipboard_format.0.into()).is_ok() }
    }

    pub fn open(&mut self) -> Result<(), Error> {
        if self.opened {
            return Err(Error::new("Clipboard already opened"));
        }

        if unsafe { OpenClipboard(Some(HWND::default())).is_err() } {
            return Err(Error::new("Failed to open clipboard"));
        }

        self.opened = true;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Error> {
        if !self.opened {
            return Err(Error::new("Clipboard already closed"));
        }

        if unsafe { CloseClipboard().is_err() } {
            return Err(Error::new("Failed to close clipboard"));
        }

        self.opened = false;
        Ok(())
    }

    pub fn opened(&self) -> bool {
        self.opened
    }

    pub fn get_clipboard_data(&self) -> Result<HANDLE, Error> {
        if !self.opened {
            return Err(Error::new("Clipboard not opened"));
        }

        match unsafe { GetClipboardData(self.clipboard_format.0.into()) } {
            Ok(h) => Ok(h),
            Err(_) => return Err(Error::new("Failed to get clipboard data")),
        }
    }

    pub fn set_clipboard_data(&self, h_global: HGLOBAL) -> Result<(), Error> {
        if !self.opened {
            return Err(Error::new("Clipboard not opened"));
        }

        match unsafe { SetClipboardData(self.clipboard_format.0.into(), Some(HANDLE(h_global.0))) }
        {
            Ok(_) => Ok(()),
            Err(_) => return Err(Error::new("Failed to set clipboard data")),
        }
    }

    pub fn empty(&self) -> Result<(), Error> {
        if !self.opened {
            return Err(Error::new("Clipboard not opened"));
        }

        match unsafe { EmptyClipboard() } {
            Ok(_) => Ok(()),
            Err(_) => return Err(Error::new("Failed to empty clipboard")),
        }
    }

    pub fn enumerate(&self) -> Vec<CLIPBOARD_FORMAT> {
        if !self.opened {
            return Vec::new();
        }

        let mut clipboard_format_list = Vec::<CLIPBOARD_FORMAT>::new();

        unsafe {
            let mut available_clipboard_format = CLIPBOARD_FORMAT::default();
            loop {
                available_clipboard_format = CLIPBOARD_FORMAT(EnumClipboardFormats(
                    available_clipboard_format.0.into(),
                ) as u16);

                if available_clipboard_format == CLIPBOARD_FORMAT(0) {
                    break;
                }

                clipboard_format_list.push(available_clipboard_format);
            }
        }

        clipboard_format_list
    }

    pub fn resolve_clipboard_format_name(&self, cf: &CLIPBOARD_FORMAT) -> Option<String> {
        match cf {
            &CF_OEMTEXT => Some("CF_OEMTEXT".to_owned()),
            &CF_UNICODETEXT => Some("CF_UNICODETEXT".to_owned()),
            _ => {
                let mut lpsz_format_name = [0u16; 256];
                let name_length =
                    unsafe { GetClipboardFormatNameW(cf.0.into(), &mut lpsz_format_name) };
                if name_length > 0 {
                    Some(String::from_utf16_lossy(
                        &lpsz_format_name[..name_length as usize],
                    ))
                } else {
                    None
                }
            }
        }
    }
}

impl Drop for WinClipboard {
    fn drop(&mut self) {
        if self.opened {
            let _ = self.close();
        }
    }
}
