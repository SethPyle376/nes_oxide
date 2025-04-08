use imgui::*;

use crate::Cpu;

const DEBUG_INSTRUCTION_COUNT: u32 = 5;

pub struct DebugGui {
    pub mem_inspect_page: u8,
}

impl Default for DebugGui {
    fn default() -> Self {
        Self {
            mem_inspect_page: 0,
        }
    }
}

impl DebugGui {
    pub fn draw_debug(&mut self, cpu: &mut Cpu, ui: &mut Ui) {
        ui.window("CPU Status")
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(|| {
                let current_instruction_trace = cpu.trace();
                ui.text_wrapped(current_instruction_trace.0);
            });

        ui.window("Memory Inspector")
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(|| {
                ui.input_scalar("Page Index", &mut self.mem_inspect_page)
                    .step(1)
                    .build();
                let zero_page = cpu.bus.get_page(self.mem_inspect_page);
                if let Some(_) = ui.begin_table("zero_page_table", 16) {
                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    for i in 0..255 {
                        ui.text(format!("{:#04X}", zero_page[i]));
                        ui.table_next_column();
                    }
                }
            });
    }
}
