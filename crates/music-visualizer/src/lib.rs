use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

// Playlist track information
#[derive(Clone, Default)]
pub struct PlaylistTrack {
    pub name: String,
    pub duration: f64,      // Duration in seconds
    pub file_type: String,  // mp3, wav, ogg, flac, etc.
}

// Playlist and playback state
#[derive(Clone)]
pub struct PlaylistState {
    pub tracks: Vec<PlaylistTrack>,
    pub current_index: Option<usize>,
    pub is_playing: bool,
    pub is_shuffled: bool,
    pub shuffle_order: Vec<usize>,
    pub current_time: f64,
    pub duration: f64,
    pub volume: f32,
}

impl Default for PlaylistState {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            is_playing: false,
            is_shuffled: false,
            shuffle_order: Vec::new(),
            current_time: 0.0,
            duration: 0.0,
            volume: 0.8,
        }
    }
}

impl PlaylistState {
    pub fn get_current_track(&self) -> Option<&PlaylistTrack> {
        self.current_index.and_then(|idx| self.tracks.get(idx))
    }
    
    pub fn get_progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.current_time / self.duration) as f32
        } else {
            0.0
        }
    }
    
    pub fn format_time(seconds: f64) -> String {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{:02}:{:02}", mins, secs)
    }
    
    pub fn shuffle_playlist(&mut self) {
        let len = self.tracks.len();
        if len == 0 {
            return;
        }
        
        self.shuffle_order = (0..len).collect();
        // Simple Fisher-Yates shuffle using our rand function
        for i in (1..len).rev() {
            let j = (rand_float() * (i + 1) as f32) as usize;
            self.shuffle_order.swap(i, j);
        }
    }
    
    pub fn get_next_index(&self) -> Option<usize> {
        let len = self.tracks.len();
        if len == 0 {
            return None;
        }
        
        match self.current_index {
            Some(idx) => {
                if self.is_shuffled && !self.shuffle_order.is_empty() {
                    let current_shuffle_pos = self.shuffle_order.iter().position(|&x| x == idx)?;
                    let next_shuffle_pos = (current_shuffle_pos + 1) % self.shuffle_order.len();
                    Some(self.shuffle_order[next_shuffle_pos])
                } else {
                    Some((idx + 1) % len)
                }
            }
            None => Some(0),
        }
    }
    
    pub fn get_prev_index(&self) -> Option<usize> {
        let len = self.tracks.len();
        if len == 0 {
            return None;
        }
        
        match self.current_index {
            Some(idx) => {
                if self.is_shuffled && !self.shuffle_order.is_empty() {
                    let current_shuffle_pos = self.shuffle_order.iter().position(|&x| x == idx)?;
                    let prev_shuffle_pos = if current_shuffle_pos == 0 {
                        self.shuffle_order.len() - 1
                    } else {
                        current_shuffle_pos - 1
                    };
                    Some(self.shuffle_order[prev_shuffle_pos])
                } else {
                    Some(if idx == 0 { len - 1 } else { idx - 1 })
                }
            }
            None => Some(0),
        }
    }
}

// Audio analysis data extracted from Web Audio API
#[derive(Clone, Default)]
pub struct AudioAnalysis {
    // Frequency bands (normalized 0.0-1.0)
    pub bass: f32,         // 20-250 Hz
    pub low_mid: f32,      // 250-500 Hz
    pub mid: f32,          // 500-2000 Hz
    pub high_mid: f32,     // 2000-4000 Hz
    pub treble: f32,       // 4000-20000 Hz
    
    // Overall metrics
    pub volume: f32,       // RMS volume
    pub peak: f32,         // Peak amplitude
    
    // Beat detection
    pub beat: bool,        // True when beat detected
    pub beat_intensity: f32,
    
    // Spectral features
    pub spectral_centroid: f32,
    pub spectral_flux: f32,
    
    // Smoothed values for animation
    pub smooth_bass: f32,
    pub smooth_mid: f32,
    pub smooth_treble: f32,
    pub smooth_volume: f32,
    
    // Raw frequency data
    pub frequency_data: Vec<u8>,
    pub time_data: Vec<u8>,
}

impl AudioAnalysis {
    pub fn new() -> Self {
        Self {
            frequency_data: vec![0u8; 256],
            time_data: vec![0u8; 256],
            ..Default::default()
        }
    }
    
