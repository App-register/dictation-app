use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use dicto_core::{Config, Dictionary};

/// Full settings window — every config option exposed visually.
pub struct SettingsWindow {
    pub window: nwg::Window,

    // --- General ---
    lang_combo: nwg::ComboBox<String>,
    model_combo: nwg::ComboBox<String>,
    mic_combo: nwg::ComboBox<String>,

    // --- Hotkey ---
    ctrl_cb: nwg::CheckBox,
    alt_cb: nwg::CheckBox,
    shift_cb: nwg::CheckBox,
    win_cb: nwg::CheckBox,
    key_input: nwg::TextInput,
    mode_combo: nwg::ComboBox<String>,

    // --- Overlay ---
    ov_mode_combo: nwg::ComboBox<String>,
    ov_x_input: nwg::TextInput,
    ov_y_input: nwg::TextInput,
    ov_font_input: nwg::TextInput,

    // --- Formatting ---
    space_cb: nwg::CheckBox,
    num_cb: nwg::CheckBox,
    trail_cb: nwg::CheckBox,
    cap_input: nwg::TextInput,

    // --- Output ---
    paste_cb: nwg::CheckBox,

    // --- Dictionary ---
    dict_list: nwg::ListBox<String>,
    word_input: nwg::TextInput,
    repl_input: nwg::TextInput,
    add_btn: nwg::Button,
    del_btn: nwg::Button,

    // --- Buttons ---
    save_btn: nwg::Button,
    cancel_btn: nwg::Button,

    /// Labels must be kept alive (dropping an HWND wrapper destroys the window)
    labels: Vec<nwg::Label>,

    // --- State ---
    config: Arc<Mutex<Config>>,
    dict: Arc<Mutex<Dictionary>>,
    /// Source of truth for dictionary entries (synced with listbox)
    dict_entries: Rc<RefCell<Vec<(String, String)>>>,
}

