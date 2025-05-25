use egui::Ui;

pub struct ValuesConfigButtonsStartReset {
    pub simulation_stopped: bool,
}

pub fn draw_start_reset_buttons(
    ui: &mut egui::Ui,
    mut values: ValuesConfigButtonsStartReset,
    mut on_change: impl FnMut(&mut egui::Ui, bool, bool),
) {
    ui.vertical(|ui| {
        ui.label("Stop or start simulation again or reset to default settings.");
        ui.horizontal(|ui| {
            let start_simulation_stopped = values.simulation_stopped;
            if ui
                .button(match values.simulation_stopped {
                    true => "start",
                    false => "stop",
                })
                .clicked()
            {
                values.simulation_stopped = !values.simulation_stopped;
            };

            let mut reset_pressed = false;
            if ui.button("reset").clicked() {
                reset_pressed = true;
            }

            if start_simulation_stopped != values.simulation_stopped || reset_pressed {
                on_change(ui, values.simulation_stopped, reset_pressed);
            }
        });
    });
}

pub struct ValuesSectionDebug {
    pub zoom: f32,
    pub pan: [f32; 2],
    pub fps: f32,
}

pub fn draw_section_debug(ui: &mut egui::Ui, values: ValuesSectionDebug) {
    ui.label(format!("zoom: {:.5}", values.zoom));
    ui.label(format!("pan: [{:.5}, {:.5}]", values.pan[0], values.pan[1]));
    ui.label(format!("FPS: {:.1}", values.fps));
}

pub struct ValuesConfigSlidersGraph {
    pub node_cnt: usize,
    pub edge_cnt: usize,
}

pub fn draw_counts_sliders(
    ui: &mut egui::Ui,
    mut values: ValuesConfigSlidersGraph,
    mut on_change: impl FnMut(i32, i32),
) {
    let start_node_cnt = values.node_cnt;
    let mut delta_node_cnt = 0;
    ui.horizontal(|ui| {
        if ui
            .add(egui::Slider::new(&mut values.node_cnt, 1..=2500).text("nodes"))
            .changed()
        {
            delta_node_cnt = values.node_cnt as i32 - start_node_cnt as i32;
        };
    });

    let start = values.edge_cnt;
    let mut delta_edge_cnt = 0;
    ui.horizontal(|ui| {
        if ui
            .add(egui::Slider::new(&mut values.edge_cnt, 1..=2500).text("edges"))
            .changed()
        {
            delta_edge_cnt = values.edge_cnt as i32 - start as i32;
        };
    });

    if delta_node_cnt != 0 || delta_edge_cnt != 0 {
        on_change(delta_node_cnt, delta_edge_cnt)
    };
}

pub struct ValuesConfigSlidersSimulation {
    pub dt: f32,
    pub cooloff_factor: f32,
    pub scale: f32,
}

pub fn draw_simulation_config_sliders(
    ui: &mut Ui,
    mut values: ValuesConfigSlidersSimulation,
    mut on_change: impl FnMut(f32, f32, f32),
) {
    let start_dt = values.dt;
    let mut delta_dt = 0.;
    ui.horizontal(|ui| {
        if ui
            .add(egui::Slider::new(&mut values.dt, 0.00..=1.).text("dt"))
            .changed()
        {
            delta_dt = values.dt - start_dt;
        };
    });

    let start_cooloff_factor = values.cooloff_factor;
    let mut delta_cooloff_factor = 0.;
    ui.horizontal(|ui| {
        if ui
            .add(egui::Slider::new(&mut values.cooloff_factor, 0.00..=1.).text("cooloff_factor"))
            .changed()
        {
            delta_cooloff_factor = values.cooloff_factor - start_cooloff_factor;
        };
    });

    let start_scale = values.scale;
    let mut delta_scale = 0.;
    ui.horizontal(|ui| {
        if ui
            .add(egui::Slider::new(&mut values.scale, 1.0..=1000.).text("scale"))
            .changed()
        {
            delta_scale = values.scale - start_scale;
        };
    });

    if delta_dt != 0. || delta_cooloff_factor != 0. || delta_scale != 0. {
        on_change(delta_dt, delta_cooloff_factor, delta_scale);
    }
}