    pub fn update_from_fft(&mut self, frequency_data: &[u8], time_data: &[u8]) {
        self.frequency_data = frequency_data.to_vec();
        self.time_data = time_data.to_vec();
        
        let len = frequency_data.len();
        if len == 0 {
            return;
        }
        
        // Calculate frequency bands
        let bass_range = 0..len / 16;
        let low_mid_range = len / 16..len / 8;
        let mid_range = len / 8..len / 4;
        let high_mid_range = len / 4..len / 2;
        let treble_range = len / 2..len;
        
        let calc_band_avg = |range: std::ops::Range<usize>| -> f32 {
            if range.is_empty() {
                return 0.0;
            }
            let sum: u32 = frequency_data[range.clone()].iter().map(|&x| x as u32).sum();
            (sum as f32) / (range.len() as f32 * 255.0)
        };
        
        let new_bass = calc_band_avg(bass_range);
        let new_low_mid = calc_band_avg(low_mid_range);
        let new_mid = calc_band_avg(mid_range);
        let new_high_mid = calc_band_avg(high_mid_range);
        let new_treble = calc_band_avg(treble_range);
        
        // Calculate volume (RMS)
        let rms: f32 = (time_data.iter()
            .map(|&x| {
                let centered = (x as f32) - 128.0;
                centered * centered
            })
            .sum::<f32>() / time_data.len() as f32)
            .sqrt() / 128.0;
        
        // Peak detection
        let peak = time_data.iter()
            .map(|&x| ((x as f32) - 128.0).abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0) / 128.0;
        
        // Beat detection (energy spike in bass)
        let bass_threshold = 0.6;
        let energy_jump = new_bass - self.smooth_bass;
        self.beat = energy_jump > 0.1 && new_bass > bass_threshold;
        self.beat_intensity = if self.beat { energy_jump.min(1.0) } else { 0.0 };
        
        // Spectral centroid (brightness)
        let total_energy: f32 = frequency_data.iter().map(|&x| x as f32).sum();
        if total_energy > 0.0 {
            let weighted_sum: f32 = frequency_data.iter()
                .enumerate()
                .map(|(i, &x)| (i as f32) * (x as f32))
                .sum();
            self.spectral_centroid = weighted_sum / total_energy / len as f32;
        }
        
        // Spectral flux (change in spectrum)
        let flux: f32 = frequency_data.iter()
            .zip(self.frequency_data.iter())
            .map(|(&new, &old)| {
                let diff = (new as f32) - (old as f32);
                if diff > 0.0 { diff } else { 0.0 }
            })
            .sum::<f32>() / (len as f32 * 255.0);
        self.spectral_flux = flux;
        
        // Smooth transitions
        let smoothing = 0.15;
        self.smooth_bass = self.smooth_bass + (new_bass - self.smooth_bass) * smoothing;
        self.smooth_mid = self.smooth_mid + (new_mid - self.smooth_mid) * smoothing;
        self.smooth_treble = self.smooth_treble + (new_treble - self.smooth_treble) * smoothing;
        self.smooth_volume = self.smooth_volume + (rms - self.smooth_volume) * smoothing;
        
        // Update raw values
        self.bass = new_bass;
        self.low_mid = new_low_mid;
        self.mid = new_mid;
        self.high_mid = new_high_mid;
        self.treble = new_treble;
        self.volume = rms;
        self.peak = peak;
    }
    
    // Demo mode with simulated audio
    pub fn simulate_demo(&mut self, time: f64) {
        // Simulate bass beat
        let beat_freq = 2.0; // BPM / 60
        let beat_phase = (time * beat_freq * std::f64::consts::TAU).sin();
        let beat_envelope = ((beat_phase + 1.0) / 2.0).powf(4.0) as f32;
        
        self.bass = 0.3 + beat_envelope * 0.5;
        self.low_mid = 0.25 + (time * 1.5).sin() as f32 * 0.15;
        self.mid = 0.3 + (time * 2.3).sin() as f32 * 0.2;
        self.high_mid = 0.2 + (time * 3.7).sin() as f32 * 0.15;
        self.treble = 0.15 + (time * 5.1).sin() as f32 * 0.1;
        
        self.volume = 0.4 + beat_envelope * 0.3;
        self.peak = self.volume * 1.2;
        
        self.beat = beat_envelope > 0.8;
        self.beat_intensity = if self.beat { beat_envelope } else { 0.0 };
        
        self.spectral_centroid = 0.5 + (time * 0.5).sin() as f32 * 0.3;
        self.spectral_flux = beat_envelope * 0.5;
        
        // Smooth values
        let smoothing = 0.1;
        self.smooth_bass = self.smooth_bass + (self.bass - self.smooth_bass) * smoothing;
        self.smooth_mid = self.smooth_mid + (self.mid - self.smooth_mid) * smoothing;
        self.smooth_treble = self.smooth_treble + (self.treble - self.smooth_treble) * smoothing;
        self.smooth_volume = self.smooth_volume + (self.volume - self.smooth_volume) * smoothing;
        
        // Generate demo frequency/time data
        for i in 0..self.frequency_data.len() {
            let freq_norm = i as f64 / self.frequency_data.len() as f64;
            let value = ((1.0 - freq_norm).powf(2.0) * self.bass as f64 * 200.0
                + (time * (10.0 + i as f64 * 0.5)).sin().abs() * 50.0) as u8;
            self.frequency_data[i] = value;
        }
        
        for i in 0..self.time_data.len() {
            let t = i as f64 / self.time_data.len() as f64;
            let wave = (t * std::f64::consts::TAU * 4.0 + time * 10.0).sin();
            let value = 128.0 + wave * 64.0 * self.volume as f64;
            self.time_data[i] = value.clamp(0.0, 255.0) as u8;
        }
    }
}

// Configuration for visualizer
#[derive(Clone)]
pub struct VisualizerConfig {
    // Base fractal parameters
    pub base_zoom: f32,
    pub base_width: f32,
    pub base_depth: u32,
    pub base_brightness: f32,
    
    // Audio reactivity multipliers
    pub zoom_bass_mult: f32,
    pub width_bass_mult: f32,
    pub depth_complexity_mult: f32,
    pub brightness_treble_mult: f32,
    pub rotation_beat_mult: f32,
    
    // Animation
    pub auto_rotate: bool,
    pub rotation_speed: f32,
    pub pulse_on_beat: bool,
    pub color_cycle: bool,
    pub color_cycle_speed: f32,
    
    // Visual style
    pub base_color: Color32,
    pub accent_color: Color32,
    pub background_color: Color32,
    pub glow_intensity: f32,
    pub particle_count: usize,
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self {
            base_zoom: 0.1,
            base_width: 1.0,
            base_depth: 16,
            base_brightness: 0.8,
            
            zoom_bass_mult: 0.1,
            width_bass_mult: 0.3,
            depth_complexity_mult: 4.0,
            brightness_treble_mult: 0.4,
            rotation_beat_mult: 0.1,
            
            auto_rotate: true,
            rotation_speed: 1.0,
            pulse_on_beat: true,
            color_cycle: true,
            color_cycle_speed: 0.1,
            
            base_color: Color32::from_rgb(100, 200, 255),
            accent_color: Color32::from_rgb(255, 100, 200),
            background_color: Color32::from_rgb(10, 10, 20),
            glow_intensity: 0.5,
            particle_count: 50,
        }
    }
}