impl SettingsWindow {
    pub fn new(config: Arc<Mutex<Config>>, dict: Arc<Mutex<Dictionary>>) -> Self {
        let initial: Vec<(String, String)> =
            dict.lock().unwrap().entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        Self {
            config,
            dict,
            dict_entries: Rc::new(RefCell::new(initial)),
            labels: Vec::new(),
            window: Default::default(),
            lang_combo: Default::default(),
            model_combo: Default::default(),
            mic_combo: Default::default(),
            ctrl_cb: Default::default(),
            alt_cb: Default::default(),
            shift_cb: Default::default(),
            win_cb: Default::default(),
            key_input: Default::default(),
            mode_combo: Default::default(),
            ov_mode_combo: Default::default(),
            ov_x_input: Default::default(),
            ov_y_input: Default::default(),
            ov_font_input: Default::default(),
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

    fn label(parent: &nwg::Window, text: &str, pos: (i32, i32)) -> Result<nwg::Label, nwg::NwgError> {
        let mut l = nwg::Label::default();
        nwg::Label::builder()
            .size((150, 22))
            .position(pos)
            .text(text)
            .parent(parent)
            .build(&mut l)?;
        Ok(l)
    }

    fn populate(&self) {
        let cfg = self.config.lock().unwrap();

        let li = match cfg.language.as_str() {
            "en" => 1,
            "fr" => 2,
            _ => 0,
        };
        self.lang_combo.set_selection(Some(li));

        let mi = if cfg.model_size == "base" { 1 } else { 0 };
        self.model_combo.set_selection(Some(mi));

        // Microphones are populated at build time; just keep current selection
        let mic_index = self.mic_combo.selection().unwrap_or(0);

        let mods: Vec<&str> = cfg.hotkey_modifiers.split(',').collect();
        self.ctrl_cb.set_check_state(state(mods.contains(&"ctrl")));
        self.alt_cb.set_check_state(state(mods.contains(&"alt")));
        self.shift_cb.set_check_state(state(mods.contains(&"shift")));
        self.win_cb.set_check_state(state(mods.contains(&"win")));
        self.key_input.set_text(&cfg.hotkey_key);

        let mm = if cfg.hotkey_mode == "toggle-to-talk" { 1 } else { 0 };
        self.mode_combo.set_selection(Some(mm));

        let om = match cfg.overlay_mode.as_str() {
            "always" => 1,
            "never" => 2,
            _ => 0,
        };
        self.ov_mode_combo.set_selection(Some(om));
        self.ov_x_input.set_text(&cfg.overlay_x.to_string());
        self.ov_y_input.set_text(&cfg.overlay_y.to_string());
        self.ov_font_input.set_text(&cfg.overlay_font_size.to_string());

        self.space_cb.set_check_state(state(cfg.space_before_punctuation));
        self.num_cb.set_check_state(state(cfg.text_to_number));
        self.trail_cb.set_check_state(state(cfg.trailing_space));
        self.cap_input.set_text(&cfg.auto_capitalize_threshold.to_string());

        self.paste_cb.set_check_state(state(cfg.auto_paste));

        // Dict list already populated at build (items pushed there first).
        // Selection index may have been reset by rebuild; restore.
        self.mic_combo.set_selection(Some(mic_index));
    }

    fn read_config(&self) -> Config {
        let mut cfg = Config::default();

        cfg.language = match self.lang_combo.selection() {
            Some(1) => "en".to_string(),
            Some(2) => "fr".to_string(),
            _ => "auto".to_string(),
        };
        cfg.model_size = match self.model_combo.selection() {
            Some(1) => "base".to_string(),
            _ => "tiny".to_string(),
        };
        // Mic: "[Default]" → empty string (use system default)
        let sel = self.mic_combo.selection_string().unwrap_or_default();
        if sel.starts_with("[Default]") {
            cfg.mic_name = "default".to_string();
        } else {
            cfg.mic_name = sel;
        }

        let mut mods = Vec::new();
        if self.ctrl_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("ctrl"); }
        if self.alt_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("alt"); }
        if self.shift_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("shift"); }
        if self.win_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("win"); }
        cfg.hotkey_modifiers = mods.join(",");

        cfg.hotkey_key = self.key_input.text().trim().to_string();
        cfg.hotkey_mode = match self.mode_combo.selection() {
            Some(1) => "toggle-to-talk".to_string(),
            _ => "push-to-talk".to_string(),
        };

        cfg.overlay_mode = match self.ov_mode_combo.selection() {
            Some(1) => "always".to_string(),
            Some(2) => "never".to_string(),
            _ => "recording".to_string(),
        };
        cfg.overlay_x = self.ov_x_input.text().trim().parse().unwrap_or(100);
        cfg.overlay_y = self.ov_y_input.text().trim().parse().unwrap_or(100);
        cfg.overlay_font_size = self.ov_font_input.text().trim().parse().unwrap_or(16);

        cfg.space_before_punctuation = self.space_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg.text_to_number = self.num_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg.trailing_space = self.trail_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg.auto_capitalize_threshold = self.cap_input.text().trim().parse().unwrap_or(3);

        cfg.auto_paste = self.paste_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg
    }

    fn sync_dict_entries(&self) {
        let entries = self.dict_entries.borrow();
        let mut dict = self.dict.lock().unwrap();
        dict.entries = entries.clone().into_iter().collect();
        let _ = dict.save();
    }

    fn add_dict_entry(&self) {
        let word = self.word_input.text().trim().to_string();
        let repl = self.repl_input.text().trim().to_string();
        if !word.is_empty() && !repl.is_empty() {
            self.dict_entries.borrow_mut().push((word.clone(), repl.clone()));
            let _ = self.dict_list.push(format!("{} -> {}", word, repl));
            self.word_input.set_text("");
            self.repl_input.set_text("");
        }
    }

    fn delete_dict_entry(&self) {
        if let Some(idx) = self.dict_list.selection() {
            self.dict_list.remove(idx);
            self.dict_entries.borrow_mut().remove(idx);
        }
    }
}

