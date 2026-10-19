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

    // Frames (section containers)
    _general: nwg::Frame,
    _hotkey: nwg::Frame,
    _overlay: nwg::Frame,
    _formatting: nwg::Frame,
    _output: nwg::Frame,
    _dictionary: nwg::Frame,

    /// Keep labels alive (dropping overwrites destroys HWND, causes crash)
    labels: Vec<nwg::Label>,

    // General
    lang_combo: nwg::ComboBox<String>,
    model_combo: nwg::ComboBox<String>,
    mic_combo: nwg::ComboBox<String>,

    // Hotkey
    ctrl_cb: nwg::CheckBox,
    alt_cb: nwg::CheckBox,
    shift_cb: nwg::CheckBox,
    win_cb: nwg::CheckBox,
    key_input: nwg::TextInput,
    capture_btn: nwg::Button,
    mode_combo: nwg::ComboBox<String>,
    cap_state: Rc<RefCell<bool>>,

    // Overlay
    ov_mode_combo: nwg::ComboBox<String>,
    ov_x: nwg::TextInput,
    ov_y: nwg::TextInput,
    ov_font: nwg::TextInput,

    // Formatting
    space_cb: nwg::CheckBox,
    num_cb: nwg::CheckBox,
    trail_cb: nwg::CheckBox,
    cap_thresh: nwg::TextInput,

    // Output
    paste_cb: nwg::CheckBox,

    // Dictionary
    dict_list: nwg::ListBox<String>,
    word_input: nwg::TextInput,
    repl_input: nwg::TextInput,
    add_btn: nwg::Button,
    del_btn: nwg::Button,
    dict_entries: Rc<RefCell<Vec<(String, String)>>>,

    // Buttons
    save_btn: nwg::Button,
    cancel_btn: nwg::Button,

    // State
    config: Arc<Mutex<Config>>,
    dict: Arc<Mutex<Dictionary>>,
}

impl SettingsWindow {
    pub fn new(config: Arc<Mutex<Config>>, dict: Arc<Mutex<Dictionary>>) -> Self {
        let entries: Vec<(String, String)> =
            dict.lock().unwrap().entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        Self {
            config, dict,
            dict_entries: Rc::new(RefCell::new(entries)),
            cap_state: Rc::new(RefCell::new(false)),
            labels: Vec::new(),
            window: Default::default(),
            _general: Default::default(), _hotkey: Default::default(),
            _overlay: Default::default(), _formatting: Default::default(),
            _output: Default::default(), _dictionary: Default::default(),
            lang_combo: Default::default(), model_combo: Default::default(),
            mic_combo: Default::default(),
            ctrl_cb: Default::default(), alt_cb: Default::default(),
            shift_cb: Default::default(), win_cb: Default::default(),
            key_input: Default::default(), capture_btn: Default::default(),
            mode_combo: Default::default(),
            ov_mode_combo: Default::default(), ov_x: Default::default(),
            ov_y: Default::default(), ov_font: Default::default(),
            space_cb: Default::default(), num_cb: Default::default(),
            trail_cb: Default::default(), cap_thresh: Default::default(),
            paste_cb: Default::default(),
            dict_list: Default::default(), word_input: Default::default(),
            repl_input: Default::default(), add_btn: Default::default(),
            del_btn: Default::default(), save_btn: Default::default(),
            cancel_btn: Default::default(),
        }
    }