// Particle for beat effects
#[derive(Clone)]
struct Particle {
    pos: Pos2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    color: Color32,
}

impl Particle {
    fn new(center: Pos2, angle: f32, speed: f32, color: Color32) -> Self {
        Self {
            pos: center,
            vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            life: 1.0,
            max_life: 1.0,
            size: 3.0 + rand_float() * 5.0,
            color,
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.vel *= 0.98; // Friction
        self.life -= dt / self.max_life;
    }
    
    fn is_alive(&self) -> bool {
        self.life > 0.0
    }
}

// Simple random function for WASM
fn rand_float() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    thread_local! {
        static SEED: RefCell<u64> = RefCell::new(12345);
    }
    
    SEED.with(|seed| {
        let mut s = seed.borrow_mut();
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        *s = hasher.finish();
        *s as f32 / u64::MAX as f32
    })
}

// Web Audio wrapper (placeholder for future expansion)
#[allow(dead_code)]
#[derive(Clone)]
struct WebAudio {
    initialized: bool,
    error_message: Option<String>,
}

impl Default for WebAudio {
    fn default() -> Self {
        Self {
            initialized: false,
            error_message: None,
        }
    }
}

// Main visualizer app
pub struct MusicVisualizerApp {
    audio: AudioAnalysis,
    config: VisualizerConfig,
    #[allow(dead_code)]
    web_audio: WebAudio,
    
    // Animation state
    time: f64,
    rotation: f32,
    particles: Vec<Particle>,
    
    // Audio data shared with JS callback
    audio_data: Rc<RefCell<(Vec<u8>, Vec<u8>)>>,
    audio_initialized: Rc<RefCell<bool>>,
    
    // Playlist state (shared for file input callback)
    playlist: PlaylistState,
    pending_tracks: Rc<RefCell<Vec<(String, String, String)>>>, // (name, type, url)
    audio_element: Rc<RefCell<Option<web_sys::HtmlAudioElement>>>,
    audio_context: Rc<RefCell<Option<web_sys::AudioContext>>>,
    analyser_node: Rc<RefCell<Option<web_sys::AnalyserNode>>>,
    file_audio_initialized: Rc<RefCell<bool>>,
    
    // UI state
    demo_mode: bool,
    show_spectrum: bool,
    show_waveform: bool,
    show_settings: bool,
    beat_flash: f32,
}

impl Default for MusicVisualizerApp {
    fn default() -> Self {
        Self {
            audio: AudioAnalysis::new(),
            config: VisualizerConfig::default(),
            web_audio: WebAudio::default(),
            time: 0.0,
            rotation: 0.0,
            particles: Vec::new(),
            audio_data: Rc::new(RefCell::new((vec![0u8; 256], vec![0u8; 256]))),
            audio_initialized: Rc::new(RefCell::new(false)),
            playlist: PlaylistState::default(),
            pending_tracks: Rc::new(RefCell::new(Vec::new())),
            audio_element: Rc::new(RefCell::new(None)),
            audio_context: Rc::new(RefCell::new(None)),
            analyser_node: Rc::new(RefCell::new(None)),
            file_audio_initialized: Rc::new(RefCell::new(false)),
            demo_mode: true,
            show_spectrum: true,
            show_waveform: true,
            show_settings: true,
            beat_flash: 0.0,
        }
    }
}

