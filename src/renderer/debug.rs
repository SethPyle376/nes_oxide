use imgui::*;

use crate::{Bus, Cartridge, Cpu};

const DEBUG_INSTRUCTION_COUNT: u32 = 5;

pub struct DebugGui {
    pub mem_inspect_page: u8,
    rom_path: String
}

impl Default for DebugGui {
    fn default() -> Self {
        Self {
            mem_inspect_page: 0,
            rom_path: String::from("")
        }
    }
}

impl DebugGui {
    pub fn draw_debug(&mut self, texture_id: &TextureId, cpu: &mut Cpu, ui: &mut Ui) {
        let size = ui.io().display_size;
        ui.window("Emulator")
            .position([0.0, 0.0], Condition::Always)
            .size(size, Condition::Always)
            .build(|| {
                ui.columns(2, "columns", true);

                ui.child_window("Left Pane")
                    .border(true)
                    .build(|| {
                        if ui.collapsing_header("CPU Status", TreeNodeFlags::DEFAULT_OPEN) {
                            let current_instruction_trace = cpu.trace();
                            ui.text_wrapped(current_instruction_trace.0);
                        }

                        if ui.collapsing_header("Memory Usage", TreeNodeFlags::empty()) {
                            ui.input_scalar("Page Index", &mut self.mem_inspect_page)
                                .step(1)
                                .build();
                            let zero_page = cpu.bus.get_page(self.mem_inspect_page);
                            if let Some(_) = ui.begin_table("zero_page_table", 16) {
                                ui.table_next_row();
                                ui.table_set_column_index(0);
                                for i in 0..255 {
                                    ui.text(format!("{:02X}", zero_page[i]));
                                    ui.table_next_column();
                                }
                            }
                        }

                        if ui.collapsing_header("ROM Loader", TreeNodeFlags::empty()) {
                            ui.input_text("ROM Path", &mut self.rom_path).build();
                            if ui.button("Load ROM") {
                                let bus = Bus::new(Cartridge::load(&self.rom_path).unwrap());
                                cpu.reset(bus);
                            }
                        }
                    });
                ui.next_column();
                ui.child_window("Right Pane")
                    .border(true)
                    .build(|| {
                        if ui.collapsing_header("Render", TreeNodeFlags::DEFAULT_OPEN) {
                            Image::new(*texture_id, [512.0, 480.0]).build(ui);
                        }
                    })
            });
    }
}
