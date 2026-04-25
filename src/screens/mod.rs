//! Navigation router for the cross-platform GPUI example app.
//!
//! This module defines the available screens, a shared navigation model,
//! and a top-level `Router` view that renders the currently active screen.
//!
//! ## Screens
//!
//! - **Home** — welcome message, colour swatches, stats, and quick-nav cards.
//! - **Counter** — increment / decrement / reset a shared tap counter.
//! - **Settings** — toggle dark mode, reset counter, change user name.
//! - **About** — app info, technology stack, architecture, and credits.
//! - **Animations** — bouncing balls with physics, trails, and particle effects.
//! - **Shaders** — dynamic gradients, floating orbs, and ripple effects.

pub mod about;
pub mod anki;
pub mod anki_settings;
pub mod audio_player;
pub mod chat;
pub mod components;
pub mod counter;
pub mod dictionaries;
pub mod feed;
pub mod form;
pub mod home;
pub mod packages_demo;
pub mod search;
pub mod settings;
pub mod settings_demo;
pub mod swiper;
pub mod video_player;
pub mod webview_browser;

use crate::demos::{AnimationPlayground, ShaderShowcase};
use gpui::{
    div, point, prelude::*, px, rgb, size, Bounds, Context, Entity, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, SharedString, Window,
};
use gpui_mobile::components::material::{MaterialTheme, NavigationBarBuilder, TopAppBar};
use gpui_mobile::{set_system_chrome, StatusBarContentStyle, SystemChromeStyle};

// ── Screen enum ──────────────────────────────────────────────────────────────

/// All navigable screens in the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Search,
    Dictionaries,
    Settings,
    About,
    Home,
    Counter,
    AppleGlass,
    Material,
    Form,
    Animations,
    Shaders,
    PackagesDemo,
    WebViewBrowser,
    Swiper,
    Feed,
    Chat,
    AudioPlayer,
    VideoPlayer,
    Anki,
    AnkiSettings,
}

impl Screen {
    /// Parse a screen from a deeplink URL path segment.
    ///
    /// Accepts URLs like `gpui://video_player`, `gpui://counter`,
    /// `gpui://settings`, etc. The host or first path segment is
    /// matched case-insensitively.
    ///
    /// Returns `None` for unrecognized paths or empty URLs.
    pub fn from_deeplink_url(url: &str) -> Option<Self> {
        // Parse: "gpui://video_player" → host = "video_player"
        //        "gpui://video_player/foo" → host = "video_player"
        let stripped = url
            .strip_prefix("gpui://")
            .or_else(|| url.strip_prefix("gpui:"))?;
        let path = stripped.split('/').next().unwrap_or("").trim();
        if path.is_empty() {
            return None;
        }
        match path.to_ascii_lowercase().as_str() {
            "search" => Some(Screen::Search),
            "dictionaries" => Some(Screen::Dictionaries),
            "settings" => Some(Screen::Settings),
            "about" => Some(Screen::About),
            "home" => Some(Screen::Home),
            "counter" => Some(Screen::Counter),
            "apple_glass" | "appleglass" => Some(Screen::AppleGlass),
            "material" => Some(Screen::Material),
            "form" => Some(Screen::Form),
            "animations" => Some(Screen::Animations),
            "shaders" => Some(Screen::Shaders),
            "packages" | "packages_demo" => Some(Screen::PackagesDemo),
            "webview" | "webview_browser" | "browser" => Some(Screen::WebViewBrowser),
            "swiper" | "discover" => Some(Screen::Swiper),
            "feed" => Some(Screen::Feed),
            "chat" => Some(Screen::Chat),
            "audio_player" | "audio" => Some(Screen::AudioPlayer),
            "video_player" | "video" => Some(Screen::VideoPlayer),
            "anki" => Some(Screen::Anki),
            "anki_settings" => Some(Screen::AnkiSettings),
            _ => None,
        }
    }