impl MusicVisualizerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    
    fn try_init_audio(&mut self) {
        if *self.audio_initialized.borrow() {
            return;
        }
        
        let audio_data = self.audio_data.clone();
        let audio_initialized = self.audio_initialized.clone();
        
        spawn_local(async move {
            match init_web_audio(audio_data.clone()).await {
                Ok(_) => {
                    *audio_initialized.borrow_mut() = true;
                    web_sys::console::log_1(&"Audio initialized successfully!".into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Audio init failed: {:?}", e).into());
                }
            }
        });
    }
    
    fn update_audio(&mut self, dt: f32) {
        // Check if playing from file
        let is_file_playing = self.playlist.is_playing && *self.file_audio_initialized.borrow();
        
        if is_file_playing {
            // Use audio data from file playback
            let data = self.audio_data.borrow();
            self.audio.update_from_fft(&data.0, &data.1);
        } else if self.demo_mode || !*self.audio_initialized.borrow() {
            self.audio.simulate_demo(self.time);
        } else {
            // Use microphone audio data
            let data = self.audio_data.borrow();
            self.audio.update_from_fft(&data.0, &data.1);
        }
        
        // Beat flash decay
        if self.audio.beat {
            self.beat_flash = 1.0;
        }
        self.beat_flash *= 0.9_f32.powf(dt * 60.0);
    }
    
    fn update_animation(&mut self, dt: f32) {
        self.time += dt as f64;
        
        // Rotation
        if self.config.auto_rotate {
            let beat_boost = if self.audio.beat { self.config.rotation_beat_mult } else { 0.0 };
            self.rotation += (self.config.rotation_speed + beat_boost) * dt;
        }
        
        // Spawn particles on beat
        if self.audio.beat && self.config.pulse_on_beat {
            let center = Pos2::new(400.0, 300.0); // Will be updated in render
            let color = self.get_current_color();
            for _ in 0..5 {
                let angle = rand_float() * std::f32::consts::TAU;
                let speed = 100.0 + rand_float() * 200.0;
                self.particles.push(Particle::new(center, angle, speed, color));
            }
        }
        
        // Update particles
        for p in &mut self.particles {
            p.update(dt);
        }
        self.particles.retain(|p| p.is_alive());
        
        // Limit particle count
        while self.particles.len() > self.config.particle_count * 2 {
            self.particles.remove(0);
        }
    }
    
    fn get_current_color(&self) -> Color32 {
        if self.config.color_cycle {
            let hue = (self.time as f32 * self.config.color_cycle_speed) % 1.0;
            hsl_to_rgb(hue, 0.8, 0.6)
        } else {
            self.config.base_color
        }
    }
    
    fn draw_fractal(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter();
        let center = rect.center();
        
        // Calculate reactive parameters
        let zoom = self.config.base_zoom + self.audio.smooth_bass * self.config.zoom_bass_mult;
        let width = self.config.base_width + self.audio.smooth_bass * self.config.width_bass_mult;
        let depth = (self.config.base_depth as f32 
            + self.audio.spectral_centroid * self.config.depth_complexity_mult) as u32;
        let brightness = self.config.base_brightness 
            + self.audio.smooth_treble * self.config.brightness_treble_mult;
        
        // Draw background with beat flash
        let bg_intensity = (self.beat_flash * 30.0) as u8;
        let bg = Color32::from_rgb(
            self.config.background_color.r().saturating_add(bg_intensity),
            self.config.background_color.g().saturating_add(bg_intensity / 2),
            self.config.background_color.b().saturating_add(bg_intensity),
        );
        painter.rect_filled(rect, 0.0, bg);
        
        // Clip drawing to rect
        let clip_rect = rect;
        
        // Calculate base length to fit within the rect (use smaller dimension)
        let max_size = rect.width().min(rect.height()) * 0.35;
        let base_length = max_size * zoom;
        let branch_angle = std::f32::consts::PI / 4.0 * width;
        let color = self.get_current_color();
        
        // Draw glow effect at center first (behind fractal)
        if self.config.glow_intensity > 0.0 {
            let glow_color = Color32::from_rgba_unmultiplied(
                color.r(),
                color.g(),
                color.b(),
                (self.config.glow_intensity * self.audio.smooth_volume * 100.0) as u8,
            );
            let glow_radius = (base_length * 0.5 * (1.0 + self.audio.smooth_bass)).min(max_size * 0.6);
            painter.circle_filled(center, glow_radius, glow_color);
        }
        
        // Draw fractal tree starting from center, growing upward
        self.draw_branch(
            painter,
            center,
            base_length,
            -std::f32::consts::PI / 2.0 + self.rotation * 0.1,
            branch_angle,
            depth,
            brightness,
            color,
            clip_rect,
        );
    }
    
    fn draw_branch(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        length: f32,
        angle: f32,
        branch_angle: f32,
        depth: u32,
        brightness: f32,
        color: Color32,
        clip_rect: Rect,
    ) {
        if depth == 0 || length < 2.0 {
            return;
        }
        
        let end = Pos2::new(
            start.x + angle.cos() * length,
            start.y + angle.sin() * length,
        );
        
        // Skip if both points are outside the clip rect
        if !clip_rect.contains(start) && !clip_rect.contains(end) {
            // Check if line might still cross the rect
            let line_rect = Rect::from_two_pos(start, end);
            if !line_rect.intersects(clip_rect) {
                return;
            }
        }
        
        // Vary color based on depth
        let depth_factor = depth as f32 / self.config.base_depth as f32;
        let line_color = Color32::from_rgba_unmultiplied(
            (color.r() as f32 * brightness * depth_factor) as u8,
            (color.g() as f32 * brightness * depth_factor) as u8,
            (color.b() as f32 * brightness * depth_factor) as u8,
            (255.0 * depth_factor) as u8,
        );
        
        let stroke_width = (depth as f32 * 0.1).max(0.5);
        painter.line_segment([start, end], Stroke::new(stroke_width, line_color));
        
        // Audio-reactive branch angles
        let angle_mod = self.audio.smooth_mid * 0.2;
        
        // Recursive branches
        let new_length = length * (0.65 + self.audio.smooth_treble * 0.1);
        
        self.draw_branch(painter, end, new_length, angle - branch_angle + angle_mod, 
            branch_angle * 0.95, depth - 1, brightness, color, clip_rect);
        self.draw_branch(painter, end, new_length, angle + branch_angle - angle_mod, 
            branch_angle * 0.95, depth - 1, brightness, color, clip_rect);
    }
    
    fn draw_spectrum(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter();
        let bar_count = 64;
        let bar_width = rect.width() / bar_count as f32;
        
        for i in 0..bar_count {
            let idx = i * self.audio.frequency_data.len() / bar_count;
            let value = if idx < self.audio.frequency_data.len() {
                self.audio.frequency_data[idx] as f32 / 255.0
            } else {
                0.0
            };
            
            let height = value * rect.height();
            let x = rect.left() + i as f32 * bar_width;
            let bar_rect = Rect::from_min_max(
                Pos2::new(x, rect.bottom() - height),
                Pos2::new(x + bar_width - 1.0, rect.bottom()),
            );
            
            let hue = i as f32 / bar_count as f32;
            let color = hsl_to_rgb(hue, 0.8, 0.5);
            painter.rect_filled(bar_rect, 0.0, color);
        }
    }
    
    fn draw_waveform(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter();
        
        let points: Vec<Pos2> = self.audio.time_data.iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = rect.left() + (i as f32 / self.audio.time_data.len() as f32) * rect.width();
                let y = rect.center().y + ((v as f32 - 128.0) / 128.0) * rect.height() * 0.5;
                Pos2::new(x, y)
            })
            .collect();
        
        if points.len() > 1 {
            for i in 0..points.len() - 1 {
                let hue = i as f32 / points.len() as f32;
                let color = hsl_to_rgb(hue, 0.7, 0.6);
                painter.line_segment([points[i], points[i + 1]], Stroke::new(2.0, color));
            }
        }
    }
    
    fn draw_particles(&mut self, painter: &egui::Painter, center: Pos2) {
        for p in &mut self.particles {
            let alpha = (p.life * 255.0) as u8;
            let color = Color32::from_rgba_unmultiplied(p.color.r(), p.color.g(), p.color.b(), alpha);
            let pos = Pos2::new(
                center.x + (p.pos.x - 400.0),
                center.y + (p.pos.y - 300.0),
            );
            painter.circle_filled(pos, p.size * p.life, color);
        }
    }
    
    fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("🎵 Music Visualizer");
            ui.separator();
            
            // ===== PLAYLIST SECTION =====
            ui.collapsing("🎶 Playlist", |ui| {
                // Add music button
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Music").clicked() {
                        self.trigger_file_input();
                    }
                    
                    // Shuffle button
                    let shuffle_text = if self.playlist.is_shuffled { "🔀 On" } else { "🔀 Off" };
                    if ui.button(shuffle_text).clicked() {
                        self.playlist.is_shuffled = !self.playlist.is_shuffled;
                        if self.playlist.is_shuffled {
                            self.playlist.shuffle_playlist();
                        }
                    }
                });
                
                ui.label("Supported: MP3, WAV, OGG, FLAC, AAC, M4A");
                ui.add_space(4.0);
                
                // Current track info and progress
                if let Some(track) = self.playlist.get_current_track().cloned() {
                    ui.group(|ui| {
                        ui.label(format!("🎵 {}", track.name));
                        ui.label(format!("Format: {}", track.file_type.to_uppercase()));
                        
                        // Progress bar
                        let progress = self.playlist.get_progress();
                        let current_time = PlaylistState::format_time(self.playlist.current_time);
                        let total_time = PlaylistState::format_time(self.playlist.duration);
                        
                        ui.horizontal(|ui| {
                            ui.label(&current_time);
                            let progress_response = ui.add(
                                egui::ProgressBar::new(progress)
                                    .desired_width(ui.available_width() - 50.0)
                            );
                            
                            // Click on progress bar to seek
                            if progress_response.clicked() {
                                if let Some(pos) = progress_response.interact_pointer_pos() {
                                    let rect = progress_response.rect;
                                    let seek_ratio = (pos.x - rect.left()) / rect.width();
                                    let seek_time = seek_ratio as f64 * self.playlist.duration;
                                    self.seek_to(seek_time);
                                }
                            }
                            
                            ui.label(&total_time);
                        });
                        
                        // Playback controls
                        ui.horizontal(|ui| {
                            // Previous
                            if ui.button("⏮").clicked() {
                                self.play_previous();
                            }
                            
                            // Play/Pause
                            let play_pause_icon = if self.playlist.is_playing { "⏸" } else { "▶" };
                            if ui.button(play_pause_icon).clicked() {
                                self.toggle_playback();
                            }
                            
                            // Next
                            if ui.button("⏭").clicked() {
                                self.play_next();
                            }
                            
                            // Stop
                            if ui.button("⏹").clicked() {
                                self.stop_playback();
                            }
                        });
                        
                        // Volume control
                        ui.horizontal(|ui| {
                            ui.label("🔊");
                            if ui.add(egui::Slider::new(&mut self.playlist.volume, 0.0..=1.0).show_value(false)).changed() {
                                self.update_volume();
                            }
                        });
                    });
                } else {
                    ui.colored_label(Color32::GRAY, "No track selected");
                }
                
                ui.add_space(8.0);
                
                // Track list
                if !self.playlist.tracks.is_empty() {
                    ui.label(format!("Tracks ({}):", self.playlist.tracks.len()));
                    
                    let mut track_to_play: Option<usize> = None;
                    let mut track_to_remove: Option<usize> = None;
                    
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .id_salt("playlist_scroll")
                        .show(ui, |ui| {
                            for (idx, track) in self.playlist.tracks.iter().enumerate() {
                                let is_current = self.playlist.current_index == Some(idx);
                                let bg_color = if is_current {
                                    Color32::from_rgba_unmultiplied(100, 200, 255, 30)
                                } else {
                                    Color32::TRANSPARENT
                                };
                                
                                egui::Frame::new()
                                    .fill(bg_color)
                                    .inner_margin(4.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Track number/playing indicator
                                            if is_current && self.playlist.is_playing {
                                                ui.label("▶");
                                            } else {
                                                ui.label(format!("{}.", idx + 1));
                                            }
                                            
                                            // Track name (clickable)
                                            let track_label = egui::Label::new(&track.name).sense(egui::Sense::click());
                                            if ui.add(track_label).clicked() {
                                                track_to_play = Some(idx);
                                            }
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                // Remove button
                                                if ui.small_button("✕").clicked() {
                                                    track_to_remove = Some(idx);
                                                }
                                                
                                                // Duration
                                                ui.label(PlaylistState::format_time(track.duration));
                                            });
                                        });
                                    });
                            }
                        });
                    
                    // Handle track actions after iteration
                    if let Some(idx) = track_to_play {
                        self.play_track(idx);
                    }
                    if let Some(idx) = track_to_remove {
                        self.remove_track(idx);
                    }
                    
                    // Clear all button
                    ui.add_space(4.0);
                    if ui.button("🗑 Clear Playlist").clicked() {
                        self.clear_playlist();
                    }
                }
            });
            
            ui.separator();
            
            // Audio source
            ui.horizontal(|ui| {
                ui.label("Audio Source:");
                if ui.selectable_label(self.demo_mode, "Demo").clicked() {
                    self.demo_mode = true;
                }
                if ui.selectable_label(!self.demo_mode, "Microphone").clicked() {
                    self.demo_mode = false;
                    self.try_init_audio();
                }
            });
            
            if !self.demo_mode && !*self.audio_initialized.borrow() {
                ui.colored_label(Color32::YELLOW, "⏳ Initializing microphone...");
            }
            
            ui.separator();
            
            // Audio levels
            ui.collapsing("📊 Audio Levels", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Bass:");
                    ui.add(egui::ProgressBar::new(self.audio.smooth_bass).show_percentage());
                });
                ui.horizontal(|ui| {
                    ui.label("Mid:");
                    ui.add(egui::ProgressBar::new(self.audio.smooth_mid).show_percentage());
                });
                ui.horizontal(|ui| {
                    ui.label("Treble:");
                    ui.add(egui::ProgressBar::new(self.audio.smooth_treble).show_percentage());
                });
                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    ui.add(egui::ProgressBar::new(self.audio.smooth_volume).show_percentage());
                });
                if self.audio.beat {
                    ui.colored_label(Color32::from_rgb(255, 100, 100), "🥁 BEAT!");
                }
            });
            
            ui.separator();
            
            // Fractal settings
            ui.collapsing("🌿 Fractal Settings", |ui| {
                ui.add(egui::Slider::new(&mut self.config.base_zoom, 0.1..=0.5).text("Zoom"));
                ui.add(egui::Slider::new(&mut self.config.base_width, 0.5..=2.0).text("Width"));
                ui.add(egui::Slider::new(&mut self.config.base_depth, 3..=12).text("Depth"));
                ui.add(egui::Slider::new(&mut self.config.base_brightness, 0.3..=1.0).text("Brightness"));
            });
            
            // Audio reactivity
            ui.collapsing("🎛️ Audio Reactivity", |ui| {
                ui.add(egui::Slider::new(&mut self.config.zoom_bass_mult, 0.0..=1.0).text("Bass → Zoom"));
                ui.add(egui::Slider::new(&mut self.config.width_bass_mult, 0.0..=1.0).text("Bass → Width"));
                ui.add(egui::Slider::new(&mut self.config.depth_complexity_mult, 0.0..=8.0).text("Complexity → Depth"));
                ui.add(egui::Slider::new(&mut self.config.brightness_treble_mult, 0.0..=1.0).text("Treble → Brightness"));
            });
            
            // Animation
            ui.collapsing("✨ Animation", |ui| {
                ui.checkbox(&mut self.config.auto_rotate, "Auto Rotate");
                ui.add(egui::Slider::new(&mut self.config.rotation_speed, 0.0..=2.0).text("Rotation Speed"));
                ui.checkbox(&mut self.config.pulse_on_beat, "Pulse on Beat");
                ui.checkbox(&mut self.config.color_cycle, "Color Cycle");
                ui.add(egui::Slider::new(&mut self.config.color_cycle_speed, 0.01..=0.5).text("Color Speed"));
            });
            
            // Display options
            ui.collapsing("🖥️ Display", |ui| {
                ui.checkbox(&mut self.show_spectrum, "Show Spectrum");
                ui.checkbox(&mut self.show_waveform, "Show Waveform");
                ui.add(egui::Slider::new(&mut self.config.glow_intensity, 0.0..=1.0).text("Glow"));
            });
            
            ui.separator();
            
            // Reset button
            if ui.button("🔄 Reset Settings").clicked() {
                self.config = VisualizerConfig::default();
            }
        });
    }
    
    // ===== PLAYLIST METHODS =====
    
    fn process_pending_tracks(&mut self) {
        // Process any tracks added from file input
        let pending = self.pending_tracks.borrow().clone();
        if !pending.is_empty() {
            for (name, file_type, _url) in pending {
                self.playlist.tracks.push(PlaylistTrack {
                    name,
                    duration: 0.0, // Will be updated when metadata loads
                    file_type,
                });
            }
            self.pending_tracks.borrow_mut().clear();
            
            // Auto-play first track if nothing is playing
            if self.playlist.current_index.is_none() && !self.playlist.tracks.is_empty() {
                self.playlist.current_index = Some(0);
            }
        }
    }
    
    fn trigger_file_input(&self) {
        // Create a hidden file input element and trigger it
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                // Check if input already exists
                let input = document
                    .get_element_by_id("music_file_input")
                    .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .unwrap_or_else(|| {
                        let input = document
                            .create_element("input")
                            .unwrap()
                            .dyn_into::<web_sys::HtmlInputElement>()
                            .unwrap();
                        input.set_type("file");
                        input.set_id("music_file_input");
                        input.set_accept("audio/*,.mp3,.wav,.ogg,.flac,.aac,.m4a");
                        input.set_multiple(true);
                        input.style().set_property("display", "none").ok();
                        document.body().unwrap().append_child(&input).ok();
                        input
                    });
                
                // Set up the change handler
                let audio_element = self.audio_element.clone();
                let audio_context = self.audio_context.clone();
                let analyser_node = self.analyser_node.clone();
                let audio_data = self.audio_data.clone();
                let file_audio_initialized = self.file_audio_initialized.clone();
                let pending_tracks = self.pending_tracks.clone();
                
                let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    let input = event.target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                    
                    if let Some(input) = input {
                        if let Some(files) = input.files() {
                            let window = web_sys::window().unwrap();
                            let document = window.document().unwrap();
                            
                            for i in 0..files.length() {
                                if let Some(file) = files.get(i) {
                                    let file_name = file.name();
                                    let file_type = file_name.split('.').last()
                                        .unwrap_or("unknown").to_lowercase();
                                    
                                    // Create object URL for the file
                                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                                        // Add to pending tracks
                                        pending_tracks.borrow_mut().push((
                                            file_name.clone(),
                                            file_type.clone(),
                                            url.clone(),
                                        ));
                                        
                                        // Create or get audio element (only for first file or if none exists)
                                        if audio_element.borrow().is_none() || i == 0 {
                                            let audio = document
                                                .get_element_by_id("visualizer_audio")
                                                .and_then(|el| el.dyn_into::<web_sys::HtmlAudioElement>().ok())
                                                .unwrap_or_else(|| {
                                                    let audio = document
                                                        .create_element("audio")
                                                        .unwrap()
                                                        .dyn_into::<web_sys::HtmlAudioElement>()
                                                        .unwrap();
                                                    audio.set_id("visualizer_audio");
                                                    document.body().unwrap().append_child(&audio).ok();
                                                    audio
                                                });
                                            
                                            audio.set_src(&url);
                                            *audio_element.borrow_mut() = Some(audio.clone());
                                            
                                            // Initialize audio context if needed
                                            if audio_context.borrow().is_none() {
                                                if let Ok(ctx) = web_sys::AudioContext::new() {
                                                    if let Ok(analyser) = ctx.create_analyser() {
                                                        analyser.set_fft_size(512);
                                                        analyser.set_smoothing_time_constant(0.8);
                                                        
                                                        if let Ok(source) = ctx.create_media_element_source(&audio) {
                                                            source.connect_with_audio_node(&analyser).ok();
                                                            analyser.connect_with_audio_node(&ctx.destination()).ok();
                                                            
                                                            *analyser_node.borrow_mut() = Some(analyser);
                                                            *audio_context.borrow_mut() = Some(ctx);
                                                            *file_audio_initialized.borrow_mut() = true;
                                                            
                                                            // Start polling audio data
                                                            let analyser_for_poll = analyser_node.clone();
                                                            let audio_data_for_poll = audio_data.clone();
                                                            
                                                            let poll_callback = Closure::wrap(Box::new(move || {
                                                                if let Some(ref analyser) = *analyser_for_poll.borrow() {
                                                                    let freq_len = analyser.frequency_bin_count() as usize;
                                                                    let time_len = analyser.fft_size() as usize;
                                                                    let mut freq_data = vec![0u8; freq_len];
                                                                    let mut time_data = vec![0u8; time_len];
                                                                    
                                                                    analyser.get_byte_frequency_data(&mut freq_data);
                                                                    analyser.get_byte_time_domain_data(&mut time_data);
                                                                    
                                                                    *audio_data_for_poll.borrow_mut() = (freq_data, time_data);
                                                                }
                                                            }) as Box<dyn Fn()>);
                                                            
                                                            window.set_interval_with_callback_and_timeout_and_arguments_0(
                                                                poll_callback.as_ref().unchecked_ref(),
                                                                16,
                                                            ).ok();
                                                            poll_callback.forget();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        web_sys::console::log_1(&format!("Added track: {} ({})", file_name, file_type).into());
                                    }
                                }
                            }
                        }
                        input.set_value(""); // Reset for next use
                    }
                }) as Box<dyn FnMut(web_sys::Event)>);
                
                input.set_onchange(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
                
                input.click();
            }
        }
    }
    
    fn play_track(&mut self, index: usize) {
        if index >= self.playlist.tracks.len() {
            return;
        }
        
        self.playlist.current_index = Some(index);
        self.playlist.is_playing = true;
        self.demo_mode = false;
        
        if let Some(ref audio) = *self.audio_element.borrow() {
            audio.play().ok();
        }
    }
    
    fn toggle_playback(&mut self) {
        if let Some(ref audio) = *self.audio_element.borrow() {
            if self.playlist.is_playing {
                audio.pause().ok();
                self.playlist.is_playing = false;
            } else {
                audio.play().ok();
                self.playlist.is_playing = true;
            }
        }
    }
    
    fn stop_playback(&mut self) {
        if let Some(ref audio) = *self.audio_element.borrow() {
            audio.pause().ok();
            audio.set_current_time(0.0);
        }
        self.playlist.is_playing = false;
        self.playlist.current_time = 0.0;
    }
    
    fn play_next(&mut self) {
        if let Some(next_idx) = self.playlist.get_next_index() {
            self.play_track(next_idx);
        }
    }
    
    fn play_previous(&mut self) {
        // If more than 3 seconds into song, restart it
        if self.playlist.current_time > 3.0 {
            self.seek_to(0.0);
            return;
        }
        
        if let Some(prev_idx) = self.playlist.get_prev_index() {
            self.play_track(prev_idx);
        }
    }
    
    fn seek_to(&mut self, time: f64) {
        if let Some(ref audio) = *self.audio_element.borrow() {
            audio.set_current_time(time);
            self.playlist.current_time = time;
        }
    }
    
    fn update_volume(&self) {
        if let Some(ref audio) = *self.audio_element.borrow() {
            audio.set_volume(self.playlist.volume as f64);
        }
    }
    
    fn remove_track(&mut self, index: usize) {
        if index < self.playlist.tracks.len() {
            self.playlist.tracks.remove(index);
            
            // Update current index if needed
            if let Some(current) = self.playlist.current_index {
                if index == current {
                    self.stop_playback();
                    self.playlist.current_index = None;
                } else if index < current {
                    self.playlist.current_index = Some(current - 1);
                }
            }
            
            // Update shuffle order
            if self.playlist.is_shuffled {
                self.playlist.shuffle_playlist();
            }
        }
    }
    
    fn clear_playlist(&mut self) {
        self.stop_playback();
        self.playlist.tracks.clear();
        self.playlist.current_index = None;
        self.playlist.shuffle_order.clear();
    }
    
    fn update_playback_state(&mut self) {
        let (current_time, duration, ended) = {
            if let Some(ref audio) = *self.audio_element.borrow() {
                let ct = audio.current_time();
                let dur = audio.duration();
                let track_ended = !dur.is_nan() && ct >= dur - 0.1;
                (ct, dur, track_ended)
            } else {
                return;
            }
        };
        
        self.playlist.current_time = current_time;
        self.playlist.duration = duration;
        
        // Check if track ended
        if ended {
            self.play_next();
        }
    }
}

