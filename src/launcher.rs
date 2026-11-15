use std::path::PathBuf;

use eframe::egui::{
    self, Button, Color32, Context, CornerRadius, Frame, Margin, RichText, Stroke, TextureHandle,
    Ui, Vec2,
};

use crate::{
    appearance::{
        Aero, Palette, aero_font, animate_feedback, cover, cover_zoom, gradient, mode_transition,
        texture,
    },
    catalog::{Category, PRODUCTS, WEBSITE},
    config::{self, Mode, Settings, Theme, View},
};

const CATEGORIES: [Category; 4] = [
    Category::Image,
    Category::Video,
    Category::Illustration,
    Category::Documents,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Explore,
    Library,
    Updates,
    Settings,
    Details(usize),
}

pub struct Launcher {
    page: Page,
    back: Page,
    query: String,
    category: Option<Category>,
    settings: Settings,
    visible_appearance: (Theme, Mode),
    path: Option<PathBuf>,
    read_error: Option<String>,
    error: Option<String>,
    logo: Option<TextureHandle>,
    dew: Option<TextureHandle>,
    aero: Option<Aero>,
    covers: [Option<TextureHandle>; PRODUCTS.len()],
    icons: [Option<TextureHandle>; PRODUCTS.len()],
}

impl Launcher {
    pub fn new(ctx: &Context, path: Option<PathBuf>) -> Self {
        let loaded = path.as_deref().map_or_else(
            || Err(std::io::Error::other("pasta de configuração indisponível")),
            config::read,
        );
        let (settings, read_error) = match loaded {
            Ok(settings) => (settings, None),
            Err(error) => (
                Settings::default(),
                Some(format!("Preferências indisponíveis. {error}")),
            ),
        };
        let visible_appearance = (settings.theme, settings.mode);
        Palette::new(visible_appearance.0, visible_appearance.1).apply(ctx);
        Self {
            page: Page::Explore,
            back: Page::Explore,
            query: String::new(),
            category: None,
            settings,
            visible_appearance,
            path,
            read_error,
            error: None,
            logo: texture(ctx, "Sunrise", include_bytes!("../assets/sunrise-logo.png")),
            dew: texture(ctx, "Orvalho", include_bytes!("../assets/dew.webp")),
            aero: None,
            covers: std::array::from_fn(|index| {
                texture(ctx, PRODUCTS[index].id, PRODUCTS[index].cover)
            }),
            icons: std::array::from_fn(|index| {
                texture(ctx, PRODUCTS[index].name, PRODUCTS[index].icon)
            }),
        }
    }

    fn change(&mut self, ctx: &Context, edit: impl FnOnce(&mut Settings)) {
        let mut next = self.settings.clone();
        edit(&mut next);
        if next == self.settings {
            return;
        }
        let Some(path) = self.path.as_deref().filter(|_| self.read_error.is_none()) else {
            return;
        };
        match config::save(path, &self.settings, &next) {
            Ok(()) => {
                self.settings = next;
                self.error = None;
                ctx.request_repaint();
            }
            Err(error) => self.error = Some(format!("Não foi possível salvar. {error}")),
        }
    }

    fn navigate(&mut self, page: Page) {
        self.page = page;
        self.error = None;
    }

    fn details(&mut self, index: usize) {
        self.back = self.page;
        self.navigate(Page::Details(index));
    }

    fn browse(&mut self, ui: &Ui, index: usize) {
        let mut dialog = rfd::FileDialog::new().set_title("Vincular executável local");
        if cfg!(windows) {
            dialog = dialog.add_filter("Aplicativo Windows", &["exe"]);
        }
        let Some(path) = dialog.pick_file() else {
            return;
        };
        match config::executable(&path) {
            Ok(path) => self.change(ui.ctx(), |settings| {
                settings.links.insert(PRODUCTS[index].id.into(), path);
            }),
            Err(error) => {
                self.error = Some(format!("Não foi possível vincular o arquivo. {error}"))
            }
        }
    }

    fn launch(&mut self, index: usize) {
        self.error = self
            .settings
            .links
            .get(PRODUCTS[index].id)
            .and_then(|path| config::open(path).err())
            .map(|error| format!("Não foi possível abrir {}. {error}", PRODUCTS[index].name));
    }

