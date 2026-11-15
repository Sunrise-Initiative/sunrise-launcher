use eframe::egui::{
    self, Button, Color32, Context, CornerRadius, Rect, Response, Stroke, TextureHandle, Ui,
};
use egui::epaint::{Mesh, RectShape};

use crate::catalog::PRODUCTS;
use crate::config::{Mode, Theme};

#[derive(Clone, Copy)]
pub struct Palette {
    pub page: Color32,
    pub rail: Color32,
    pub card: Color32,
    pub control: Color32,
    pub border: Color32,
    pub selected: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub feature: Color32,
    pub aero: bool,
    pub mode: Mode,
}

impl Palette {
    pub fn new(theme: Theme, mode: Mode) -> Self {
        let colors = match (theme, mode) {
            (Theme::Modern, Mode::Dark) => [
                0x191423, 0x20182b, 0x251c31, 0x332641, 0x594462, 0xfff4f3, 0xc8b9d3, 0xffbf8d,
                0x321b1d, 0x3e2436,
            ],
            (Theme::Modern, Mode::Light) => [
                0xf9f3eb, 0xf1e8e0, 0xfffbf7, 0xf5e8e0, 0xc2a79f, 0x35232d, 0x6a505d, 0x923c33,
                0xffffff, 0xf3d8cc,
            ],
            (Theme::Aero, Mode::Dark) => [
                0x10282a, 0x112e32, 0x18373b, 0x233f43, 0x618789, 0xf3fff6, 0xc5ded2, 0xb3e89a,
                0x172c15, 0x123029,
            ],
            (Theme::Aero, Mode::Light) => [
                0xdeeff9, 0xb8dff3, 0xeaf6fd, 0xe0f1fa, 0x82b1cc, 0x12294b, 0x31536c, 0x075a9e,
                0xffffff, 0xe1f0f8,
            ],
        };
        let [
            page,
            rail,
            card,
            control,
            border,
            text,
            muted,
            accent,
            on_accent,
            feature,
        ] = colors.map(rgb);
        let selected = match (theme, mode) {
            (Theme::Modern, Mode::Dark) => Color32::from_rgb(91, 55, 86),
            (Theme::Modern, Mode::Light) => Color32::from_rgb(238, 204, 194),
            (Theme::Aero, Mode::Dark) => Color32::from_rgb(54, 94, 48),
            (Theme::Aero, Mode::Light) => Color32::from_rgb(166, 216, 248),
        };
        Self {
            page,
            rail,
            card,
            control,
            border,
            selected,
            text,
            muted,
            accent,
            on_accent,
            feature,
            aero: theme == Theme::Aero,
            mode,
        }
    }

    pub fn apply(self, ctx: &Context) {
        let (theme, window_theme, mut visuals) = match self.mode {
            Mode::Dark => (
                egui::Theme::Dark,
                egui::SystemTheme::Dark,
                egui::Visuals::dark(),
            ),
            Mode::Light => (
                egui::Theme::Light,
                egui::SystemTheme::Light,
                egui::Visuals::light(),
            ),
        };
        visuals.override_text_color = Some(self.text);
        visuals.panel_fill = self.page;
        visuals.window_fill = self.card;
        visuals.extreme_bg_color = self.rail;
        visuals.hyperlink_color = self.accent;
        visuals.selection.bg_fill = self.selected;
        visuals.selection.stroke = Stroke::new(1.0, self.text);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.weak_bg_fill = self.control;
            widget.bg_fill = self.control;
            widget.bg_stroke = Stroke::new(1.0, self.border);
            widget.fg_stroke = Stroke::new(1.0, self.text);
            widget.corner_radius = CornerRadius::same(7);
        }
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, self.accent);
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, self.accent);
        ctx.set_theme(theme);
        ctx.set_visuals_of(theme, visuals);
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(window_theme));
    }

    pub fn panel_fill(self, base: Color32) -> Color32 {
        if self.aero {
            Color32::from_rgba_unmultiplied(
                base.r(),
                base.g(),
                base.b(),
                if self.mode == Mode::Light { 228 } else { 238 },
            )
        } else {
            base
        }
    }
}