impl eframe::App for MusicVisualizerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt);
        
        // Process any pending tracks from file input
        self.process_pending_tracks();
        
        // Update playback state from audio element
        self.update_playback_state();
        
        // Update audio and animation
        self.update_audio(dt);
        self.update_animation(dt);
        
        // Request repaint for animation
        ctx.request_repaint();
        
        // Side panel for settings
        if self.show_settings {
            egui::SidePanel::left("settings_panel")
                .resizable(true)
                .default_width(280.0)
                .show(ctx, |ui| {
                    self.draw_settings_panel(ui);
                });
        }
        
        // Main visualization area
        egui::CentralPanel::default().show(ctx, |ui| {
            let _available = ui.available_rect_before_wrap();
            
            // Toggle settings button
            ui.horizontal(|ui| {
                if ui.button(if self.show_settings { "◀ Hide" } else { "▶ Show" }).clicked() {
                    self.show_settings = !self.show_settings;
                }
                ui.label(format!("FPS: {:.0}", 1.0 / dt));
            });
            
            let remaining = ui.available_rect_before_wrap();
            
            // Layout visualization areas
            let spectrum_height = if self.show_spectrum { 80.0 } else { 0.0 };
            let waveform_height = if self.show_waveform { 60.0 } else { 0.0 };
            let bottom_ui_height = spectrum_height + waveform_height;
            
            let fractal_rect = Rect::from_min_max(
                remaining.min,
                Pos2::new(remaining.max.x, remaining.max.y - bottom_ui_height),
            );
            
            // Draw fractal
            let fractal_response = ui.allocate_rect(fractal_rect, egui::Sense::hover());
            if fractal_response.hovered() {
                // Could add mouse interaction here
            }
            self.draw_fractal(ui, fractal_rect);
            
            // Draw particles
            let painter = ui.painter();
            self.draw_particles(painter, fractal_rect.center());
            
            // Draw spectrum analyzer
            if self.show_spectrum {
                let spectrum_rect = Rect::from_min_max(
                    Pos2::new(remaining.min.x, remaining.max.y - bottom_ui_height),
                    Pos2::new(remaining.max.x, remaining.max.y - waveform_height),
                );
                ui.allocate_rect(spectrum_rect, egui::Sense::hover());
                self.draw_spectrum(ui, spectrum_rect);
            }
            
            // Draw waveform
            if self.show_waveform {
                let waveform_rect = Rect::from_min_max(
                    Pos2::new(remaining.min.x, remaining.max.y - waveform_height),
                    remaining.max,
                );
                ui.allocate_rect(waveform_rect, egui::Sense::hover());
                self.draw_waveform(ui, waveform_rect);
            }
        });
    }
}

