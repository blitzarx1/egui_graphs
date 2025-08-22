use egui::{Context, Key, Modifiers};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    ToggleDebug,
    OpenKeybindings,
    CloseKeybindings,
    ResetAll,
    ToggleNavMode,
    FitToScreenOnce,
    PanToGraph,
    AddNodes(u32),
    RemoveNodes(u32),
    SwapNodes(u32),
    AddEdges(u32),
    RemoveEdges(u32),
    SwapEdges(u32),
}

pub fn dispatch(ctx: &Context) -> Vec<Command> {
    let mut cmds = Vec::new();
    let mut any_key_pressed = false;
    let mut any_pointer_pressed = false;
    let mut pressed_h = false; // 'h' or 'H'
    let mut pressed_shift_slash = false; // '?'

    ctx.input(|i| {
        for ev in &i.events {
            match ev {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                    ..
                } => {
                    if *pressed {
                        any_key_pressed = true;
                    }
                    if *pressed && !modifiers.any() && *key == Key::H {
                        pressed_h = true;
                    }
                    if *pressed && *key == Key::Slash && modifiers.shift {
                        pressed_shift_slash = true;
                    }
                }
                egui::Event::PointerButton { pressed, .. } => {
                    if *pressed {
                        any_pointer_pressed = true;
                    }
                }
                egui::Event::Text(t) => {
                    if t == "?" {
                        pressed_shift_slash = true;
                    }
                    if t.eq_ignore_ascii_case("h") {
                        pressed_h = true;
                    }
                }
                _ => {}
            }
        }

        // Backspace: reset defaults
        if i.key_pressed(Key::Backspace) && !i.modifiers.any() {
            cmds.push(Command::ResetAll);
        }
        // Space family: navigation controls
        if i.key_pressed(Key::Space) {
            if i.modifiers.ctrl && i.modifiers.shift {
                cmds.push(Command::PanToGraph);
            } else if i.modifiers.ctrl {
                cmds.push(Command::ToggleNavMode);
            } else if !i.modifiers.any() {
                cmds.push(Command::FitToScreenOnce);
            }
        }
        // Esc: close modal if open
        if i.key_pressed(Key::Escape) {
            cmds.push(Command::CloseKeybindings);
        }
        // Tab handled centrally in DemoApp::process_keybindings (consumed globally).

        for ev in &i.events {
            if let egui::Event::Key {
                key,
                pressed,
                modifiers,
                ..
            } = ev
            {
                if !pressed {
                    continue;
                }
                match key {
                    Key::D => {
                        if !modifiers.any() {
                            cmds.push(Command::ToggleDebug);
                        }
                    }
                    Key::G => {}
                    Key::H => {}
                    Key::Slash => {}
                    Key::N => push_node_cmds(&mut cmds, modifiers),
                    Key::M => push_node10_cmds(&mut cmds, modifiers),
                    Key::E => push_edge_cmds(&mut cmds, modifiers),
                    Key::R => push_edge10_cmds(&mut cmds, modifiers),
                    _ => {}
                }
            }
        }
    });

    // Tab is consumed and handled in DemoApp::process_keybindings

    if pressed_h || pressed_shift_slash {
        cmds.push(Command::OpenKeybindings);
    }
    if any_key_pressed || any_pointer_pressed { /* could be used by caller */ }

    cmds
}

fn push_node_cmds(cmds: &mut Vec<Command>, m: &Modifiers) {
    if m.ctrl && m.shift {
        cmds.push(Command::SwapNodes(1));
    } else if m.shift {
        cmds.push(Command::RemoveNodes(1));
    } else {
        cmds.push(Command::AddNodes(1));
    }
}
fn push_node10_cmds(cmds: &mut Vec<Command>, m: &Modifiers) {
    if m.ctrl && m.shift {
        cmds.push(Command::SwapNodes(10));
    } else if m.shift {
        cmds.push(Command::RemoveNodes(10));
    } else {
        cmds.push(Command::AddNodes(10));
    }
}
fn push_edge_cmds(cmds: &mut Vec<Command>, m: &Modifiers) {
    if m.ctrl && m.shift {
        cmds.push(Command::SwapEdges(1));
    } else if m.shift {
        cmds.push(Command::RemoveEdges(1));
    } else {
        cmds.push(Command::AddEdges(1));
    }
}
fn push_edge10_cmds(cmds: &mut Vec<Command>, m: &Modifiers) {
    if m.ctrl && m.shift {
        cmds.push(Command::SwapEdges(10));
    } else if m.shift {
        cmds.push(Command::RemoveEdges(10));
    } else {
        cmds.push(Command::AddEdges(10));
    }
}
