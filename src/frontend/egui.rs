use crate::config::Config;
use crate::core::bus::Bus;
use crate::core::cpu::CPU;
use crate::core::frame::Frame;
use crate::core::joypad::Buttons;
use crate::ines_parser::File;
use eframe::egui::{ColorImage, Key, Ui};
use eframe::epaint::ImageData;
use eframe::App;
use lazy_static::lazy_static;
use std::collections::HashMap;
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

pub struct EGuiApp {
    cpu: CPU<'static>,
}

impl App for EGuiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.cpu.run_until_frame();
        ctx.request_repaint_after(Duration::new(0, 16_666_667));
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.show_texture(ui);
            self.handle_keyevent(ctx);
        });
    }
}

impl EGuiApp {
    pub fn new(rom: File) -> Self {
        let mut cpu = CPU::new(Bus::new(&rom));

        if Config::get_bool("enable_logging", false) {
            cpu.set_sink(Box::new(
                std::fs::File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(Config::get_string("logging_path", "log.log"))
                    .unwrap(),
            ));
            cpu.enable_logging();
        }
        cpu.reset();

        Self { cpu }
    }

    fn show_texture(&self, ui: &mut Ui) {
        let texture = ui
            .ctx()
            .load_texture("NES", self.cpu.bus.ppu.curr_frame, Default::default());
        ui.image((texture.id(), texture.size_vec2()));
    }

    fn handle_keyevent(&mut self, ctx: &eframe::egui::Context) {
        let joypad = &mut self.cpu.bus.joypad;
        let keys_down = ctx.input(|i| i.keys_down.clone());
        KEY_MAP.iter().for_each(|(key, button)| {
            joypad.buttons.set(*button, keys_down.contains(key));
        });
    }
}