    /// Human-readable title for the screen (used in the nav bar).
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Search => "Search",
            Screen::Dictionaries => "Dictionaries",
            Screen::Settings => "Settings",
            Screen::About => "About",
            Screen::Home => "Home",
            Screen::Counter => "Counter",
            Screen::AppleGlass => "Apple Liquid Glass",
            Screen::Material => "Material Design 3",
            Screen::Form => "Material Form",
            Screen::Animations => "Animations",
            Screen::Shaders => "Shaders",
            Screen::PackagesDemo => "Packages",
            Screen::WebViewBrowser => "Browser",
            Screen::Swiper => "Discover",
            Screen::Feed => "Feed",
            Screen::Chat => "Sarah Johnson",
            Screen::AudioPlayer => "Audio Player",
            Screen::VideoPlayer => "Video Player",
            Screen::Anki => "Anki",
            Screen::AnkiSettings => "Anki Settings",
        }
    }

    /// Whether this screen is a primary tab-bar destination.
    ///
    /// Tab roots are the screens directly reachable from the bottom
    /// navigation bar. Navigating between them clears the history
    /// stack so the back button is never shown on these screens.
    pub fn is_tab_root(&self) -> bool {
        matches!(
            self,
            Screen::Search | Screen::Dictionaries | Screen::Settings | Screen::About
        )
    }
}

// ── Colour palette (Google Material) ─────────────────────────────────────────

pub const BASE: u32 = 0x121318; // Dark surface
pub const SURFACE0: u32 = 0x1E1F25; // Dark surface container
pub const SURFACE1: u32 = 0x282A2F; // Dark surface container high
pub const TEXT: u32 = 0xE2E2E9; // Dark on-surface
pub const SUBTEXT: u32 = 0xC4C6D0; // Dark on-surface-variant
pub const BLUE: u32 = 0x4285F4; // Google Blue
pub const GREEN: u32 = 0x34A853; // Google Green
pub const RED: u32 = 0xEA4335; // Google Red
pub const MAUVE: u32 = 0xA142F4; // Google Purple
pub const YELLOW: u32 = 0xFBBC04; // Google Yellow
pub const PEACH: u32 = 0xFA7B17; // Google Orange
pub const TEAL: u32 = 0x24C1E0; // Google Teal
pub const MANTLE: u32 = 0x0D0E13; // Dark surface container lowest
pub const SKY: u32 = 0x4FC3F7; // Light Blue
pub const LAVENDER: u32 = 0x7B8CF8; // Indigo light

// Light mode equivalents (used inline in screen render functions).
pub const LIGHT_TEXT: u32 = 0x1A1B20;
pub const LIGHT_SUBTEXT: u32 = 0x44474F;
pub const LIGHT_CARD_BG: u32 = 0xEDEDF4;
pub const LIGHT_DIVIDER: u32 = 0xC4C6D0;

// ── Safe area ────────────────────────────────────────────────────────────────

/// Safe area insets in logical pixels.
///
/// These represent the areas occupied by system UI (status bar, navigation
/// bar, camera notch) that the app content should pad around.
#[derive(Debug, Clone, Copy, Default)]
pub struct SafeArea {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

// ── Router ───────────────────────────────────────────────────────────────────

/// Top-level view that owns navigation state and delegates rendering to the
/// active screen.
pub struct Router {
    pub current_screen: Screen,
    /// Shared state: a global tap counter (carried across screens for demo).
    pub tap_count: u32,
    /// User name shown on the home screen.
    pub user_name: SharedString,
    /// A flag toggled in Settings.
    pub dark_mode: bool,
    /// Global font size multiplier.
    pub font_size_multiplier: f32,
    /// Navigation history stack for back navigation.
    history: Vec<Screen>,
    /// Safe area insets (logical pixels) to pad around system chrome.
    pub safe_area: SafeArea,

    // ── Yomichan state ───────────────────────────────────────────────────
    pub search_state: Entity<search::SearchState>,
    pub anki_state: Entity<anki::AnkiRouter>,
    pub anki_settings_state: Entity<anki_settings::AnkiSettingsState>,
    pub dictionaries_state: Entity<dictionaries::DictionariesState>,
    settings_state: Entity<settings::SettingsState>,

    /// Monokakido-style zoom header.
    zoom_header: Entity<gpui_mobile::components::material::ZoomHeader>,

    // ── Demo view state ──────────────────────────────────────────────────
    /// The animation playground demo (lazily created when the screen is visited).
    animation_playground: Option<AnimationPlayground>,
    /// The shader showcase demo (lazily created when the screen is visited).
    shader_showcase: Option<ShaderShowcase>,
}

impl Router {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::with_initial_screen(Screen::default(), window, cx)
    }

