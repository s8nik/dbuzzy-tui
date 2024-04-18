use crate::SmartString;

pub struct Clipboard {
    local: SmartString,
    global: Option<arboard::Clipboard>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Self {
        let local = SmartString::new_const();
        let global = arboard::Clipboard::new().ok();

        Self { local, global }
    }

    pub fn set_local(&mut self, text: String) {
        self.local = text.into();
    }

    pub fn get_local(&self) -> SmartString {
        self.local.to_owned()
    }

    pub fn set_global(&mut self, text: String) {
        if let Some(clipboard) = self.global.as_mut() {
            clipboard.set_text(text).ok();
        }
    }

    pub fn get_global(&mut self) -> SmartString {
        self.global
            .as_mut()
            .and_then(|x| x.get_text().ok())
            .unwrap_or_default()
            .into()
    }
}
