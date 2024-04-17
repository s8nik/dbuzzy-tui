use crate::SmartString;

pub struct Clipboard {
    local: SmartString,
    global: arboard::Clipboard,
}

impl Clipboard {
    pub fn set_local(&mut self, text: String) {
        self.local = text.into();
    }

    pub fn get_local(&self) -> &str {
        &self.local
    }

    pub fn set_global(&mut self, text: String) {
        let _ = self.global.set_text(text);
    }

    pub fn get_global(&mut self) -> SmartString {
        self.global.get_text().unwrap_or_default().into()
    }
}