    /// Create a router starting at the given screen.
    ///
    /// If the screen is not a tab-root, `Home` is pushed onto the
    /// history stack so the back button works.
    pub fn with_initial_screen(
        screen: Screen,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let safe_area = Self::query_safe_area();

        let user_name = if cfg!(target_os = "ios") {
            "iOS"
        } else if cfg!(target_os = "android") {
            "Android"
        } else {
            "Mobile"
        };

        let mut history = Vec::new();
        if !screen.is_tab_root() {
            history.push(Screen::Search);
        }

        let search_state = cx.new(|cx| search::SearchState::new(window, cx));
        let anki_state = cx.new(|_| anki::AnkiRouter::new());
        let anki_settings_state = cx.new(|cx| anki_settings::AnkiSettingsState::new(window, cx));
        let global_yomichan = cx.global::<crate::GlobalYomichan>().clone();
        anki_settings::AnkiSettingsState::refresh_anki_data(
            &anki_settings_state,
            global_yomichan,
            cx,
        );

        let dictionaries_state = cx.new(|cx| dictionaries::DictionariesState::new(window, cx));
        let settings_state = cx.new(|cx| settings::SettingsState::new(window, cx));
        let zoom_header = cx.new(|_cx| {
            gpui_mobile::components::material::ZoomHeader::new(MaterialTheme::from_appearance(true))
        });

        Self {
            current_screen: screen,
            tap_count: 0,
            user_name: user_name.into(),
            dark_mode: true,
            font_size_multiplier: 1.2,
            history,
            safe_area,
            search_state,
            anki_state,
            anki_settings_state,
            dictionaries_state,
            settings_state,
            zoom_header,
            animation_playground: None,
            shader_showcase: None,
        }
    }

    /// Query the safe area insets from the platform.
    ///
    /// On Android, reads insets from the global `AndroidPlatform` via
    /// `jni`.  On iOS, safe area insets are managed by UIKit and
    /// will be queried once the iOS platform integration exposes them.
    ///
    /// Returns logical-pixel insets if available, otherwise zeros (no padding).
    fn query_safe_area() -> SafeArea {
        #[cfg(target_os = "android")]
        {
            use gpui_mobile::android::jni;
            if let Some(platform) = jni::platform() {
                if let Some(win) = platform.primary_window() {
                    let insets = win.safe_area_insets_logical();
                    log::info!(
                        "Router: safe area insets (logical px): top={:.1} bottom={:.1} left={:.1} right={:.1}",
                        insets.top, insets.bottom, insets.left, insets.right,
                    );
                    return SafeArea {
                        top: insets.top,
                        bottom: insets.bottom,
                        left: insets.left,
                        right: insets.right,
                    };
                }
            }
        }

        #[cfg(target_os = "ios")]
        {
            let (top, bottom, left, right) = gpui_mobile::safe_area_insets();
            if top > 0.0 || bottom > 0.0 {
                return SafeArea {
                    top,
                    bottom,
                    left,
                    right,
                };
            }
            // Fallback for before the window is ready
            return SafeArea {
                top: 59.0,
                bottom: 34.0,
                left: 0.0,
                right: 0.0,
            };
        }

        #[allow(unreachable_code)]
        SafeArea::default()
    }

    /// Navigate to a new screen, pushing the current one onto the history stack.
    pub fn navigate_to(&mut self, screen: Screen) {
        if self.current_screen != screen {
            // Dismiss webview when leaving the browser screen
            // if self.current_screen == Screen::WebViewBrowser {
            //     webview_browser::dismiss_webview();
            // }
            // // Dismiss video surface when leaving video player
            // if self.current_screen == Screen::VideoPlayer {
            //     video_player::dismiss();
            // }
            // Pause audio when leaving audio player
            if self.current_screen == Screen::AudioPlayer {
                audio_player::dismiss();
            }
            // Dismiss keyboard when leaving form or chat screens
            form::dismiss_form_keyboard();
            chat::dismiss_chat();
            if screen.is_tab_root() {
                // Switching to a tab-bar root screen — clear history so
                // the back button is not shown on primary destinations.
                self.history.clear();
            } else {
                self.history.push(self.current_screen);
            }
            self.current_screen = screen;

            // Lazily initialise demo state when first visited.
            match screen {
                Screen::Animations if self.animation_playground.is_none() => {
                    self.animation_playground = Some(AnimationPlayground::new());
                }
                Screen::Shaders if self.shader_showcase.is_none() => {
                    self.shader_showcase = Some(ShaderShowcase::new());
                }
                _ => {}
            }
        }
    }