    fn navigation(&mut self, ui: &mut Ui, palette: Palette) {
        Frame::new().inner_margin(Margin::same(16)).show(ui, |ui| {
            ui.spacing_mut().button_padding.x = 12.0;
            if !palette.aero {
                self.brand(ui);
                ui.add_space(24.0);
            }
            let pages = [
                ("Explorar", Page::Explore),
                ("Biblioteca", Page::Library),
                ("Atualizações", Page::Updates),
                ("Configurações", Page::Settings),
            ];
            let count = if palette.aero { 4 } else { 3 };
            for (index, &(label, page)) in pages.iter().take(count).enumerate() {
                let active =
                    page == self.page || matches!(self.page, Page::Details(_)) && page == self.back;
                if self.nav_button(ui, palette, index, label, active).clicked() {
                    self.navigate(page);
                }
            }
            if palette.aero {
                return;
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                if self
                    .nav_button(ui, palette, 3, "Configurações", self.page == Page::Settings)
                    .clicked()
                {
                    self.navigate(Page::Settings);
                }
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new(concat!("Versão ", env!("CARGO_PKG_VERSION")))
                        .color(palette.muted)
                        .size(12.0),
                );
            });
        });
    }

    fn brand(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some(logo) = &self.logo {
                ui.image((logo.id(), egui::vec2(27.0, 27.0)));
            }
            ui.label(RichText::new("Sunrise Launcher").strong().size(15.0));
        });
    }

    fn nav_button(
        &self,
        ui: &mut Ui,
        palette: Palette,
        index: usize,
        label: &str,
        active: bool,
    ) -> egui::Response {
        if let Some(aero) = self.aero_assets(palette) {
            let compact = ui.available_width() < 150.0;
            let text = RichText::new(label)
                .size(if compact { 14.0 } else { 16.0 })
                .color(if active {
                    palette.on_accent
                } else {
                    palette.text
                });
            let icon_size = if compact { 18.0 } else { 25.0 };
            let mut button = match &aero.navigation[index] {
                Some(icon) => Button::image_and_text((icon.id(), Vec2::splat(icon_size)), text),
                None => Button::new(text),
            }
            .selected(active);
            let width = ui.available_width();
            if active {
                return aero.button(ui, button, [width, 42.0], true, palette);
            }
            button = button
                .min_size(egui::vec2(width, 42.0))
                .frame_when_inactive(false);
            return ui.add(button);
        }
        let button = Button::new(RichText::new(label).size(15.0))
            .min_size(egui::vec2(ui.available_width(), 38.0))
            .wrap_mode(egui::TextWrapMode::Extend)
            .selected(active);
        let response = ui.add(button);
        if active {
            let rect = response.rect;
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    rect.left_top() + egui::vec2(1.0, 7.0),
                    rect.left_bottom() + egui::vec2(4.0, -7.0),
                ),
                CornerRadius::same(2),
                palette.accent,
            );
        }
        response
    }

    fn aero_assets(&self, palette: Palette) -> Option<&Aero> {
        self.aero.as_ref().filter(|_| palette.aero)
    }

    fn artwork(&self, palette: Palette, index: usize) -> Option<&TextureHandle> {
        self.aero_assets(palette)
            .and_then(|aero| aero.covers[index].as_ref())
            .or(self.covers[index].as_ref())
    }

    fn filter_button(
        &self,
        ui: &mut Ui,
        palette: Palette,
        label: &str,
        selected: bool,
    ) -> egui::Response {
        let aero = self.aero_assets(palette);
        let family = if aero.is_some() {
            aero_font(ui, false)
        } else {
            egui::FontFamily::Proportional
        };
        let font = egui::FontId::new(15.0, family);
        let width = ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(label.to_owned(), font.clone(), palette.text)
                .size()
                .x
        });
        let size = [(width + 24.0).max(70.0), 40.0];
        let mut text = RichText::new(label).font(font);
        if aero.is_some() {
            text = text.color(if selected {
                palette.on_accent
            } else {
                palette.text
            });
        }
        let button = Button::new(text).selected(selected);
        if let Some(aero) = aero {
            aero.button(ui, button, size, selected, palette)
        } else {
            ui.add_sized(size, button)
        }
    }

    fn explore(&mut self, ui: &mut Ui, palette: Palette) {
        ui.horizontal(|ui| {
            title(ui, "Explorar");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                const LABEL: &str = "Buscar aplicativos";
                let width = ui.available_width().min(300.0);
                let radius = CornerRadius::same(12);
                let margin = Margin {
                    left: 32,
                    right: 12,
                    top: 7,
                    bottom: 7,
                };
                ui.visuals_mut().weak_text_color = Some(palette.muted);
                let input = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .id_salt("catalog-search")
                        .hint_text(LABEL)
                        .text_color(palette.text)
                        .vertical_align(egui::Align::Center)
                        .min_size(egui::vec2(width, 38.0))
                        .margin(margin)
                        .frame(
                            Frame::new()
                                .inner_margin(margin)
                                .fill(palette.card)
                                .stroke(Stroke::new(1.0, palette.border))
                                .corner_radius(radius),
                        ),
                );
                ui.ctx()
                    .accesskit_node_builder(input.id, |node| node.set_label(LABEL));
                let color = if input.has_focus() {
                    ui.painter().rect_stroke(
                        input.rect,
                        radius,
                        Stroke::new(2.0, palette.accent),
                        egui::StrokeKind::Inside,
                    );
                    palette.accent
                } else {
                    palette.muted
                };
                let center = egui::pos2(input.rect.left() + 15.0, input.rect.center().y - 1.5);
                let ink = Stroke::new(1.6, color);
                ui.painter().circle_stroke(center, 4.5, ink);
                ui.painter().line_segment(
                    [center + egui::vec2(3.3, 3.3), center + egui::vec2(7.0, 7.0)],
                    ink,
                );
            });
        });
        ui.add_space(if palette.aero { 12.0 } else { 18.0 });
        if self.query.trim().is_empty() && self.category.is_none() {
            self.feature(ui, palette);
            ui.add_space(if palette.aero { 15.0 } else { 22.0 });
        }

        ui.heading(RichText::new("Aplicativos").size(24.0).strong());
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for category in std::iter::once(None).chain(CATEGORIES.into_iter().map(Some)) {
                let text = category.map_or("Todos", Category::label);
                if self
                    .filter_button(ui, palette, text, self.category == category)
                    .clicked()
                {
                    self.category = category;
                }
            }
            ui.separator();
            for (label, view) in [("Cartões", View::Cards), ("Lista", View::List)] {
                let clicked = ui
                    .add_enabled_ui(self.read_error.is_none(), |ui| {
                        self.filter_button(ui, palette, label, self.settings.view == view)
                            .clicked()
                    })
                    .inner;
                if clicked {
                    self.change(ui.ctx(), |settings| settings.view = view);
                }
            }
        });
        ui.add_space(14.0);
        let matches: Vec<_> = PRODUCTS
            .iter()
            .enumerate()
            .filter(|(_, product)| product.matches(&self.query, self.category))
            .map(|(index, _)| index)
            .collect();
        if matches.is_empty() {
            ui.label(RichText::new("Nenhum aplicativo corresponde à busca.").color(palette.muted));
            if ui.button("Limpar filtros").clicked() {
                self.query.clear();
                self.category = None;
            }
        } else if self.settings.view == View::Cards {
            let gap = 14.0;
            let columns =
                (((ui.available_width() + gap) / (204.0 + gap)).floor() as usize).clamp(1, 4);
            let width = (ui.available_width() - gap * (columns - 1) as f32) / columns as f32;
            for row in matches.chunks(columns) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for &index in row {
                        self.card(ui, palette, index, width);
                    }
                });
                ui.add_space(10.0);
            }
        } else {
            for index in matches {
                self.product_row(ui, palette, index);
            }
        }
    }

    fn feature(&mut self, ui: &mut Ui, palette: Palette) {
        let width = ui.available_width();
        let radius = CornerRadius::same(14);
        let artwork = self
            .aero_assets(palette)
            .and_then(|aero| aero.lake.clone())
            .or_else(|| self.covers[0].clone());
        if width < 560.0 {
            surface(palette)
                .fill(palette.feature)
                .corner_radius(radius)
                .show(ui, |ui| {
                    let (art, _) =
                        ui.allocate_exact_size(egui::vec2(width - 2.0, 94.0), egui::Sense::hover());
                    if let Some(artwork) = &artwork {
                        feature_cover(
                            ui,
                            art,
                            artwork,
                            CornerRadius {
                                sw: 0,
                                se: 0,
                                ..radius
                            },
                        );
                    }
                    Frame::new().inner_margin(Margin::same(15)).show(ui, |ui| {
                        self.feature_text(ui, palette, 32.0);
                    });
                });
            return;
        }
        let height = if palette.aero { 214.0 } else { 252.0 };
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        ui.painter().rect_filled(rect, radius, palette.feature);
        let split = if palette.aero { 0.40 } else { 0.45 };
        let art_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left() + rect.width() * split, rect.top()),
            rect.right_bottom(),
        );
        if let Some(artwork) = &artwork {
            let corners = CornerRadius {
                nw: 0,
                sw: 0,
                ..radius
            };
            feature_cover(ui, art_rect, artwork, corners);
            if palette.aero {
                if palette.mode == Mode::Dark {
                    ui.painter()
                        .rect_filled(art_rect, corners, Color32::from_black_alpha(27));
                }
                let fade = egui::Rect::from_min_size(
                    art_rect.min,
                    egui::vec2(art_rect.width() * 0.35, art_rect.height()),
                );
                gradient(
                    ui,
                    fade,
                    [
                        palette.feature,
                        Color32::TRANSPARENT,
                        Color32::TRANSPARENT,
                        palette.feature,
                    ],
                );
            }
        }
        ui.painter().rect_stroke(
            rect,
            radius,
            Stroke::new(1.0, palette.border),
            egui::StrokeKind::Inside,
        );
        let text_rect = egui::Rect::from_min_max(
            rect.min + egui::vec2(21.0, if palette.aero { 15.0 } else { 24.0 }),
            egui::pos2(art_rect.left() - 12.0, rect.bottom() - 12.0),
        );
        // The banner already reserved its height in the parent layout.
        let mut text = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
        self.feature_text(&mut text, palette, if width >= 760.0 { 48.0 } else { 35.0 });
    }

    fn feature_text(&mut self, ui: &mut Ui, palette: Palette, title_size: f32) {
        ui.label(
            RichText::new(PRODUCTS[0].subtitle)
                .color(palette.muted)
                .size(13.0)
                .strong(),
        );
        ui.add_space(7.0);
        ui.heading(RichText::new(PRODUCTS[0].name).size(title_size).strong());
        ui.add_space(if palette.aero { 4.0 } else { 6.0 });
        ui.label(
            RichText::new(PRODUCTS[0].description)
                .size(16.0)
                .color(palette.text),
        );
        ui.add_space(if palette.aero { 7.0 } else { 11.0 });
        let button = primary(palette, "Ver aplicativo");
        let response = if let Some(aero) = self.aero_assets(palette) {
            aero.button(ui, button, [154.0, 35.0], true, palette)
        } else {
            ui.add(button)
        };
        if response.clicked() {
            self.details(0);
        }
    }

    fn card(&mut self, ui: &mut Ui, palette: Palette, index: usize, width: f32) {
        let product = &PRODUCTS[index];
        let linked = self.settings.links.contains_key(product.id);
        let id = ui.id().with(product.id);
        let hovered = ui
            .ctx()
            .read_response(id)
            .is_some_and(|response| response.hovered())
            && !ui.input(|input| input.any_touches());
        let motion = animate_feedback(ui, id.with("hover"), hovered);
        let zoom = if ui.style().animation_time > 0.0 {
            1.0 + 0.035 * motion
        } else {
            1.0
        };
        let frame = surface(palette).show(ui, |ui| {
            ui.vertical(|ui| {
                let width = width - 2.0;
                ui.set_width(width);
                let art_height = if palette.aero { 85.0 } else { 102.0 };
                let (bounds, _) =
                    ui.allocate_exact_size(egui::vec2(width, art_height), egui::Sense::hover());
                if let Some(image) = self.artwork(palette, index) {
                    cover_zoom(
                        ui,
                        bounds,
                        image,
                        CornerRadius {
                            nw: 10,
                            ne: 10,
                            sw: 0,
                            se: 0,
                        },
                        zoom,
                    );
                }
                let margin = if palette.aero { 8 } else { 12 };
                let status = if linked {
                    "Vínculo local"
                } else {
                    "Planejado"
                };
                Frame::new()
                    .inner_margin(Margin::same(margin))
                    .show(ui, |ui| {
                        ui.set_width(width - 2.0 * f32::from(margin));
                        ui.allocate_ui_with_layout(
                            egui::vec2(
                                ui.available_width(),
                                if palette.aero { 38.0 } else { 57.0 },
                            ),
                            egui::Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                self.icon(ui, index, 32.0);
                                ui.vertical(|ui| {
                                    let mut name = RichText::new(product.name).strong().size(18.0);
                                    if palette.aero {
                                        name = name.family(aero_font(ui, true));
                                    }
                                    ui.add(egui::Label::new(name).selectable(false));
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(if palette.aero {
                                                status
                                            } else {
                                                product.subtitle
                                            })
                                            .color(palette.muted)
                                            .size(12.0),
                                        )
                                        .selectable(false),
                                    );
                                });
                            },
                        );
                        if !palette.aero {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(status).color(palette.muted).size(12.0),
                                )
                                .selectable(false),
                            );
                        }
                        let (action, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 34.0),
                            egui::Sense::hover(),
                        );
                        let family = if let Some(aero) = self.aero_assets(palette) {
                            ui.painter()
                                .add(aero.button_surface(action, false, motion, 0.0, palette));
                            aero_font(ui, false)
                        } else {
                            ui.painter().rect(
                                action,
                                CornerRadius::same(7),
                                palette.control,
                                Stroke::new(1.0, palette.border),
                                egui::StrokeKind::Inside,
                            );
                            egui::FontFamily::Proportional
                        };
                        ui.painter().text(
                            action.center(),
                            egui::Align2::CENTER_CENTER,
                            "Ver detalhes",
                            egui::FontId::new(14.0, family),
                            palette.text,
                        );
                    });
            });
        });
        let rect = frame.response.rect;
        let response = ui
            .interact(rect, id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                ui.is_enabled(),
                format!("Ver detalhes de {}", product.name),
            )
        });
        let highlight = motion.max(f32::from(response.has_focus()));
        if highlight > 0.0 {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(11),
                Stroke::new(2.0, palette.accent.gamma_multiply(highlight)),
                egui::StrokeKind::Inside,
            );
        }
        if response.clicked() {
            self.details(index);
        }
    }

    fn icon(&self, ui: &mut Ui, index: usize, size: f32) {
        if let Some(image) = &self.icons[index] {
            ui.image((image.id(), Vec2::splat(size)));
        }
    }

    fn product_row(&mut self, ui: &mut Ui, palette: Palette, index: usize) {
        let product = &PRODUCTS[index];
        let linked = self.settings.links.contains_key(product.id);
        surface(palette)
            .inner_margin(Margin::same(11))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    self.icon(ui, index, 36.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(product.name).strong());
                        ui.label(RichText::new(product.subtitle).color(palette.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self
                            .action_button(ui, palette, "Ver detalhes", "↗", false)
                            .clicked()
                        {
                            self.details(index);
                        }
                        ui.label(
                            RichText::new(if linked {
                                "Vínculo local"
                            } else {
                                "Planejado"
                            })
                            .color(palette.muted),
                        );
                    });
                });
            });
    }

    fn action_button(
        &self,
        ui: &mut Ui,
        palette: Palette,
        label: &str,
        symbol: &str,
        primary: bool,
    ) -> egui::Response {
        let text = RichText::new(format!("{symbol}  {label}"))
            .size(16.0)
            .color(if primary {
                palette.on_accent
            } else {
                palette.text
            });
        let width = ui
            .fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(
                        format!("{symbol}  {label}"),
                        egui::FontId::proportional(16.0),
                        palette.text,
                    )
                    .size()
                    .x
                    + 36.0
            })
            .max(120.0)
            .min(ui.available_width());
        let widget = Button::new(text);
        let response = if let Some(aero) = self.aero_assets(palette) {
            aero.button(ui, widget, [width, 46.0], primary, palette)
        } else {
            ui.add_sized(
                [width, 46.0],
                widget.fill(if primary {
                    palette.accent
                } else {
                    palette.control
                }),
            )
        };
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_label(label));
        response
    }

    fn detail_page(&mut self, ui: &mut Ui, palette: Palette, index: usize) {
        let product = &PRODUCTS[index];
        if self
            .action_button(ui, palette, "Voltar", "←", false)
            .clicked()
        {
            self.navigate(self.back);
        }
        ui.add_space(16.0);
        surface(palette)
            .inner_margin(Margin::same(20))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    self.icon(ui, index, 72.0);
                    ui.vertical(|ui| {
                        title(ui, product.name);
                        ui.label(
                            RichText::new(product.subtitle)
                                .size(16.0)
                                .color(palette.muted),
                        );
                        ui.label(RichText::new("Em planejamento").color(palette.accent));
                    });
                });
                ui.add_space(16.0);
                let width = ui.available_width();
                let (bounds, _) = ui.allocate_exact_size(
                    egui::vec2(width, (width * 0.16).clamp(90.0, 120.0)),
                    egui::Sense::hover(),
                );
                if let Some(artwork) = self.artwork(palette, index) {
                    cover(ui, bounds, artwork, CornerRadius::same(10));
                }
                ui.add_space(16.0);
                ui.label(
                    RichText::new(product.description)
                        .size(17.0)
                        .color(palette.text),
                );
                ui.add_space(16.0);
                ui.horizontal_wrapped(|ui| {
                    if self
                        .action_button(ui, palette, "Página do aplicativo", "↗", false)
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
                            "{WEBSITE}/apps/{}",
                            product.id
                        )));
                    }
                });
            });
        ui.add_space(18.0);
        ui.label(
            RichText::new("Recursos planejados")
                .size(21.0)
                .strong()
                .color(palette.text),
        );
        ui.add_space(10.0);
        let columns = if ui.available_width() >= 780.0 {
            product.scope.len()
        } else {
            1
        };
        ui.columns(columns, |uis| {
            for (position, item) in product.scope.iter().enumerate() {
                let ui = &mut uis[position % columns];
                surface(palette)
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let (mark, _) =
                            ui.allocate_exact_size(egui::vec2(24.0, 6.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(mark, CornerRadius::same(3), palette.accent);
                        ui.label(RichText::new(*item).size(16.0).strong().color(palette.text));
                    });
                ui.add_space(8.0);
            }
        });
        ui.add_space(10.0);
        surface(palette).inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Neste computador").size(21.0).strong().color(palette.text));
            ui.add_space(8.0);
            ui.label(RichText::new("Adicione um executável local para acessar este aplicativo pela Biblioteca.").color(palette.muted));
            ui.add_space(14.0);
            self.local_actions(ui, palette, index);
        });
    }

    fn library(&mut self, ui: &mut Ui, palette: Palette) {
        title(ui, "Biblioteca");
        ui.add_space(12.0);
        if self.settings.links.is_empty() {
            ui.label(RichText::new("Vincule um executável nos detalhes de um aplicativo para abri-lo pela Biblioteca.").color(palette.muted));
            if self
                .action_button(ui, palette, "Explorar aplicativos", "↗", true)
                .clicked()
            {
                self.navigate(Page::Explore);
            }
        }
        for (index, product) in PRODUCTS.iter().enumerate() {
            if self.settings.links.contains_key(product.id) {
                surface(palette)
                    .inner_margin(Margin::same(13))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            self.icon(ui, index, 35.0);
                            ui.label(RichText::new(product.name).strong().size(19.0));
                        });
                        self.local_actions(ui, palette, index);
                    });
                ui.add_space(8.0);
            }
        }
    }

    fn local_actions(&mut self, ui: &mut Ui, palette: Palette, index: usize) {
        let id = PRODUCTS[index].id;
        let linked = self.settings.links.contains_key(id);
        let editable = self.read_error.is_none();
        let mut available = false;
        if let Some(path) = self.settings.links.get(id) {
            available = config::executable(path).is_ok();
            ui.label(
                RichText::new(path.display().to_string())
                    .color(palette.muted)
                    .size(12.0),
            );
            if !available {
                ui.label(
                    RichText::new("Executável indisponível. Selecione outro arquivo.")
                        .color(palette.muted),
                );
            }
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if linked
                && ui
                    .add_enabled_ui(available, |ui| {
                        self.action_button(ui, palette, "Abrir", "▶", true)
                    })
                    .inner
                    .clicked()
            {
                self.launch(index);
            }
            let label = if linked {
                "Trocar executável"
            } else {
                "Vincular executável"
            };
            if ui
                .add_enabled_ui(editable, |ui| {
                    self.action_button(ui, palette, label, "+", !linked)
                })
                .inner
                .clicked()
            {
                self.browse(ui, index);
            }
            if linked
                && ui
                    .add_enabled_ui(editable, |ui| {
                        self.action_button(ui, palette, "Remover vínculo", "×", false)
                    })
                    .inner
                    .clicked()
            {
                self.change(ui.ctx(), |settings| {
                    settings.links.remove(id);
                });
            }
        });
    }

    fn preferences(&mut self, ui: &mut Ui, palette: Palette) {
        title(ui, "Configurações");
        ui.label(format!("Sunrise Launcher {}", env!("CARGO_PKG_VERSION")));
        ui.add_space(15.0);
        ui.label(RichText::new("Aparência").size(20.0).strong());
        ui.add_enabled_ui(self.read_error.is_none(), |ui| {
            ui.horizontal_wrapped(|ui| {
                for (label, theme) in [("Moderno", Theme::Modern), ("Frutiger Aero", Theme::Aero)] {
                    if ui.radio(self.settings.theme == theme, label).clicked() {
                        self.change(ui.ctx(), |settings| settings.theme = theme);
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                for (label, mode) in [("Claro", Mode::Light), ("Escuro", Mode::Dark)] {
                    if ui.radio(self.settings.mode == mode, label).clicked() {
                        self.change(ui.ctx(), |settings| settings.mode = mode);
                    }
                }
            });
            let mut reduce_motion = self.settings.reduce_motion;
            if ui
                .checkbox(&mut reduce_motion, "Reduzir movimento")
                .changed()
            {
                self.change(ui.ctx(), |settings| settings.reduce_motion = reduce_motion);
            }
        });
        ui.add_space(12.0);
        ui.label(RichText::new("As escolhas ficam salvas neste computador.").color(palette.muted));
        if let Some(path) = &self.path {
            ui.label(
                RichText::new(path.display().to_string())
                    .color(palette.muted)
                    .size(12.0),
            );
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let bounds = ui.available_rect_before_wrap();
        let (mode, opacity) = mode_transition(ui, self.settings.mode, self.settings.reduce_motion);
        let palette = Palette::new(self.settings.theme, mode);
        let visible_appearance = (self.settings.theme, mode);
        if self.visible_appearance != visible_appearance {
            palette.apply(ui.ctx());
            self.visible_appearance = visible_appearance;
        }
        ui.style_mut().animation_time = if self.settings.reduce_motion {
            0.0
        } else {
            0.16
        };
        if palette.aero {
            self.aero.get_or_insert_with(|| Aero::new(ui.ctx()));
            Aero::typography(ui);
            let wallpaper = match palette.mode {
                Mode::Light => self
                    .aero_assets(palette)
                    .and_then(|aero| aero.lake.as_ref()),
                Mode::Dark => self.dew.as_ref(),
            };
            ui.painter()
                .rect_filled(bounds, CornerRadius::ZERO, palette.rail);
            if let Some(wallpaper) = wallpaper {
                cover(ui, bounds, wallpaper, CornerRadius::ZERO);
            }
            let shade = if palette.mode == Mode::Dark { 80 } else { 0 };
            gradient(
                ui,
                bounds,
                [
                    Color32::from_black_alpha(shade / 2),
                    Color32::TRANSPARENT,
                    Color32::from_black_alpha(shade),
                    Color32::from_black_alpha(shade),
                ],
            );
            Frame::new()
                .outer_margin(Margin::same(if bounds.width() >= 980.0 { 19 } else { 10 }))
                .inner_margin(Margin::same(2))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(174)))
                .corner_radius(CornerRadius::same(14))
                .show(ui, |ui| {
                    egui::Panel::top("aero_brand")
                        .exact_size(42.0)
                        .resizable(false)
                        .frame(
                            Frame::new()
                                .inner_margin(Margin::symmetric(12, 6))
                                .fill(palette.panel_fill(palette.rail)),
                        )
                        .show(ui, |ui| {
                            let area = ui.available_rect_before_wrap();
                            let glint = if palette.mode == Mode::Light { 154 } else { 42 };
                            gradient(
                                ui,
                                area.expand2(egui::vec2(9.0, 4.0)),
                                [
                                    Color32::from_white_alpha(glint),
                                    Color32::from_white_alpha(glint / 2),
                                    Color32::TRANSPARENT,
                                    Color32::TRANSPARENT,
                                ],
                            );
                            ui.horizontal(|ui| {
                                self.brand(ui);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let (label, mode) = match self.settings.mode {
                                            Mode::Light => ("Modo escuro", Mode::Dark),
                                            Mode::Dark => ("Modo claro", Mode::Light),
                                        };
                                        let clicked = ui
                                            .add_enabled_ui(self.read_error.is_none(), |ui| {
                                                let button = Button::new(
                                                    RichText::new(label).color(palette.text),
                                                );
                                                self.aero_assets(palette).is_some_and(|aero| {
                                                    aero.button(
                                                        ui,
                                                        button,
                                                        [112.0, 30.0],
                                                        false,
                                                        palette,
                                                    )
                                                    .clicked()
                                                })
                                            })
                                            .inner;
                                        if clicked {
                                            self.change(ui.ctx(), |settings| settings.mode = mode);
                                        }
                                    },
                                );
                            });
                        });
                    self.content(ui, palette);
                });
        } else {
            self.content(ui, palette);
        }
        if opacity > 0 {
            let top = Color32::from_rgba_unmultiplied(70, 94, 105, opacity);
            let bottom = Color32::from_rgba_unmultiplied(32, 56, 65, opacity);
            gradient(ui, bounds, [top, top, bottom, bottom]);
        }
    }

    fn content(&mut self, ui: &mut Ui, palette: Palette) {
        let rail_width = if ui.available_width() < 780.0 {
            if palette.aero { 166.0 } else { 158.0 }
        } else {
            211.0
        };
        egui::Panel::left("navigation")
            .exact_size(rail_width)
            .resizable(false)
            .frame(Frame::new().fill(palette.panel_fill(palette.rail)))
            .show(ui, |ui| self.navigation(ui, palette));
        egui::CentralPanel::default().frame(Frame::new().fill(palette.panel_fill(palette.page))).show(ui, |ui| {
            let page = self.page;
            egui::ScrollArea::vertical().id_salt(format!("{page:?}")).auto_shrink([false; 2]).show(ui, |ui| {
                let padding = if palette.aero { 20 } else { 22 };
                Frame::new().inner_margin(Margin::same(padding)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if let Some(message) = self.error.as_ref().or(self.read_error.as_ref()) {
                        ui.label(RichText::new(message).strong().color(palette.text));
                        ui.add_space(8.0);
                    }
                    match page {
                        Page::Explore => self.explore(ui, palette),
                        Page::Library => self.library(ui, palette),
                        Page::Updates => {
                            title(ui, "Atualizações");
                            ui.add_space(12.0);
                            ui.label(RichText::new("Os aplicativos Rise ainda estão planejados. As versões públicas aparecerão no site do projeto.").color(palette.muted));
                            ui.hyperlink_to("Consultar o roadmap", format!("{WEBSITE}/#roadmap"));
                        }
                        Page::Settings => self.preferences(ui, palette),
                        Page::Details(index) => self.detail_page(ui, palette, index),
                    }
                });
            });
        });
    }
}