const BLUE_GLASS: [u32; 6] = [0x2875af, 0x226caa, 0x175d9d, 0x074a95, 0x08549e, 0x1b659e];
const PEARL_GLASS: [u32; 6] = [0xf7fcff, 0xe8f6ff, 0xdceefa, 0xc6e4f6, 0xc9e7f7, 0xd9effa];
const GREEN_GLASS: [u32; 6] = [0xb9ed98, 0xa8dc8b, 0x9ed27e, 0x7eba63, 0x8ecb73, 0xa3d98b];
const DARK_GLASS: [u32; 6] = [0x355d60, 0x2c4f53, 0x22464a, 0x19393d, 0x204247, 0x284d51];

pub struct Aero {
    pub lake: Option<TextureHandle>,
    pub covers: [Option<TextureHandle>; PRODUCTS.len()],
    pub navigation: [Option<TextureHandle>; 4],
    blue: TextureHandle,
    pearl: TextureHandle,
    green: TextureHandle,
    dark: TextureHandle,
}

impl Aero {
    pub fn new(ctx: &Context) -> Self {
        #[cfg(windows)]
        load_windows_fonts(ctx);

        let glass = |name, colors: [u32; 6]| {
            let pixels = colors.map(rgb).to_vec();
            ctx.load_texture(
                name,
                egui::ColorImage::new([1, pixels.len()], pixels),
                egui::TextureOptions::LINEAR,
            )
        };
        let icons: [&[u8]; 4] = [
            include_bytes!("../assets/aero-light/oxygen/website.png"),
            include_bytes!("../assets/aero-light/oxygen/folder.png"),
            include_bytes!("../assets/aero-light/oxygen/view-refresh.png"),
            include_bytes!("../assets/aero-light/oxygen/preferences-system.png"),
        ];
        Self {
            lake: texture(
                ctx,
                "Aero: lago",
                include_bytes!("../assets/aero-light/lake.webp"),
            ),
            covers: std::array::from_fn(|index| {
                let product = &PRODUCTS[index];
                texture(ctx, &format!("Aero: {}", product.name), product.aero_cover)
            }),
            navigation: icons.map(|bytes| texture(ctx, "Ícone de navegação Aero", bytes)),
            blue: glass("Vidro azul", BLUE_GLASS),
            pearl: glass("Vidro perolado", PEARL_GLASS),
            green: glass("Vidro verde", GREEN_GLASS),
            dark: glass("Vidro verde-petróleo", DARK_GLASS),
        }
    }

    pub fn typography(ui: &mut Ui) {
        let regular = aero_font(ui, false);
        let bold = aero_font(ui, true);
        let styles = &mut ui.style_mut().text_styles;
        for style in [
            egui::TextStyle::Body,
            egui::TextStyle::Button,
            egui::TextStyle::Small,
        ] {
            if let Some(font) = styles.get_mut(&style) {
                font.family = regular.clone();
            }
        }
        styles.insert(egui::TextStyle::Heading, egui::FontId::new(28.0, bold));
    }

    pub fn button(
        &self,
        ui: &mut Ui,
        widget: Button<'_>,
        size: [f32; 2],
        selected: bool,
        palette: Palette,
    ) -> Response {
        let background = ui.painter().add(egui::Shape::Noop);
        let response = ui.add_sized(size, widget.fill(Color32::TRANSPARENT).stroke(Stroke::NONE));
        if ui.is_rect_visible(response.rect) {
            let hover = animate_feedback(
                ui,
                response.id.with("hover"),
                response.hovered() && !ui.input(|input| input.any_touches()),
            );
            let press = animate_feedback(
                ui,
                response.id.with("press"),
                response.is_pointer_button_down_on(),
            );
            ui.painter().set(
                background,
                self.button_surface(
                    response.rect,
                    selected,
                    hover.max(f32::from(response.has_focus())),
                    press,
                    palette,
                ),
            );
        }
        response
    }

    pub fn button_surface(
        &self,
        rect: Rect,
        selected: bool,
        highlight: f32,
        press: f32,
        palette: Palette,
    ) -> egui::Shape {
        let radius = CornerRadius::same(8);
        let (fill, edge, sheen_alpha, sheen_brighten) = match (palette.mode, selected) {
            (Mode::Light, true) => (&self.blue, rgb(0x185a8c), 172.0, 64.0),
            (Mode::Light, false) => (&self.pearl, rgb(0x83adc8), 172.0, 64.0),
            (Mode::Dark, true) => (&self.green, rgb(0x68975b), 124.0, 54.0),
            (Mode::Dark, false) => (&self.dark, palette.border, 66.0, 48.0),
        };
        let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
        let tint = Color32::from_gray((255.0 - 24.0 * press) as u8);
        let face = RectShape::filled(rect, radius, tint).with_texture(fill.id(), uv);
        let rim = RectShape::stroke(
            rect,
            radius,
            Stroke::new(1.0, edge),
            egui::StrokeKind::Inside,
        );
        let sheen = RectShape::stroke(
            rect.shrink(2.0),
            CornerRadius::same(6),
            Stroke::new(
                1.0,
                Color32::from_white_alpha((sheen_alpha + sheen_brighten * highlight) as u8),
            ),
            egui::StrokeKind::Inside,
        );
        let focus_rim = RectShape::stroke(
            rect,
            radius,
            Stroke::new(2.0, palette.accent.gamma_multiply(highlight)),
            egui::StrokeKind::Inside,
        );
        egui::Shape::Vec(vec![
            face.into(),
            rim.into(),
            sheen.into(),
            focus_rim.into(),
        ])
    }
}