    /// Go back to the previous screen. Returns `true` if navigation occurred.
    pub fn go_back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            // Dismiss webview when leaving browser
            // if self.current_screen == Screen::WebViewBrowser {
            //     webview_browser::dismiss_webview();
            // }
            // // Dismiss video surface when leaving video player
            // if self.current_screen == Screen::VideoPlayer {
            //     video_player::dismiss();
            // }
            // Pause audio when leaving audio player
            if self.current_screen == Screen::AudioPlayer {
                audio_player::dismiss();
            }
            // Dismiss keyboard when navigating back
            form::dismiss_form_keyboard();
            chat::dismiss_chat();
            self.current_screen = prev;
            true
        } else {
            false
        }
    }

    /// Whether the back button should be shown.
    ///
    /// Tab-bar root screens never show a back button, even if there
    /// is history (e.g. the user navigated Home → Counter → Home —
    /// history is cleared on tab switches so this is defensive).
    pub fn can_go_back(&self) -> bool {
        !self.current_screen.is_tab_root() && !self.history.is_empty()
    }
}

// ── Render ───────────────────────────────────────────────────────────────────

impl Render for Router {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // this renders a lot, every click since thats gpui works
        // , so uncomment if having rout switching problems
        //log::info!("Router: render() screen={:?}", self.current_screen);

        // Apply global font size multiplier
        window.set_rem_size(px(16.0 * self.font_size_multiplier));

        let show_tab_bar = self.current_screen.is_tab_root();
        let theme =
            gpui_mobile::components::material::MaterialTheme::from_appearance(self.dark_mode);

        // Sync zoom header theme
        let _ = self.zoom_header.update(cx, |this, cx| {
            this.set_theme(theme, cx);
        });

        let bg_color = theme.surface;
        let text_color = theme.on_surface;
        let safe_top = self.safe_area.top;
        let safe_bottom = self.safe_area.bottom;

        // ── Compute system chrome style ──────────────────────────────────
        let chrome = self.system_chrome_style();
        let top_color = chrome.status_bar_color.unwrap_or(bg_color);
        let bottom_color = chrome.navigation_bar_color.unwrap_or(bg_color);

        // Apply to the OS-level status bar / navigation bar.
        set_system_chrome(&chrome);

        let is_fullscreen_demo =
            matches!(self.current_screen, Screen::Animations | Screen::Shaders);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(bg_color))
            .text_color(rgb(text_color))
            // ── Top safe-area spacer (status bar / notch) ────────────────
            .when(safe_top > 0.0, |d| {
                d.child(div().w_full().h(px(safe_top)).bg(rgb(top_color)))
            })
            // ── Top app bar (iOS only, pushed below safe area) ───────────
            .when(cfg!(target_os = "ios") && !is_fullscreen_demo, |d| {
                d.child(self.render_top_bar(cx))
            })
            // ── Screen content ───────────────────────────────────────────
            .child(self.render_current_screen(window, cx))
            // ── Bottom tab bar (only for tab-root screens) ───────────────
            .when(show_tab_bar, |d| d.child(self.render_tab_bar(cx)))
            // ── Bottom safe-area spacer (nav bar / gesture indicator) ────
            .when(safe_bottom > 0.0 && show_tab_bar, |d| {
                d.child(div().w_full().h(px(safe_bottom)).bg(rgb(bottom_color)))
            })
            // ── Monokakido Zoom Header (Last child = Highest Z-Index) ─────
            .child(self.zoom_header.clone())
            .into_any_element()
    }
}