impl eframe::App for Launcher {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

fn feature_cover(ui: &Ui, bounds: egui::Rect, artwork: &TextureHandle, corners: CornerRadius) {
    let zoom = if ui.style().animation_time > 0.0 {
        let hovered = ui.rect_contains_pointer(bounds) && !ui.input(|input| input.any_touches());
        1.0 + 0.045 * animate_feedback(ui, ui.id().with("feature-artwork"), hovered)
    } else {
        1.0
    };
    cover_zoom(ui, bounds, artwork, corners, zoom);
}

fn title(ui: &mut Ui, text: &str) {
    ui.heading(RichText::new(text).size(28.0).strong());
}

fn surface(palette: Palette) -> Frame {
    Frame::new()
        .fill(palette.card)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(11))
}

fn primary(palette: Palette, label: &str) -> Button<'_> {
    Button::new(RichText::new(label).strong().color(palette.on_accent))
        .fill(palette.accent)
        .min_size(egui::vec2(120.0, 34.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn light_aero_retains_accessible_navigation_and_seven_card_actions() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        let initial = Settings::default();
        let mut settings = initial.clone();
        settings.theme = Theme::Aero;
        settings.mode = Mode::Light;
        config::save(&path, &initial, &settings).unwrap();
        let mut ui = Harness::builder()
            .with_size(egui::vec2(1280.0, 850.0))
            .build_ui_state(
                |ui, launcher: &mut Launcher| launcher.show(ui),
                Launcher::new(&Context::default(), Some(path.clone())),
            );
        ui.run();
        assert_eq!(ui.get_all_by_label_contains("Ver detalhes de ").count(), 7);
        for label in [
            "Todos",
            "Imagem",
            "Vídeo",
            "Ilustração",
            "Documentos",
            "Cartões",
            "Lista",
        ] {
            let control = ui.get_by_label(label).rect();
            assert!(
                control.height() >= 40.0 && control.width() >= 70.0,
                "{label}: {control:?}"
            );
        }
        let action = ui.get_by_label("Ver aplicativo").rect();
        let catalog = ui.get_by_label("Aplicativos").rect();
        assert!(
            catalog.top() > action.bottom() + 24.0,
            "{catalog:?} / {action:?}"
        );
        for (name, index, offset) in [
            ("Ver detalhes de RisePhoto", 0, egui::vec2(12.0, 12.0)),
            ("Ver detalhes de RiseFilm", 1, egui::vec2(4.0, 126.0)),
        ] {
            let position = ui.get_by_label(name).rect().min + offset;
            ui.event(egui::Event::PointerMoved(position));
            for pressed in [true, false] {
                ui.event(egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            ui.run();
            assert_eq!(ui.state().page, Page::Details(index));
            ui.get_by_label("Voltar").click();
            ui.run();
        }
        ui.get_by_label("Ver detalhes de RiseVector").focus();
        ui.run();
        ui.key_press(egui::Key::Enter);
        ui.run();
        assert_eq!(ui.state().page, Page::Details(2));
        for scale in [1.0, 1.5, 2.0] {
            let mut ui = Harness::builder()
                .with_size(egui::vec2(650.0, 475.0))
                .with_pixels_per_point(scale)
                .build_ui_state(
                    |ui, launcher: &mut Launcher| launcher.show(ui),
                    Launcher::new(&Context::default(), Some(path.clone())),
                );
            let settings = ui.get_by_label("Configurações").rect();
            let explore = ui.get_by_label("Ver aplicativo").rect();
            assert!(settings.right() < explore.left());
            assert!(explore.bottom() < 475.0, "scale {scale}: {explore:?}");
            ui.get_by_label("Configurações").click();
            ui.run();
            assert_eq!(ui.state().page, Page::Settings);
            assert!(ui.query_by_label("Claro").is_some());
        }
    }

    #[test]
    fn aero_layout_keeps_search_and_feature_action_inside_the_viewport() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        let initial = Settings::default();
        let mut settings = initial.clone();
        settings.theme = Theme::Aero;
        config::save(&path, &initial, &settings).unwrap();
        let mut ui = Harness::builder()
            .with_size(egui::vec2(650.0, 475.0))
            .build_ui_state(
                |ui, launcher: &mut Launcher| launcher.show(ui),
                Launcher::new(&Context::default(), Some(path.clone())),
            );
        for width in [650.0, 850.0, 1280.0] {
            ui.set_size(egui::vec2(width, 580.0)).run();
            let search = ui.get_by_label("Buscar aplicativos").rect();
            let action = ui.get_by_label("Ver aplicativo").rect();
            let catalog = ui.get_by_label("Aplicativos").rect();
            assert!(
                search.right() < width && search.right() > width - 65.0 && search.width() > 130.0
            );
            assert!(search.height() >= 38.0);
            assert!(action.left() > 140.0 && action.bottom() < 475.0);
            assert!(catalog.top() > action.bottom() + 25.0);
        }
        ui.set_size(egui::vec2(1280.0, 850.0)).run();
        let first_row: Vec<_> = ui
            .get_all_by_label_contains("Ver detalhes de ")
            .take(4)
            .map(|card| card.rect().bottom())
            .collect();
        assert_eq!(first_row.len(), 4);
        for top in &first_row[1..] {
            assert!((*top - first_row[0]).abs() < 1.0);
        }
        let search = ui.get_by_label("Buscar aplicativos").rect();
        let position = egui::pos2(search.left() + 15.0, search.center().y);
        ui.event(egui::Event::PointerMoved(position));
        for pressed in [true, false] {
            ui.event(egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        ui.run();
        ui.event(egui::Event::Text("risevector".into()));
        ui.run();
        assert_eq!(ui.state().query, "risevector");
        assert!(ui.query_by_label("Ver detalhes de RiseVector").is_some());
        assert!(ui.query_by_label("Ver detalhes de RiseFilm").is_none());
        ui.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        ui.key_press(egui::Key::Backspace);
        ui.run();
        assert!(ui.state().query.is_empty());
        ui.get_by_label("Modo claro").click();
        ui.run();
        assert_eq!(config::read(&path).unwrap().mode, Mode::Light);
        ui.get_by_label("Modo escuro").click();
        ui.run();
        assert_eq!(config::read(&path).unwrap().mode, Mode::Dark);
        ui.get_by_label("Ver aplicativo").click();
        ui.run();
        assert_eq!(ui.state().page, Page::Details(0));
    }

    #[test]
    fn accessible_flow_filters_opens_details_and_persists_appearance() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        let mut ui = Harness::builder()
            .with_size(egui::vec2(1280.0, 850.0))
            .build_ui_state(
                |ui, launcher: &mut Launcher| launcher.show(ui),
                Launcher::new(&Context::default(), Some(path.clone())),
            );
        ui.get_by_label("Imagem").click();
        ui.run();
        assert_eq!(ui.state().category.map(Category::label), Some("Imagem"));
        assert!(ui.query_by_label("RiseFilm").is_none());
        ui.get_by_label("Todos").click();
        ui.run();
        ui.get_by_label("Ver aplicativo").click();
        ui.run();
        assert_eq!(ui.state().page, Page::Details(0));
        assert!(ui.query_by_label("Vincular executável").is_some());
        assert!(ui.query_by_label("Instalar").is_none());
        ui.get_by_label("Configurações").click();
        ui.run();
        ui.get_by_label("Frutiger Aero").click();
        ui.run();
        assert_eq!(config::read(&path).unwrap().theme, Theme::Aero);
        ui.get_by_label("Reduzir movimento").click();
        ui.run();
        assert!(config::read(&path).unwrap().reduce_motion);
    }

    #[test]
    fn library_remove_preserves_executable_and_stale_links_offer_repair() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        let executable = std::env::current_exe().unwrap();
        let initial = Settings::default();
        let mut saved = initial.clone();
        saved.links.insert("risephoto".into(), executable.clone());
        saved
            .links
            .insert("risefilm".into(), folder.path().join("missing.exe"));
        config::save(&path, &initial, &saved).unwrap();
        let mut launcher = Launcher::new(&Context::default(), Some(path.clone()));
        launcher.page = Page::Library;
        let mut ui = Harness::builder()
            .with_size(egui::vec2(900.0, 620.0))
            .build_ui_state(|ui, launcher| launcher.show(ui), launcher);
        assert!(
            ui.query_by_label("Executável indisponível. Selecione outro arquivo.")
                .is_some()
        );
        ui.get_all_by_label("Remover vínculo")
            .next()
            .unwrap()
            .click();
        ui.run();
        assert!(executable.is_file());
        assert!(!config::read(&path).unwrap().links.contains_key("risephoto"));
    }
}