// HSL to RGB color conversion
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = match (h * 6.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

// Initialize Web Audio API
async fn init_web_audio(audio_data: Rc<RefCell<(Vec<u8>, Vec<u8>)>>) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;
    
    // Request microphone access
    let constraints = web_sys::MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    constraints.set_video(&JsValue::FALSE);
    
    let promise = media_devices.get_user_media_with_constraints(&constraints)?;
    let stream: web_sys::MediaStream = wasm_bindgen_futures::JsFuture::from(promise).await?.into();
    
    // Create audio context and analyser
    let audio_ctx = web_sys::AudioContext::new()?;
    let analyser = audio_ctx.create_analyser()?;
    analyser.set_fft_size(512);
    analyser.set_smoothing_time_constant(0.8);
    
    let source = audio_ctx.create_media_stream_source(&stream)?;
    source.connect_with_audio_node(&analyser)?;
    
    // Store analyser for later use (simplified - in production would use more robust pattern)
    let freq_data_length = analyser.frequency_bin_count() as usize;
    let time_data_length = analyser.fft_size() as usize;
    
    // Set up animation frame callback
    let audio_data_clone = audio_data.clone();
    let analyser_clone = analyser.clone();
    
    let callback = Closure::wrap(Box::new(move || {
        let mut freq_data = vec![0u8; freq_data_length];
        let mut time_data = vec![0u8; time_data_length];
        
        analyser_clone.get_byte_frequency_data(&mut freq_data);
        analyser_clone.get_byte_time_domain_data(&mut time_data);
        
        *audio_data_clone.borrow_mut() = (freq_data, time_data);
    }) as Box<dyn Fn()>);
    
    // Start polling audio data
    let window_clone = window.clone();
    fn request_frame(window: &web_sys::Window, callback: &Closure<dyn Fn()>) {
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            16, // ~60fps
        ).ok();
    }
    
    request_frame(&window_clone, &callback);
    callback.forget(); // Leak the closure to keep it alive
    
    Ok(())
}

// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no global window exists")
            .document()
            .expect("should have a document on window");
        
        let canvas = document
            .get_element_by_id("music_visualizer_canvas")
            .expect("no canvas element with id 'music_visualizer_canvas'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element with id 'music_visualizer_canvas' is not a canvas");
        
        let web_options = eframe::WebOptions::default();
        
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(MusicVisualizerApp::new(cc)))),
            )
            .await
            .expect("Failed to start eframe");
    });
    
    Ok(())
}