impl Router {
    /// Compute the system chrome style for the current screen and theme.
    ///
    /// Default: dark mode → dark status bar with light text; light mode → light
    /// status bar with dark text. Fullscreen demo screens override to dark chrome.
    fn system_chrome_style(&self) -> SystemChromeStyle {
        let is_fullscreen_demo =
            matches!(self.current_screen, Screen::Animations | Screen::Shaders);
        let theme =
            gpui_mobile::components::material::MaterialTheme::from_appearance(self.dark_mode);

        if is_fullscreen_demo {
            SystemChromeStyle {
                status_bar_color: Some(BASE),
                status_bar_style: StatusBarContentStyle::Light,
                navigation_bar_color: Some(BASE),
            }
        } else {
            SystemChromeStyle {
                status_bar_color: Some(theme.surface),
                status_bar_style: if self.dark_mode {
                    StatusBarContentStyle::Light
                } else {
                    StatusBarContentStyle::Dark
                },
                navigation_bar_color: Some(if self.current_screen.is_tab_root() {
                    theme.surface_container // matches NavigationBar
                } else {
                    theme.surface // no tab bar, match content bg
                }),
            }
        }
    }

    /// Render the content area for the currently active screen.
    ///
    /// Regular screens are wrapped in a scrollable container. Demo screens
    /// (Animations, Shaders) fill the remaining space with their own content
    /// and touch handlers.
    fn render_current_screen(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match self.current_screen {
            Screen::Animations => {
                return self
                    .render_animations_content(window, cx)
                    .into_any_element();
            }
            Screen::Shaders => {
                return self.render_shaders_content(window, cx).into_any_element();
            }
            _ => {}
        }

        let screen_content = match self.current_screen {
            Screen::Search => self.render_search_screen(cx).into_any_element(),
            Screen::Anki => self.render_anki_screen(cx).into_any_element(),
            Screen::AnkiSettings => self.render_anki_settings_screen(cx).into_any_element(),
            Screen::Dictionaries => self.render_dictionaries_screen(cx).into_any_element(),
            Screen::Settings => self.render_settings_screen(cx).into_any_element(),
            Screen::About => self.render_about_screen(cx).into_any_element(),
            _ => panic!("match self.current_screen doesn't have this one"),
        };

        div()
            .id("screen-scroll-container")
            .flex_1()
            .overflow_y_scroll()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_this, _event: &MouseDownEvent, _window, cx| {
                    let form_had_focus = form::has_focused_field();
                    let chat_had_focus = chat::CHAT_STATE.with(|s| s.borrow().focused);
                    if form_had_focus {
                        form::dismiss_form_keyboard();
                    }
                    if chat_had_focus {
                        chat::dismiss_chat();
                    }
                    if form_had_focus || chat_had_focus {
                        cx.notify();
                    }
                }),
            )
            .child(screen_content)
            .into_any_element()
    }

    /// Render the bottom tab bar using the Material Design navigation bar.
    ///
    /// Animations and Shaders are accessible from the Home screen nav cards
    /// instead of occupying bottom bar slots.
    fn render_tab_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.current_screen;
        let dark = self.dark_mode;

        NavigationBarBuilder::new(dark)
            .item(
                "⌕",
                "Search",
                current == Screen::Search,
                cx.listener(move |this, _, _, cx| {
                    this.navigate_to(Screen::Search);
                    cx.notify();
                }),
            )
            .item(
                "⌂",
                "Dicts",
                current == Screen::Dictionaries,
                cx.listener(move |this, _, _, cx| {
                    this.navigate_to(Screen::Dictionaries);
                    cx.notify();
                }),
            )
            .item(
                "⛩",
                "Anki",
                current == Screen::Anki,
                cx.listener(move |this, _, _, cx| {
                    this.navigate_to(Screen::Anki);
                    cx.notify();
                }),
            )
            .item(
                "☰",
                "Settings",
                current == Screen::Settings,
                cx.listener(move |this, _, _, cx| {
                    this.navigate_to(Screen::Settings);
                    cx.notify();
                }),
            )
            .item(
                "♥",
                "About",
                current == Screen::About,
                cx.listener(move |this, _, _, cx| {
                    this.navigate_to(Screen::About);
                    cx.notify();
                }),
            )
            .build()
    }

    // ── Per-screen render helpers ────────────────────────────────────────────

    fn render_search_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        search::render(&self.search_state, cx.entity().clone(), self, cx)
    }

    fn render_anki_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        anki::render(&self.anki_state, self, cx)
    }

    fn render_anki_settings_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        anki_settings::render(&self.anki_settings_state, self, cx)
    }

    fn render_dictionaries_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        dictionaries::render(&self.dictionaries_state, self, cx)
    }

    fn render_settings_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        settings::render(&self.settings_state, cx.entity().clone(), self, cx)
    }

    fn render_about_screen(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        about::render(self)
    }

    /// Render the top app bar with navigation controls and screen title.
    fn render_top_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme =
            gpui_mobile::components::material::MaterialTheme::from_appearance(self.dark_mode);
        let title = self.current_screen.title();

        let mut bar = TopAppBar::center_aligned(title, theme);

        if self.can_go_back() {
            bar = bar.leading_icon("←", cx.listener(|this, _, _, cx| {
                this.go_back();
                cx.notify();
            }));
        }

        bar.build()
    }

    fn render_animations_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        window.request_animation_frame();

        if self.animation_playground.is_none() {
            self.animation_playground = Some(AnimationPlayground::new());
        }

        let viewport = window.viewport_size();
        if let Some(playground) = &mut self.animation_playground {
            playground.set_bounds(Bounds {
                origin: point(0.0, 0.0),
                size: size(viewport.width.as_f32(), viewport.height.as_f32()),
            });
        }

        div()
            .flex_1()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                    if let Some(playground) = &mut this.animation_playground {
                        let pos = point(event.position.x.as_f32(), event.position.y.as_f32());
                        playground.touch_start = Some((pos, std::time::Instant::now()));
                        playground.current_touch = Some(pos);
                        cx.notify();
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if let Some(playground) = &mut this.animation_playground {
                    let pos = point(event.position.x.as_f32(), event.position.y.as_f32());
                    if playground.touch_start.is_none() {
                        playground.touch_start = Some((pos, std::time::Instant::now()));
                    }
                    playground.current_touch = Some(pos);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _window, cx| {
                    if let Some(playground) = &mut this.animation_playground {
                        let position = point(event.position.x.as_f32(), event.position.y.as_f32());

                        if let Some((start_pos, start_time)) = playground.touch_start.take() {
                            let elapsed = start_time.elapsed();
                            let dx = position.x - start_pos.x;
                            let dy = position.y - start_pos.y;
                            let distance = (dx * dx + dy * dy).sqrt();

                            if elapsed < std::time::Duration::from_millis(200) && distance < 20.0 {
                                let color_rgb = crate::demos::random_color(playground.next_ball_id);
                                playground.spawn_particles(position, rgb(color_rgb).into());
                                playground.next_ball_id += 1;
                            } else {
                                let dt = elapsed.as_secs_f32().max(0.01);
                                let velocity = point(dx / dt * 0.5, dy / dt * 0.5);
                                playground.spawn_ball(start_pos, velocity);
                            }
                        }
                        playground.current_touch = None;
                        cx.notify();
                    }
                }),
            )
            .child(if let Some(playground) = &mut self.animation_playground {
                playground.render_content(window).into_any_element()
            } else {
                div().into_any_element()
            })
    }

    fn render_shaders_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        window.request_animation_frame();

        if self.shader_showcase.is_none() {
            self.shader_showcase = Some(ShaderShowcase::new());
        }

        if let Some(showcase) = &mut self.shader_showcase {
            let viewport = window.viewport_size();
            showcase.set_screen_center(point(
                viewport.width.as_f32() / 2.0,
                viewport.height.as_f32() / 2.0,
            ));
        }

        div()
            .flex_1()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                    if let Some(showcase) = &mut this.shader_showcase {
                        let pos = point(event.position.x.as_f32(), event.position.y.as_f32());
                        showcase.touch_position = Some(pos);
                        showcase.spawn_ripple(pos);
                        cx.notify();
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if let Some(showcase) = &mut this.shader_showcase {
                    let pos = point(event.position.x.as_f32(), event.position.y.as_f32());
                    showcase.touch_position = Some(pos);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    if let Some(showcase) = &mut this.shader_showcase {
                        showcase.touch_position = None;
                        cx.notify();
                    }
                }),
            )
            .child(if let Some(showcase) = &mut self.shader_showcase {
                showcase.render_content(window).into_any_element()
            } else {
                div().into_any_element()
            })
    }
}