    fn populate(&self) {
        let cfg = self.config.lock().unwrap();
        let li = match cfg.language.as_str() { "en" => 1, "fr" => 2, _ => 0 };
        self.lang_combo.set_selection(Some(li));
        let mi = if cfg.model_size == "base" { 1 } else { 0 };
        self.model_combo.set_selection(Some(mi));
        let mods: Vec<&str> = cfg.hotkey_modifiers.split(',').collect();
        self.ctrl_cb.set_check_state(state(mods.contains(&"ctrl")));
        self.alt_cb.set_check_state(state(mods.contains(&"alt")));
        self.shift_cb.set_check_state(state(mods.contains(&"shift")));
        self.win_cb.set_check_state(state(mods.contains(&"win")));
        self.key_input.set_text(&cfg.hotkey_key);
        let mm = if cfg.hotkey_mode == "toggle-to-talk" { 1 } else { 0 };
        self.mode_combo.set_selection(Some(mm));
        let om = match cfg.overlay_mode.as_str() { "always" => 1, "never" => 2, _ => 0 };
        self.ov_mode_combo.set_selection(Some(om));
        self.ov_x.set_text(&cfg.overlay_x.to_string());
        self.ov_y.set_text(&cfg.overlay_y.to_string());
        self.ov_font.set_text(&cfg.overlay_font_size.to_string());
        self.space_cb.set_check_state(state(cfg.space_before_punctuation));
        self.num_cb.set_check_state(state(cfg.text_to_number));
        self.trail_cb.set_check_state(state(cfg.trailing_space));
        self.cap_thresh.set_text(&cfg.auto_capitalize_threshold.to_string());
        self.paste_cb.set_check_state(state(cfg.auto_paste));
    }

    fn read_config(&self) -> Config {
        let mut cfg = Config::default();
        cfg.language = match self.lang_combo.selection() { Some(1) => "en", Some(2) => "fr", _ => "auto" }.into();
        cfg.model_size = match self.model_combo.selection() { Some(1) => "base", _ => "tiny" }.into();
        let sel = self.mic_combo.selection_string().unwrap_or_default();
        cfg.mic_name = if sel.starts_with("[Default]") { "default".into() } else { sel };
        let mut mods = Vec::new();
        if self.ctrl_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("ctrl") }
        if self.alt_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("alt") }
        if self.shift_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("shift") }
        if self.win_cb.check_state() == nwg::CheckBoxState::Checked { mods.push("win") }
        cfg.hotkey_modifiers = mods.join(",");
        cfg.hotkey_key = self.key_input.text().trim().to_string();
        cfg.hotkey_mode = match self.mode_combo.selection() { Some(1) => "toggle-to-talk", _ => "push-to-talk" }.into();
        cfg.overlay_mode = match self.ov_mode_combo.selection() { Some(1) => "always", Some(2) => "never", _ => "recording" }.into();
        cfg.overlay_x = self.ov_x.text().trim().parse().unwrap_or(100);
        cfg.overlay_y = self.ov_y.text().trim().parse().unwrap_or(100);
        cfg.overlay_font_size = self.ov_font.text().trim().parse().unwrap_or(16);
        cfg.space_before_punctuation = self.space_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg.text_to_number = self.num_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg.trailing_space = self.trail_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg.auto_capitalize_threshold = self.cap_thresh.text().trim().parse().unwrap_or(3);
        cfg.auto_paste = self.paste_cb.check_state() == nwg::CheckBoxState::Checked;
        cfg
    }

    fn start_capture(&self) {
        *self.cap_state.borrow_mut() = true;
        self.capture_btn.set_text("Press a key…");
        self.window.set_focus();
    }
}