fn state(b: bool) -> nwg::CheckBoxState {
    if b { nwg::CheckBoxState::Checked } else { nwg::CheckBoxState::Unchecked }
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
                .position((300, 40))
                .title("Dicto — Settings")
                .build(&mut data.window)?;

            // ══════ GENERAL ════════
            data.labels.push(Self::label(&data.window, "Language:", (15, 18))?);
            nwg::ComboBox::builder()
                .size((110, 160)).position((90, 16))
                .parent(&data.window)
                .collection(vec!["auto".to_string(), "en".to_string(), "fr".to_string()])
                .selected_index(Some(0))
                .build(&mut data.lang_combo)?;

            data.labels.push(Self::label(&data.window, "Model:", (215, 18))?);
            nwg::ComboBox::builder()
                .size((100, 120)).position((260, 16))
                .parent(&data.window)
                .collection(vec!["tiny".to_string(), "base".to_string()])
                .selected_index(Some(0))
                .build(&mut data.model_combo)?;

            data.labels.push(Self::label(&data.window, "Microphone:", (15, 48))?);
            let devices = dicto_core::audio::AudioCapture::list_devices();
            let mut mic_items = vec!["[Default]".to_string()];
            for (_id, name) in &devices {
                mic_items.push(name.clone());
            }
            nwg::ComboBox::builder()
                .size((300, 200)).position((90, 46))
                .parent(&data.window)
                .collection(mic_items)
                .selected_index(Some(0))
                .build(&mut data.mic_combo)?;

            // ════════ HOTKEY ════════
            data.labels.push(Self::label(&data.window, "── Hotkey ──────────────", (15, 78))?);
            let mut ctrl = nwg::CheckBox::default();
            nwg::CheckBox::builder()
                .size((50, 22)).position((15, 100)).text("Ctrl").parent(&data.window)
                .build(&mut ctrl)?;
            data.ctrl_cb = ctrl;
            nwg::CheckBox::builder()
                .size((50, 22)).position((70, 100)).text("Alt").parent(&data.window)
                .build(&mut data.alt_cb)?;
            nwg::CheckBox::builder()
                .size((60, 22)).position((125, 100)).text("Shift").parent(&data.window)
                .build(&mut data.shift_cb)?;
            nwg::CheckBox::builder()
                .size((50, 22)).position((190, 100)).text("Win").parent(&data.window)
                .build(&mut data.win_cb)?;

            data.labels.push(Self::label(&data.window, "Key:", (15, 130))?);
            nwg::TextInput::builder()
                .size((80, 24)).position((50, 128)).text("F4").parent(&data.window)
                .build(&mut data.key_input)?;

            data.labels.push(Self::label(&data.window, "Mode:", (150, 130))?);
            nwg::ComboBox::builder()
                .size((140, 120)).position((195, 128))
                .parent(&data.window)
                .collection(vec!["push-to-talk".to_string(), "toggle-to-talk".to_string()])
                .selected_index(Some(0))
                .build(&mut data.mode_combo)?;

            // ════════ OVERLAY ════════
            data.labels.push(Self::label(&data.window, "── Overlay ──────────────", (15, 160))?);
            nwg::ComboBox::builder()
                .size((100, 120)).position((15, 182))
                .parent(&data.window)
                .collection(vec!["recording".to_string(), "always".to_string(), "never".to_string()])
                .selected_index(Some(0))
                .build(&mut data.ov_mode_combo)?;

            data.labels.push(Self::label(&data.window, "X:", (130, 184))?);
            nwg::TextInput::builder()
                .flags(nwg::TextInputFlags::NUMBER)
                .size((50, 24)).position((148, 182)).text("100").parent(&data.window)
                .build(&mut data.ov_x_input)?;
            data.labels.push(Self::label(&data.window, "Y:", (210, 184))?);
            nwg::TextInput::builder()
                .flags(nwg::TextInputFlags::NUMBER)
                .size((50, 24)).position((228, 182)).text("100").parent(&data.window)
                .build(&mut data.ov_y_input)?;
            data.labels.push(Self::label(&data.window, "Font:", (290, 184))?);
            nwg::TextInput::builder()
                .flags(nwg::TextInputFlags::NUMBER)
                .size((40, 24)).position((328, 182)).text("16").parent(&data.window)
                .build(&mut data.ov_font_input)?;

            // ════════ FORMATTING ════════
            data.labels.push(Self::label(&data.window, "── Formatting Rules ─────", (15, 214))?);
            nwg::CheckBox::builder()
                .size((240, 22)).position((15, 236)).text("Space before ? and ! (French)")
                .parent(&data.window).build(&mut data.space_cb)?;
            nwg::CheckBox::builder()
                .size((180, 22)).position((280, 236)).text("Text-to-number")
                .parent(&data.window).build(&mut data.num_cb)?;
            nwg::CheckBox::builder()
                .size((130, 22)).position((15, 260)).text("Trailing space")
                .parent(&data.window).build(&mut data.trail_cb)?;
            data.labels.push(Self::label(&data.window, "Auto-cap threshold (words):", (160, 262))?);
            nwg::TextInput::builder()
                .flags(nwg::TextInputFlags::NUMBER)
                .size((50, 24)).position((345, 260)).text("3").parent(&data.window)
                .build(&mut data.cap_input)?;

            // ════════ OUTPUT ════════
            data.labels.push(Self::label(&data.window, "── Output ───────────────", (15, 292))?);
            nwg::CheckBox::builder()
                .size((240, 22)).position((15, 314)).text("Auto-paste to active window")
                .parent(&data.window).build(&mut data.paste_cb)?;

            // ════════ DICTIONARY ════════
            data.labels.push(Self::label(&data.window, "── Dictionary ───────────", (15, 344))?);
            nwg::ListBox::builder()
                .size((280, 200)).position((15, 366))
                .parent(&data.window)
                .build(&mut data.dict_list)?;
            nwg::TextInput::builder()
                .size((140, 24)).position((310, 366)).text("Word").parent(&data.window)
                .build(&mut data.word_input)?;
            nwg::TextInput::builder()
                .size((140, 24)).position((310, 396)).text("Replacement").parent(&data.window)
                .build(&mut data.repl_input)?;
            nwg::Button::builder()
                .size((90, 28)).position((310, 431)).text("Add").parent(&data.window)
                .build(&mut data.add_btn)?;
            nwg::Button::builder()
                .size((90, 28)).position((410, 431)).text("Delete").parent(&data.window)
                .build(&mut data.del_btn)?;

            // ════════ BOTTOM ════════
            nwg::Button::builder()
                .size((120, 30)).position((140, 600)).text("Save & Close").parent(&data.window)
                .build(&mut data.save_btn)?;
            nwg::Button::builder()
                .size((120, 30)).position((280, 600)).text("Cancel").parent(&data.window)
                .build(&mut data.cancel_btn)?;

            // Populate listbox from dictionary
            let entries: Vec<(String, String)> = data.dict_entries.borrow().clone();
            for (w, r) in entries {
                let _ = data.dict_list.push(format!("{} -> {}", w, r));
            }

            data.populate();

            let ui = SettingsWindowUi {
                inner: Rc::new(data),
                default_handler: RefCell::new(None),
            };

            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose if &handle == &ui.window => {
                            nwg::stop_thread_dispatch();
                        }
                        E::OnButtonClick => {
                            if &handle == &ui.save_btn {
                                let cfg = ui.read_config();
                                {
                                    let mut c = ui.config.lock().unwrap();
                                    *c = cfg.clone();
                                }
                                let _ = cfg.save();
                                ui.sync_dict_entries();
                                nwg::stop_thread_dispatch();
                            } else if &handle == &ui.cancel_btn {
                                nwg::stop_thread_dispatch();
                            } else if &handle == &ui.add_btn {
                                ui.add_dict_entry();
                            } else if &handle == &ui.del_btn {
                                ui.delete_dict_entry();
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