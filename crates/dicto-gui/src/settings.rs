use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use dicto_core::{Config, Dictionary};

pub struct SettingsWindow {
    pub window: nwg::Window,
    pub lang_combo: nwg::ComboBox,
    pub model_combo: nwg::ComboBox,
    pub ctrl_cb: nwg::CheckBox,
    pub alt_cb: nwg::CheckBox,
    pub shift_cb: nwg::CheckBox,
    pub win_cb: nwg::CheckBox,
    pub key_input: nwg::TextInput,
    pub mode_combo: nwg::ComboBox,
    pub ov_mode_combo: nwg::ComboBox,
    pub ov_x: nwg::TextInput,
    pub ov_y: nwg::TextInput,
    pub ov_font: nwg::TextInput,
    pub space_cb: nwg::CheckBox,
    pub num_cb: nwg::CheckBox,
    pub trail_cb: nwg::CheckBox,
    pub cap_input: nwg::TextInput,
    pub paste_cb: nwg::CheckBox,
    pub dict_list: nwg::ListBox,
    pub word_input: nwg::TextInput,
    pub repl_input: nwg::TextInput,
    pub add_btn: nwg::Button,
    pub del_btn: nwg::Button,
    pub save_btn: nwg::Button,
    pub cancel_btn: nwg::Button,
    config: Arc<Mutex<Config>>,
    dict: Arc<Mutex<Dictionary>>,
}

impl SettingsWindow {
    pub fn new(config: Arc<Mutex<Config>>, dict: Arc<Mutex<Dictionary>>) -> Self {
        Self {
            config, dict,
            window: Default::default(),
            lang_combo: Default::default(),
            model_combo: Default::default(),
            ctrl_cb: Default::default(),
            alt_cb: Default::default(),
            shift_cb: Default::default(),
            win_cb: Default::default(),
            key_input: Default::default(),
            mode_combo: Default::default(),
            ov_mode_combo: Default::default(),
            ov_x: Default::default(),
            ov_y: Default::default(),
            ov_font: Default::default(),
            space_cb: Default::default(),
            num_cb: Default::default(),
            trail_cb: Default::default(),
            cap_input: Default::default(),
            paste_cb: Default::default(),
            dict_list: Default::default(),
            word_input: Default::default(),
            repl_input: Default::default(),
            add_btn: Default::default(),
            del_btn: Default::default(),
            save_btn: Default::default(),
            cancel_btn: Default::default(),
        }
    }

    fn read_controls(&self) -> Config {
        let mut cfg = Config::default();
        if let Some(t) = self.lang_combo.text() { cfg.language = t.trim().to_string(); }
        if let Some(t) = self.model_combo.text() { cfg.model_size = t.trim().to_string(); }
        let mut mods = Vec::new();
        if self.ctrl_cb.is_checked() { mods.push("ctrl"); }
        if self.alt_cb.is_checked() { mods.push("alt"); }
        if self.shift_cb.is_checked() { mods.push("shift"); }
        if self.win_cb.is_checked() { mods.push("win"); }
        cfg.hotkey_modifiers = mods.join(",");
        cfg.hotkey_key = self.key_input.text().unwrap_or_default().trim().to_string();
        if let Some(t) = self.mode_combo.text() { cfg.hotkey_mode = t.trim().to_string(); }
        if let Some(t) = self.ov_mode_combo.text() { cfg.overlay_mode = t.trim().to_string(); }
        cfg.overlay_x = self.ov_x.text().unwrap_or_default().trim().parse().unwrap_or(100);
        cfg.overlay_y = self.ov_y.text().unwrap_or_default().trim().parse().unwrap_or(100);
        cfg.overlay_font_size = self.ov_font.text().unwrap_or_default().trim().parse().unwrap_or(16);
        cfg.space_before_punctuation = self.space_cb.is_checked();
        cfg.text_to_number = self.num_cb.is_checked();
        cfg.trailing_space = self.trail_cb.is_checked();
        cfg.auto_capitalize_threshold = self.cap_input.text().unwrap_or_default().trim().parse().unwrap_or(3);
        cfg.auto_paste = self.paste_cb.is_checked();
        cfg
    }
}

mod settings_ui {
    use super::*;

    pub struct SettingsWindowUi {
        pub inner: Rc<SettingsWindow>,
        pub default_handler: RefCell<Option<nwg::EventHandler>>,
    }