fn state(b: bool) -> nwg::CheckBoxState { if b { nwg::CheckBoxState::Checked } else { nwg::CheckBoxState::Unchecked } }

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
                .size((520, 700))
                .position((300, 40))
                .title("Dicto — Settings")
                .build(&mut data.window)?;

            // === GENERAL ===
            nwg::Frame::builder()
                .size((490, 80)).position((15, 10)).parent(&data.window).build(&mut data._general)?;
            let mut lbl = nwg::Label::default();
            nwg::Label::builder().size((100,16)).position((22,13)).text("General").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::Label::builder().size((60,20)).position((25,30)).text("Language:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::ComboBox::builder().size((110,24)).position((90,28)).parent(&data.window)
                .collection(vec!["auto".into(),"en".into(),"fr".into()]).build(&mut data.lang_combo)?;

            nwg::Label::builder().size((45,20)).position((215,30)).text("Model:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::ComboBox::builder().size((100,24)).position((260,28)).parent(&data.window)
                .collection(vec!["tiny".into(),"base".into()]).build(&mut data.model_combo)?;

            nwg::Label::builder().size((70,20)).position((25,55)).text("Microphone:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            let devs = dicto_core::audio::AudioCapture::list_devices();
            let mut items = vec!["[Default]".into()];
            for (_, n) in &devs { items.push(n.clone()); }
            nwg::ComboBox::builder().size((280,24)).position((100,53)).parent(&data.window)
                .collection(items).build(&mut data.mic_combo)?;

            // === HOTKEY ===
            nwg::Frame::builder()
                .size((490, 100)).position((15, 95))
                .parent(&data.window).build(&mut data._hotkey)?;
            nwg::Label::builder().size((100,16)).position((22,12)).text("Hotkey").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::Label::builder().size((60,20)).position((25,115)).text("Modifiers:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::CheckBox::builder().size((50,22)).position((90,113)).text("Ctrl").parent(&data.window).build(&mut data.ctrl_cb)?;
            nwg::CheckBox::builder().size((50,22)).position((145,113)).text("Alt").parent(&data.window).build(&mut data.alt_cb)?;
            nwg::CheckBox::builder().size((60,22)).position((200,113)).text("Shift").parent(&data.window).build(&mut data.shift_cb)?;
            nwg::CheckBox::builder().size((50,22)).position((265,113)).text("Win").parent(&data.window).build(&mut data.win_cb)?;

            nwg::Label::builder().size((25,20)).position((25,140)).text("Key:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::TextInput::builder().size((60,24)).position((55,138)).text("F4").parent(&data.window).build(&mut data.key_input)?;
            nwg::Button::builder().size((80,26)).position((120,138)).text("Capture").parent(&data.window).build(&mut data.capture_btn)?;

            nwg::Label::builder().size((40,20)).position((215,140)).text("Mode:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::ComboBox::builder().size((130,24)).position((260,138)).parent(&data.window)
                .collection(vec!["push-to-talk".into(),"toggle-to-talk".into()]).build(&mut data.mode_combo)?;

            // === OVERLAY ===
            nwg::Frame::builder()
                .size((490, 70)).position((15, 200))
                .parent(&data.window).build(&mut data._overlay)?;
            nwg::Label::builder().size((100,16)).position((22,12)).text("Overlay").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::Label::builder().size((35,20)).position((25,220)).text("Show:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::ComboBox::builder().size((100,24)).position((65,218)).parent(&data.window)
                .collection(vec!["recording".into(),"always".into(),"never".into()]).build(&mut data.ov_mode_combo)?;

            nwg::Label::builder().size((20,20)).position((180,220)).text("X:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::TextInput::builder().flags(nwg::TextInputFlags::NUMBER).size((50,24)).position((195,218)).text("100").parent(&data.window).build(&mut data.ov_x)?;
            nwg::Label::builder().size((20,20)).position((255,220)).text("Y:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::TextInput::builder().flags(nwg::TextInputFlags::NUMBER).size((50,24)).position((270,218)).text("100").parent(&data.window).build(&mut data.ov_y)?;
            nwg::Label::builder().size((35,20)).position((330,220)).text("Font:").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::TextInput::builder().flags(nwg::TextInputFlags::NUMBER).size((40,24)).position((365,218)).text("16").parent(&data.window).build(&mut data.ov_font)?;

            // === FORMATTING ===
            nwg::Frame::builder()
                .size((490, 80)).position((15, 275))
                .parent(&data.window).build(&mut data._formatting)?;
            nwg::Label::builder().size((100,16)).position((22,12)).text("Formatting Rules").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::CheckBox::builder().size((240,22)).position((25,295)).text("Space before ? and ! (French)").parent(&data.window).build(&mut data.space_cb)?;
            nwg::CheckBox::builder().size((150,22)).position((280,295)).text("Text-to-number").parent(&data.window).build(&mut data.num_cb)?;
            nwg::CheckBox::builder().size((120,22)).position((25,322)).text("Trailing space").parent(&data.window).build(&mut data.trail_cb)?;
            nwg::Label::builder().size((180,20)).position((160,324)).text("Auto-cap threshold (words):").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::TextInput::builder().flags(nwg::TextInputFlags::NUMBER).size((50,24)).position((340,320)).text("3").parent(&data.window).build(&mut data.cap_thresh)?;

            // === OUTPUT ===
            nwg::Frame::builder()
                .size((490, 55)).position((15, 360))
                .parent(&data.window).build(&mut data._output)?;
            nwg::Label::builder().size((100,16)).position((22,12)).text("Output").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::CheckBox::builder().size((250,22)).position((25,380)).text("Auto-paste to active window").parent(&data.window).build(&mut data.paste_cb)?;

            // === DICTIONARY ===
            nwg::Frame::builder()
                .size((490, 165)).position((15, 420))
                .parent(&data.window).build(&mut data._dictionary)?;
            nwg::Label::builder().size((100,16)).position((22,12)).text("Dictionary").parent(&data.window).build(&mut lbl)?;
            data.labels.push(lbl);
            lbl = nwg::Label::default();
            nwg::ListBox::builder().size((260,140)).position((25,440)).parent(&data.window).build(&mut data.dict_list)?;
            nwg::TextInput::builder().size((120,24)).position((300,440)).text("Word").parent(&data.window).build(&mut data.word_input)?;
            nwg::TextInput::builder().size((120,24)).position((300,470)).text("Replacement").parent(&data.window).build(&mut data.repl_input)?;
            nwg::Button::builder().size((80,26)).position((300,500)).text("Add").parent(&data.window).build(&mut data.add_btn)?;
            nwg::Button::builder().size((80,26)).position((390,500)).text("Delete").parent(&data.window).build(&mut data.del_btn)?;

            // === BOTTOM ===
            nwg::Button::builder().size((130,30)).position((120,640)).text("Save && Close").parent(&data.window).build(&mut data.save_btn)?;
            nwg::Button::builder().size((130,30)).position((270,640)).text("Cancel").parent(&data.window).build(&mut data.cancel_btn)?;

            // Populate controls
            for (w, r) in data.dict_entries.borrow().iter() {
                let _ = data.dict_list.push(format!("{} -> {}", w, r));
            }
            data.populate();

            let ui = SettingsWindowUi {
                inner: Rc::new(data),
                default_handler: RefCell::new(None),
            };

            let cap = ui.cap_state.clone();
            let evt_ui = Rc::downgrade(&ui.inner);

            let handle_events = move |evt, evt_data: nwg::EventData, handle: nwg::ControlHandle| {
                if let Some(ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose if handle == ui.window.handle => nwg::stop_thread_dispatch(),
                        E::OnButtonClick => {
                            if handle == ui.save_btn.handle {
                                let cfg = ui.read_config();
                                { *ui.config.lock().unwrap() = cfg.clone(); }
                                let _ = cfg.save();
                                let entries = ui.dict_entries.borrow();
                                let mut d = ui.dict.lock().unwrap();
                                d.entries = entries.clone().into_iter().collect();
                                let _ = d.save();
                                nwg::stop_thread_dispatch();
                            } else if handle == ui.cancel_btn.handle {
                                nwg::stop_thread_dispatch();
                            } else if handle == ui.add_btn.handle {
                                let word = ui.word_input.text().trim().to_string();
                                let repl = ui.repl_input.text().trim().to_string();
                                if !word.is_empty() && !repl.is_empty() {
                                    ui.dict_entries.borrow_mut().push((word.clone(), repl.clone()));
                                    let _ = ui.dict_list.push(format!("{} -> {}", word, repl));
                                    ui.word_input.set_text(""); ui.repl_input.set_text("");
                                }
                            } else if handle == ui.del_btn.handle {
                                if let Some(idx) = ui.dict_list.selection() {
                                    ui.dict_list.remove(idx);
                                    ui.dict_entries.borrow_mut().remove(idx);
                                }
                            } else if handle == ui.capture_btn.handle {
                                ui.start_capture();
                            }
                        }
                        E::OnKeyPress => {
                            if *cap.borrow() {
                                let vk = evt_data.on_key();
                                if vk != 0 {
                                    ui.key_input.set_text(&crate::hook::vk_display(vk));
                                    *cap.borrow_mut() = false;
                                    ui.capture_btn.set_text("Capture");
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