use crate::core::console::Console;
use crate::core::frame::Frame;
use crate::core::joypad::Buttons;
use crate::ines_parser::File;
use crossbeam::channel::{self, Sender};
use eframe::egui::{self, menu, CentralPanel, ColorImage, Key, Ui};
use eframe::epaint::ImageData;
use eframe::App;
use lazy_static::lazy_static;
use rfd::FileDialog;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

impl From<Frame> for ImageData {
    fn from(value: Frame) -> Self {
        ColorImage::from_rgb([256, 240], &value.image).into()
    }
}

lazy_static! {
    static ref KEY_MAP: HashMap<Key, Buttons> = [
        (Key::W, Buttons::UP),
        (Key::S, Buttons::DOWN),
        (Key::D, Buttons::RIGHT),
        (Key::A, Buttons::LEFT),
        (Key::U, Buttons::SELECT),
        (Key::I, Buttons::START),
        (Key::K, Buttons::A),
        (Key::J, Buttons::B),
    ]
    .iter()
    .fold(HashMap::new(), |mut acc, (key, button)| {
        acc.insert(*key, *button);
        acc
    });
}

pub enum ConsoleMsg {
    JoypadDown(Buttons),
    JoypadUp(Buttons),
    RunFrame,
}

#[derive(Default)]
pub struct EGuiApp {
    console: Option<Arc<Mutex<Console<'static>>>>,
    channel: Option<Sender<ConsoleMsg>>,
}

impl App for EGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(channel) = &self.channel {
            channel.send(ConsoleMsg::RunFrame).unwrap();
        }

        // Draw
        ctx.request_repaint_after(Duration::new(0, 16_666_667 / 2));
        CentralPanel::default().show(ctx, |ui| {
            menu::bar(ui, |ui| {
                if ui.button("Load ROM").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.load(File::new(path))
                    }
                }
            });

            self.show_texture(ui);
            self.handle_keyevent(ctx);
        });
    }
}

impl EGuiApp {
    pub fn new() -> Self {
        Self {
            channel: None,
            console: None,
        }
    }

    fn load(&mut self, rom: File) {
        let (send, recv) = channel::bounded::<ConsoleMsg>(1024);
        let console = Arc::new(Mutex::new(Console::new(rom)));
        self.channel = Some(send);
        self.console = Some(console.clone());

        std::thread::spawn(move || {
            Console::run_thread(console.clone(), recv);
        });
    }

    fn show_texture(&self, ui: &mut Ui) {
        if let Some(console) = &self.console {
            let console = console.lock().unwrap();
            let texture =
                ui.ctx()
                    .load_texture("NES", console.cpu.bus.ppu.curr_frame, Default::default());
            ui.image((texture.id(), texture.size_vec2()));
        }
    }

    fn handle_keyevent(&mut self, ctx: &eframe::egui::Context) {
        if let Some(channel) = &self.channel {
            let keys_down = ctx.input(|i| i.keys_down.clone());
            KEY_MAP.iter().for_each(|(key, button)| {
                if keys_down.contains(key) {
                    channel.try_send(ConsoleMsg::JoypadDown(*button)).unwrap();
                } else {
                    channel.send(ConsoleMsg::JoypadUp(*button)).unwrap();
                }
            });
        }
    }
}