fn font_family(bold: bool) -> egui::FontFamily {
    egui::FontFamily::Name(if bold {
        "Aero emphasized".into()
    } else {
        "Aero text".into()
    })
}

pub fn aero_font(ui: &Ui, bold: bool) -> egui::FontFamily {
    let family = font_family(bold);
    if ui.fonts(|fonts| fonts.definitions().families.contains_key(&family)) {
        family
    } else {
        egui::FontFamily::Proportional
    }
}

#[cfg(windows)]
fn load_windows_fonts(ctx: &Context) {
    let Some(root) = std::env::var_os("SystemRoot") else {
        return;
    };
    let directory = std::path::PathBuf::from(root).join("Fonts");
    let mut fonts = egui::FontDefinitions::default();
    let fallback = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (filename, bold) in [("segoeui.ttf", false), ("segoeuib.ttf", true)] {
        if let Ok(bytes) = std::fs::read(directory.join(filename)) {
            fonts.font_data.insert(
                filename.to_owned(),
                egui::FontData::from_owned(bytes).into(),
            );
            let family = std::iter::once(filename.to_owned())
                .chain(fallback.iter().cloned())
                .collect();
            fonts.families.insert(font_family(bold), family);
        }
    }
    ctx.set_fonts(fonts);
}

fn rgb(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

pub fn texture(ctx: &Context, name: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let rgba = image::load_from_memory(bytes).ok()?.into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    Some(ctx.load_texture(name, image, egui::TextureOptions::LINEAR))
}

pub fn cover(ui: &Ui, bounds: Rect, texture: &TextureHandle, corners: CornerRadius) {
    cover_zoom(ui, bounds, texture, corners, 1.0);
}

pub fn cover_zoom(
    ui: &Ui,
    bounds: Rect,
    texture: &TextureHandle,
    corners: CornerRadius,
    zoom: f32,
) {
    let ratio = bounds.aspect_ratio() / texture.aspect_ratio();
    let uv_size = if ratio > 1.0 {
        egui::vec2(1.0, 1.0 / ratio)
    } else {
        egui::vec2(ratio, 1.0)
    };
    let uv = Rect::from_center_size(egui::pos2(0.5, 0.5), uv_size / zoom);
    ui.painter()
        .add(RectShape::filled(bounds, corners, Color32::WHITE).with_texture(texture.id(), uv));
}

pub fn animate_feedback(ui: &Ui, id: egui::Id, active: bool) -> f32 {
    let duration = ui.style().animation_time;
    if duration <= 0.0 {
        return f32::from(active);
    }
    ui.ctx().animate_bool_with_time_and_easing(
        id,
        active,
        duration,
        egui::emath::easing::sin_in_out,
    )
}

pub fn mode_transition(ui: &Ui, target: Mode, reduce_motion: bool) -> (Mode, u8) {
    let progress = ui.ctx().animate_bool_with_time_and_easing(
        ui.id().with("mode-transition"),
        target == Mode::Dark,
        if reduce_motion { 0.0 } else { 0.24 },
        egui::emath::easing::linear,
    );
    let mode = if progress < 0.5 {
        Mode::Light
    } else {
        Mode::Dark
    };
    // Both palettes meet behind the same opaque tint at the midpoint.
    let opacity = ((std::f32::consts::PI * progress).sin() * 255.0).round() as u8;
    (mode, opacity)
}

pub fn gradient(ui: &Ui, bounds: Rect, colors: [Color32; 4]) {
    let mut mesh = Mesh::default();
    for (position, color) in [
        bounds.left_top(),
        bounds.right_top(),
        bounds.right_bottom(),
        bounds.left_bottom(),
    ]
    .into_iter()
    .zip(colors)
    {
        mesh.colored_vertex(position, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn mode_transition_preserves_its_progress_when_reversed() {
        let mut ui = Harness::builder().with_step_dt(0.02).build_ui_state(
            |ui, (target, reduced, frame)| *frame = mode_transition(ui, *target, *reduced),
            (Mode::Light, false, (Mode::Light, 0)),
        );
        assert_eq!(ui.state().2, (Mode::Light, 0));
        ui.state_mut().0 = Mode::Dark;
        ui.step();
        let (mode, opacity) = ui.state().2;
        assert_eq!(mode, Mode::Light);
        assert!((1..255).contains(&opacity));
        for _ in 0..7 {
            ui.step();
        }
        let (mode, before_reverse) = ui.state().2;
        assert_eq!(mode, Mode::Dark);
        assert!(before_reverse > 0);
        ui.state_mut().0 = Mode::Light;
        ui.step();
        assert!(ui.state().2.1.abs_diff(before_reverse) < 70);
        for _ in 0..14 {
            ui.step();
        }
        assert_eq!(ui.state().2, (Mode::Light, 0));
        ui.state_mut().0 = Mode::Dark;
        ui.state_mut().1 = true;
        ui.step();
        assert_eq!(ui.state().2, (Mode::Dark, 0));
    }

    #[test]
    fn feedback_animates_and_reduced_motion_applies_changes_immediately() {
        let mut ui = Harness::builder().with_step_dt(0.025).build_ui_state(
            |ui, (active, reduced, progress)| {
                ui.style_mut().animation_time = if *reduced { 0.0 } else { 0.16 };
                *progress = animate_feedback(ui, egui::Id::new("test-feedback"), *active);
            },
            (false, false, 0.0),
        );
        ui.state_mut().0 = true;
        ui.step();
        let during = ui.state().2;
        assert!(during > 0.0 && during < 1.0, "{during}");
        ui.step();
        let before_reverse = ui.state().2;
        ui.state_mut().0 = false;
        ui.step();
        let after_reverse = ui.state().2;
        assert!(after_reverse < before_reverse && before_reverse - after_reverse < 0.3);
        ui.state_mut().0 = true;
        for _ in 0..8 {
            ui.step();
        }
        assert_eq!(ui.state().2, 1.0);
        ui.state_mut().0 = false;
        ui.step();
        assert!((0.0..1.0).contains(&ui.state().2));
        ui.state_mut().1 = true;
        ui.step();
        assert_eq!(ui.state().2, 0.0);
    }

    fn luminance(color: Color32) -> f32 {
        let linear = |value| {
            let value = f32::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }

    #[test]
    fn aero_glass_gradient_keeps_button_text_legible() {
        for (mode, accent, surface) in [
            (Mode::Light, BLUE_GLASS, PEARL_GLASS),
            (Mode::Dark, GREEN_GLASS, DARK_GLASS),
        ] {
            let p = Palette::new(Theme::Aero, mode);
            for (text, samples) in [(p.on_accent, accent), (p.text, surface)] {
                for sample in samples {
                    for tint in [1.0, 231.0 / 255.0] {
                        let a = luminance(text);
                        let b = luminance(rgb(sample).gamma_multiply(tint));
                        assert!(
                            (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
                            "{mode:?}: {sample:06x}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn text_and_primary_buttons_have_aa_contrast() {
        for theme in [Theme::Modern, Theme::Aero] {
            for mode in [Mode::Light, Mode::Dark] {
                let p = Palette::new(theme, mode);
                let pairs = [
                    (p.text, p.card),
                    (p.muted, p.card),
                    (p.muted, p.rail),
                    (p.muted, p.page),
                    (p.muted, p.feature),
                    (p.text, p.selected),
                    (p.accent, p.page),
                    (p.text, p.control),
                    (p.on_accent, p.accent),
                ];
                for (foreground, background) in pairs {
                    let a = luminance(foreground);
                    let b = luminance(background);
                    assert!(
                        (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
                        "{theme:?}/{mode:?}: {foreground:?} on {background:?}"
                    );
                }
                for backdrop in [Color32::BLACK, Color32::WHITE] {
                    for base in [p.page, p.rail] {
                        let panel = backdrop.blend(p.panel_fill(base));
                        let a = luminance(p.muted);
                        let b = luminance(panel);
                        assert!(
                            (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
                            "{theme:?}/{mode:?}: wallpaper under {base:?}"
                        );
                    }
                }
            }
        }
    }
}
