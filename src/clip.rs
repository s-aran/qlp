use crate::error::Error;

#[derive(Debug, Clone)]
pub enum ClipboardFormat {
    Text(String),
    Html(String),
}

impl ToString for ClipboardFormat {
    fn to_string(&self) -> String {
        match self {
            ClipboardFormat::Text(s) => s,
            ClipboardFormat::Html(s) => s,
        }
        .to_string()
    }
}

impl Default for ClipboardFormat {
    fn default() -> Self {
        ClipboardFormat::Text(String::new())
    }
}

pub struct Clipboard {}

pub trait Clip {
    fn new() -> Self;

    fn determine_format(&self) -> Result<ClipboardFormat, Error>;

    fn get_data(&mut self, format: &ClipboardFormat) -> Result<ClipboardFormat, Error>;
    fn set_data(&mut self, data: &ClipboardFormat) -> Result<(), Error>;

    fn get_html<T: ToString>(data: &T) -> String;
}

#[cfg(target_os = "linux")]
pub mod clipboard {
    use crate::error::Error;

    use super::{Clip, Clipboard, ClipboardFormat};

    impl Clip for Clipboard {
        fn new() -> Self {
            Self {}
        }

        fn get_data(&mut self, format: &ClipboardFormat) -> Result<ClipboardFormat, Error> {
            Ok(ClipboardFormat::Text("".to_owned()))
        }

        fn set_data(&mut self, data: &ClipboardFormat) -> Result<(), Error> {
            Ok(())
        }

        fn get_html<T>(data: &T) -> String {
            String::default()
        }

        fn determine_format(&self) -> Result<ClipboardFormat, Error> {
            Ok(ClipboardFormat::Text("".to_owned()))
        }
    }
}

#[cfg(target_os = "windows")]
pub mod clipboard {
    use regex::Regex;

    use crate::{error::Error, global_memory::GlobalMemory, win_clipboard::WinClipboard};

    use super::{Clip, Clipboard, ClipboardFormat};

    impl Clipboard {
        fn create_instance_by(format: &ClipboardFormat) -> WinClipboard {
            match format {
                ClipboardFormat::Text(_) => WinClipboard::new_with_unicode_text(),
                ClipboardFormat::Html(_) => WinClipboard::new_wth_html_text(),
            }
        }

        fn decode(data: *const u16, size: usize, format: &ClipboardFormat) -> String {
            let slice = unsafe { std::slice::from_raw_parts(data, size / 2) };
            match format {
                // reduce last \0
                ClipboardFormat::Text(_) => String::from_utf16(&slice[..slice.len() - 1]).unwrap(),
                ClipboardFormat::Html(_) => {
                    let mut utf8_vec: Vec<u8> = vec![];
                    slice.iter().for_each(|e| {
                        utf8_vec.push(*e as u8);
                        utf8_vec.push((e >> 8) as u8);
                    });
                    // reduce last \0
                    String::from_utf8_lossy(&utf8_vec[..&utf8_vec.len() - 1]).to_string()
                }
            }
        }

        // fn encode<T>(data: &ClipboardFormat) -> Vec<T>
        // where
        //     T: Sized,
        // {
        //     match data {
        //         ClipboardFormat::Text(s) => s.encode_utf16().collect(),
        //         ClipboardFormat::Html(s) => s.encode_utf16().collect(),
        //     }
        // }
    }

    impl Clip for Clipboard {
        fn new() -> Self {
            Self {}
        }

        fn get_data(&mut self, format: &ClipboardFormat) -> Result<ClipboardFormat, Error> {
            let mut instance = Clipboard::create_instance_by(&self.determine_format().unwrap());
            if !instance.type_of() {
                return Err(Error::new("Clipboard format not available"));
            }

            instance.open()?;
            let h_global = instance.get_clipboard_data()?;

            let mut mem = GlobalMemory::new();
            let data = match mem.lock_by_handle(h_global) {
                Ok(ptr) => ptr as *const u16,
                Err(e) => {
                    return Err(Error::new(format!(
                        "Failed to lock memory by handle: {}",
                        e.to_string()
                    )));
                }
            };

            let str_data = Clipboard::decode(data, mem.size(), format);

            Ok(match format {
                ClipboardFormat::Text(_) => ClipboardFormat::Text(str_data),
                ClipboardFormat::Html(_) => ClipboardFormat::Html(str_data),
            })
        }