    impl NativeUi<SettingsWindowUi> for SettingsWindow {
        fn build_ui(mut data: SettingsWindow) -> Result<SettingsWindowUi, nwg::NwgError> {
            use nwg::Event as E;

            nwg::Window::builder()
                .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
                .size((540, 680))
                .position((300, 50))
                .title("Dicto - Settings")
                .build(&mut data.window)?;

            // Language
            let mut lang_label = nwg::Label::default();
            nwg::Label::builder()
                .size((70, 22)).position((15, 18))
                .text("Language:").parent(&data.window)
                .build(&mut lang_label)?;
            nwg::ComboBox::builder()
                .size((120, 24)).position((90, 16))
                .parent(&data.window)
                .collection(vec!["auto", "en", "fr"])
                .build(&mut data.lang_combo)?;

            // Model
            let mut model_label = nwg::Label::default();
            nwg::Label::builder()
                .size((50, 22)).position((230, 18))
                .text("Model:").parent(&data.window)
                .build(&mut model_label)?;
            nwg::ComboBox::builder()
                .size((120, 24)).position((280, 16))
                .parent(&data.window)
                .collection(vec!["tiny", "base"])
                .build(&mut data.model_combo)?;

            // Hotkey
            let mut hk_title = nwg::Label::default();
            nwg::Label::builder()
                .size((520, 20)).position((10, 50))
                .text("--- Hotkey ---").parent(&data.window)
                .build(&mut hk_title)?;
            nwg::CheckBox::builder()
                .size((50, 22)).position((15, 72))
                .text("Ctrl").parent(&data.window)
                .build(&mut data.ctrl_cb)?;
            nwg::CheckBox::builder()
                .size((50, 22)).position((70, 72))
                .text("Alt").parent(&data.window)
                .build(&mut data.alt_cb)?;
            nwg::CheckBox::builder()
                .size((60, 22)).position((125, 72))
                .text("Shift").parent(&data.window)
                .build(&mut data.shift_cb)?;
            nwg::CheckBox::builder()
                .size((50, 22)).position((190, 72))
                .text("Win").parent(&data.window)
                .build(&mut data.win_cb)?;

            let mut key_label = nwg::Label::default();
            nwg::Label::builder()
                .size((35, 22)).position((15, 100))
                .text("Key:").parent(&data.window)
                .build(&mut key_label)?;
            nwg::TextInput::builder()
                .size((80, 24)).position((50, 98))
                .text("F4").parent(&data.window)
                .build(&mut data.key_input)?;

            let mut mode_label = nwg::Label::default();
            nwg::Label::builder()
                .size((45, 22)).position((150, 100))
                .text("Mode:").parent(&data.window)
                .build(&mut mode_label)?;
            nwg::ComboBox::builder()
                .size((140, 24)).position((195, 98))
                .parent(&data.window)
                .collection(vec!["push-to-talk", "toggle-to-talk"])
                .build(&mut data.mode_combo)?;

            // Overlay
            let mut ov_title = nwg::Label::default();
            nwg::Label::builder()
                .size((520, 20)).position((10, 130))
                .text("--- Overlay ---").parent(&data.window)
                .build(&mut ov_title)?;
            nwg::ComboBox::builder()
                .size((100, 24)).position((15, 152))
                .parent(&data.window)
                .collection(vec!["recording", "always", "never"])
                .build(&mut data.ov_mode_combo)?;

            let mut x_lbl = nwg::Label::default();
            nwg::Label::builder()
                .size((20, 22)).position((130, 154))
                .text("X:").parent(&data.window)
                .build(&mut x_lbl)?;
            nwg::TextInput::builder()
                .size((50, 24)).position((148, 152))
                .text("100").parent(&data.window)
                .build(&mut data.ov_x)?;

            let mut y_lbl = nwg::Label::default();
            nwg::Label::builder()
                .size((20, 22)).position((210, 154))
                .text("Y:").parent(&data.window)
                .build(&mut y_lbl)?;
            nwg::TextInput::builder()
                .size((50, 24)).position((228, 152))
                .text("100").parent(&data.window)
                .build(&mut data.ov_y)?;

            let mut f_lbl = nwg::Label::default();
            nwg::Label::builder()
                .size((40, 22)).position((290, 154))
                .text("Font:").parent(&data.window)
                .build(&mut f_lbl)?;
            nwg::TextInput::builder()
                .size((40, 24)).position((328, 152))
                .text("16").parent(&data.window)
                .build(&mut data.ov_font)?;

            // Formatting
            let mut fmt_title = nwg::Label::default();
            nwg::Label::builder()
                .size((520, 20)).position((10, 184))
                .text("--- Formatting Rules ---").parent(&data.window)
                .build(&mut fmt_title)?;
            nwg::CheckBox::builder()
                .size((260, 22)).position((15, 206))
                .text("Space before ? and ! (French)").parent(&data.window)
                .build(&mut data.space_cb)?;
            nwg::CheckBox::builder()
                .size((200, 22)).position((280, 206))
                .text("Text-to-number").parent(&data.window)
                .build(&mut data.num_cb)?;
            nwg::CheckBox::builder()
                .size((130, 22)).position((15, 230))
                .text("Trailing space").parent(&data.window)
                .build(&mut data.trail_cb)?;

            let mut cap_lbl = nwg::Label::default();
            nwg::Label::builder()
                .size((180, 22)).position((160, 232))
                .text("Auto-cap threshold (words):").parent(&data.window)
                .build(&mut cap_lbl)?;
            nwg::TextInput::builder()
                .size((50, 24)).position((345, 230))
                .text("3").parent(&data.window)
                .build(&mut data.cap_input)?;

            // Output
            let mut out_title = nwg::Label::default();
            nwg::Label::builder()
                .size((520, 20)).position((10, 260))
                .text("--- Output ---").parent(&data.window)
                .build(&mut out_title)?;
            nwg::CheckBox::builder()
                .size((220, 22)).position((15, 280))
                .text("Auto-paste to active window").parent(&data.window)
                .build(&mut data.paste_cb)?;

            // Dictionary
            let mut dict_title = nwg::Label::default();
            nwg::Label::builder()
                .size((520, 20)).position((10, 310))
                .text("--- Dictionary ---").parent(&data.window)
                .build(&mut dict_title)?;
            nwg::ListBox::builder()
                .size((280, 200)).position((15, 332))
                .parent(&data.window)
                .build(&mut data.dict_list)?;
            nwg::TextInput::builder()
                .size((140, 24)).position((310, 332))
                .text("Word").parent(&data.window)
                .build(&mut data.word_input)?;
            nwg::TextInput::builder()
                .size((140, 24)).position((310, 362))
                .text("Replacement").parent(&data.window)
                .build(&mut data.repl_input)?;
            nwg::Button::builder()
                .size((90, 28)).position((310, 397))
                .text("Add").parent(&data.window)
                .build(&mut data.add_btn)?;
            nwg::Button::builder()
                .size((90, 28)).position((410, 397))
                .text("Delete").parent(&data.window)
                .build(&mut data.del_btn)?;

            // Save & Cancel
            nwg::Button::builder()
                .size((120, 30)).position((140, 600))
                .text("Save & Close").parent(&data.window)
                .build(&mut data.save_btn)?;
            nwg::Button::builder()
                .size((120, 30)).position((280, 600))
                .text("Cancel").parent(&data.window)
                .build(&mut data.cancel_btn)?;

            let ui = SettingsWindowUi {
                inner: Rc::new(data),
                default_handler: RefCell::new(None),
            };

            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt: nwg::Event, _evt_data: Option<&nwg::EventData>, handle: nwg::ControlHandle| {
                if let Some(ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose if handle == ui.window.handle => {
                            nwg::stop_thread_dispatch();
                        }
                        E::OnButtonClick => {
                            if handle == ui.save_btn.handle {
                                let cfg = ui.read_controls();
                                let _ = cfg.save();
                                let mut dict = Dictionary::default();
                                let count = ui.dict_list.items_count();
                                for i in 0..count {
                                    if let Some(txt) = ui.dict_list.text_at(i) {
                                        if let Some(pos) = txt.find(" -> ") {
                                            dict.entries.insert(txt[..pos].to_string(), txt[pos+3..].to_string());
                                        }
                                    }
                                }
                                {
                                    let mut d = ui.dict.lock().unwrap();
                                    *d = dict;
                                    let _ = d.save();
                                }
                                nwg::stop_thread_dispatch();
                            } else if handle == ui.cancel_btn.handle {
                                nwg::stop_thread_dispatch();
                            } else if handle == ui.add_btn.handle {
                                let word = ui.word_input.text().unwrap_or_default();
                                let repl = ui.repl_input.text().unwrap_or_default();
                                if !word.is_empty() && !repl.is_empty() {
                                    let _ = ui.dict_list.add(format!("{} -> {}", word, repl));
                                    ui.word_input.set_text("");
                                    ui.repl_input.set_text("");
                                }
                            } else if handle == ui.del_btn.handle {
                                if let Some(idx) = ui.dict_list.selection() {
                                    let _ = ui.dict_list.remove_at(idx);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            };

            *ui.default_handler.borrow_mut() = Some(
                nwg::full_bind_event_handler(&ui.window.handle, handle_events)
            );

            Ok(ui)
        }
    }

    impl Deref for SettingsWindowUi {
        type Target = SettingsWindow;
        fn deref(&self) -> &SettingsWindow { &self.inner }
    }
}