use crate::metrics::MetricsRecorder;
use crate::ui_consts::{DEBUG_MONO_FONT_SIZE, UI_MARGIN};
use crate::{MAX_EDGE_COUNT, MAX_NODE_COUNT};
use egui::{FontId, Pos2, Rect, Ui};

pub fn render(
    ui: &mut Ui,
    metrics: &MetricsRecorder,
    node_count: usize,
    edge_count: usize,
    last_step_count: usize,
    pan: Option<[f32; 2]>,
    zoom: Option<f32>,
) {
    // Compose overlay text
    let text = {
        let fps_line = format!("FPS: {:.1}", metrics.fps());
        let step_avg = metrics.step_avg_5s();
        let draw_avg = metrics.draw_avg_5s();
        let step_line = format!("TStep: {:.2} ms (avg 5s)", step_avg);
        let draw_line = format!("TDraw: {:.2} ms (avg 5s)", draw_avg);
        let n_line = if node_count >= MAX_NODE_COUNT {
            format!("N: {node_count} MAX")
        } else {
            format!("N: {node_count}")
        };
        let e_line = if edge_count >= MAX_EDGE_COUNT {
            format!("E: {edge_count} MAX")
        } else {
            format!("E: {edge_count}")
        };
        let steps_line = format!("Steps: {}", last_step_count);
        let (pan_line, zoom_line) = match (pan, zoom) {
            (Some(p), Some(z)) => (
                Some(format!("Pan: [{:.1}, {:.1}]", p[0], p[1])),
                Some(format!("Zoom: {:.2}", z)),
            ),
            (Some(p), None) => (Some(format!("Pan: [{:.1}, {:.1}]", p[0], p[1])), None),
            (None, Some(z)) => (None, Some(format!("Zoom: {:.2}", z))),
            (None, None) => (None, None),
        };
        let mut lines = vec![fps_line, step_line, draw_line, n_line, e_line, steps_line];
        if let Some(pl) = pan_line {
            lines.push(pl);
        }
        if let Some(zl) = zoom_line {
            lines.push(zl);
        }
        lines.join("\n")
    };

    let text_color = ui.style().visuals.strong_text_color();
    let panel_rect: Rect = ui.max_rect();
    let font_id = FontId::monospace(DEBUG_MONO_FONT_SIZE);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.clone(), font_id, text_color));
    let pos = Pos2::new(
        panel_rect.right() - UI_MARGIN - galley.size().x,
        panel_rect.top() + UI_MARGIN,
    );
    let painter = ui.painter_at(panel_rect);
    painter.galley(pos, galley, text_color);
}