        fn set_data(&mut self, data: &ClipboardFormat) -> Result<(), Error> {
            let src: Vec<u16> = match data {
                ClipboardFormat::Text(s) => s.encode_utf16().collect(),
                ClipboardFormat::Html(s) => s.encode_utf16().collect(),
            };

            let mut instance = Clipboard::create_instance_by(data);

            instance.open()?;
            instance.empty()?;

            let global_size = (src.len() + 1) * std::mem::size_of::<u16>();

            let mut mem = GlobalMemory::new();
            let ptr = match mem.alloc_without_free(global_size) {
                Ok(ptr) => ptr,
                Err(e) => {
                    return Err(Error::new(format!(
                        "Failed to allocate memory: {}",
                        e.to_string()
                    )));
                }
            };

            unsafe {
                std::ptr::copy(src.as_ptr(), ptr as *mut u16, src.len());
            }

            Ok(instance.set_clipboard_data(mem.get_global())?)
        }

        fn get_html<T>(data: &T) -> String
        where
            T: ToString,
        {
            const RE_VERSION_PATTERN: &'static str = r"^Version:([0-9\.]+)$";
            const RE_START_HTML_PATTERN: &'static str = r"^StartHTML:([0-9]+)$";
            const RE_END_HTML_PATTERN: &'static str = r"^EndHTML:([0-9]+)$";
            const RE_START_FRAGMENT_PATTERN: &'static str = r"^StartFragment:([0-9]+)$";
            const RE_END_FRAGMENT_PATTERN: &'static str = r"^EndFragment:([0-9]+)$";

            let re_version = Regex::new(RE_VERSION_PATTERN).unwrap();
            let re_start_html = Regex::new(RE_START_HTML_PATTERN).unwrap();
            let re_end_html = Regex::new(RE_END_HTML_PATTERN).unwrap();
            let re_start_fragment = Regex::new(RE_START_FRAGMENT_PATTERN).unwrap();
            let re_end_fragment = Regex::new(RE_END_FRAGMENT_PATTERN).unwrap();

            let mut matched_count = 0;
            let mut html_lines = vec![];

            for raw_line in data.to_string().lines() {
                let line = raw_line.trim();
                if line.is_empty() {
                    continue;
                }

                if re_version.is_match(line) {
                    matched_count += 1;
                    continue;
                }

                if re_start_html.is_match(line) {
                    matched_count += 1;
                    continue;
                }

                if re_end_html.is_match(line) {
                    matched_count += 1;
                    continue;
                }

                if re_start_fragment.is_match(line) {
                    matched_count += 1;
                    continue;
                }

                if re_end_fragment.is_match(line) {
                    matched_count += 1;
                    continue;
                }

                if matched_count < 5 {
                    continue;
                }

                html_lines.push(line.to_owned());
            }

            html_lines.join("\n")
        }

        fn determine_format(&self) -> Result<ClipboardFormat, Error> {
            let mut instance = WinClipboard::new_with_unicode_text();
            instance.open()?;

            let formats = instance.enumerate();
            let format_names: Vec<String> = formats
                .iter()
                .filter_map(|f| instance.resolve_clipboard_format_name(&f))
                .collect();

            // HTML
            if format_names.contains(&"HTML Format".to_string()) {
                return Ok(ClipboardFormat::Html("".to_string()));
            }

            // Plain text
            if format_names.contains(&"CF_UNICODETEXT".to_string())
                || format_names.contains(&"CF_OEMTEXT".to_string())
            {
                return Ok(ClipboardFormat::Text("".to_string()));
            }

            // unsupported
            Err(Error::new("Clipboard format not available"))
        }
    }
}
