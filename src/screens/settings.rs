use gpui::{
    Context, Entity, IntoElement, ParentElement, Styled,
    div, rgb, InteractiveElement, px, prelude::*,
};
use gpui_mobile::components::material::{MaterialTheme, Dropdown, PopupModal, ModalPosition, FilledTonalButton, TextButton, FilledButton, TextInput};
use gpui_mobile::KeyboardType;
use crate::GlobalYomichan;
use super::Router;

pub struct SettingsState {
    pub profile_dropdown_open: bool,
    pub show_add_profile_modal: bool,
    pub new_profile_name: gpui_mobile::components::material::TextField,
}

impl SettingsState {
    pub fn new(_window: &mut gpui::Window, _cx: &mut Context<Self>) -> Self {
        Self {
            profile_dropdown_open: false,
            show_add_profile_modal: false,
            new_profile_name: gpui_mobile::components::material::TextField::new(""),
        }
    }

    pub fn switch_profile(name: String, router: &Entity<super::Router>, cx: &mut gpui::App) {
        log::info!("Switching profile to: {}", name);
        let global_yomichan = cx.global::<GlobalYomichan>().clone();
        {
            let ycd = global_yomichan.read();
            let opts = ycd.options();
            {
                let mut opts_guard = opts.write();
                if let Some(idx) = opts_guard.profiles.get_index_of(&name) {
                    log::info!("Found profile index: {}", idx);
                    opts_guard.current_profile = idx;
                } else {
                    log::warn!("Profile not found: {}", name);
                }
            }
            let _ = ycd.update_options();
        }
        
        // Clear search view cache to ensure it re-renders with new profile settings
        let _ = router.update(cx, |r, cx| {
            let _ = r.search_state.update(cx, |s, cx| {
                s.view_cache.clear();
                cx.notify();
            });
            cx.notify();
        });
    }

    pub fn add_profile(&mut self, cx: &mut Context<Self>) {
        if self.new_profile_name.text.is_empty() {
            return;
        }

        let name = self.new_profile_name.text.clone();
        log::info!("Adding profile: {}", name);
        let global_yomichan = cx.global::<GlobalYomichan>().clone();
        {
            let ycd = global_yomichan.read();
            let opts = ycd.options();
            {
                let mut opts_guard = opts.write();

                if !opts_guard.profiles.contains_key(&name) {
                    log::info!("Profile {} does not exist, creating it.", name);
                    let current_profile = opts_guard.profiles.get_index(opts_guard.current_profile).map(|(_, v)| v.clone()).unwrap_or_default();
                    opts_guard.profiles.insert(name.clone(), current_profile);
                    if let Some(idx) = opts_guard.profiles.get_index_of(&name) {
                        log::info!("Created and switching to profile index: {}", idx);
                        opts_guard.current_profile = idx;
                    }
                } else {
                    log::warn!("Profile {} already exists.", name);
                }
            }
            let _ = ycd.update_options();
        }
        self.new_profile_name.text.clear();
        self.new_profile_name.cursor = 0;
        self.show_add_profile_modal = false;
        gpui_mobile::hide_keyboard();
        cx.notify();
    }

    pub fn nuke_database(cx: &mut Context<Router>) {
        let data_dir = gpui_mobile::packages::path_provider::support_directory()
            .or_else(|_| gpui_mobile::packages::path_provider::documents_directory())
            .unwrap();
        
        log::info!("NUKING DATABASE at {:?}", data_dir);
        
        match yomichan_rs::Yomichan::nuke_database(&data_dir) {
            Ok(_) => {
                log::info!("Successfully nuked database. Quitting app for restart.");
                cx.quit();
            },
            Err(e) => log::error!("Failed to nuke database: {}", e),
        }
        
        cx.notify();
    }
}

pub fn render(
    state: &Entity<SettingsState>,
    router_handle: Entity<super::Router>,
    router: &super::Router,
    cx: &mut Context<super::Router>,
) -> impl IntoElement {
    let dark_mode = router.dark_mode;
    let theme = MaterialTheme::from_appearance(dark_mode);
    let text_color = theme.on_surface;
    let sub_text = theme.on_surface_variant;
    let card_bg = theme.surface_container_high;

    let global_yomichan = cx.global::<GlobalYomichan>().clone();
    let (profiles, current_profile_idx) = {
        let ycd = global_yomichan.read();
        let opts_ptr = ycd.options();
        let opts = opts_ptr.read();
        (opts.profiles.keys().cloned().collect::<Vec<String>>(), opts.current_profile)
    };

    let current_profile_name = profiles.get(current_profile_idx).cloned().unwrap_or_else(|| "Default".to_string());

    let state_read = state.read(cx);
    let show_modal = state_read.show_add_profile_modal;

    div()
        .id("settings-page")
        .relative()
        .flex()
        .flex_col()
        //.size_full()
        .overflow_y_scroll()
        .child(
            div()
                .flex()
                .flex_col()
                .size_full()
                .gap_4()
                .px_4()
                .py_6()
                .overflow_y_hidden()
                .child(section_header("Appearance", sub_text))
                .child(
                    settings_card(card_bg)
                        .child(toggle_row(
                            "Dark Mode",
                            "Use a dark colour scheme",
                            dark_mode,
                            text_color,
                            sub_text,
                            theme,
                            cx.listener(|this, _, _, cx| {
                                this.dark_mode = !this.dark_mode;
                                cx.notify();
                            }),
                        ))
                        .child(div().h_px().bg(rgb(theme.surface_container_low)))
                        .child(action_row(
                            "Font Size",
                            &format!("{:.1}x multiplier", router.font_size_multiplier),
                            false,
                            theme,
                            cx.listener(|this, _, _, cx| {
                                if this.font_size_multiplier >= 1.5 {
                                    this.font_size_multiplier = 1.0;
                                } else {
                                    this.font_size_multiplier += 0.1;
                                }
                                cx.notify();
                            }),
                        ))
                )
                .child(section_header("Profiles", sub_text))
                .child(
                    settings_card(card_bg)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_3()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(div().text_base().text_color(rgb(text_color)).child("Active Profile"))
                                        .child(div().text_xs().text_color(rgb(sub_text)).child("Switch or add profiles"))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child({
                                            let mut dropdown = Dropdown::new(current_profile_name, theme)
                                                .open(state_read.profile_dropdown_open)
                                                .on_toggle(cx.listener({
                                                    let state = state.clone();
                                                    move |_, _, _, cx| {
                                                        state.update(cx, |s, cx| {
                                                            s.profile_dropdown_open = !s.profile_dropdown_open;
                                                            cx.notify();
                                                        });
                                                    }
                                                }));
                                            
                                            for name in &profiles {
                                                let name_clone = name.clone();
                                                let state = state.clone();
                                                let router_handle = router_handle.clone();
                                                dropdown = dropdown.item(name, move |_, _, cx| {
                                                    SettingsState::switch_profile(name_clone.clone(), &router_handle, cx);
                                                    let _ = state.update(cx, |s, cx| {
                                                        s.profile_dropdown_open = false;
                                                        cx.notify();
                                                    });
                                                });
                                            }
                                            dropdown
                                        })
                                        .child(
                                            FilledTonalButton::new("+", theme)
                                                .on_click(cx.listener({
                                                    let state = state.clone();
                                                    let router_handle = router_handle.clone();
                                                    move |_, _, _, cx| {
                                                        let async_cx = cx.to_async();
                                                        state.update(cx, |s, cx| {
                                                            s.show_add_profile_modal = true;

                                                            // WORKAROUND: Programmatically focus and show keyboard
                                                            gpui_mobile::show_keyboard();

                                                            let state = state.clone();
                                                            let router_handle = router_handle.clone();
                                                            gpui_mobile::set_text_input_callback(Some(Box::new(move |text| {
                                                                let state = state.clone();
                                                                let router_handle = router_handle.clone();
                                                                let text = text.to_string();
                                                                let _ = async_cx.update(move |cx| {
                                                                    let _ = state.update(cx, |s, cx| {
                                                                        if text == "\x08" {
                                                                            s.new_profile_name.delete_at_cursor();
                                                                        } else if text != "\n" {
                                                                            s.new_profile_name.insert_at_cursor(&text);
                                                                        }
                                                                        cx.notify();
                                                                    });
                                                                    let _ = router_handle.update(cx, |r, cx| {
                                                                        let _ = r.search_state.update(cx, |_, cx| cx.notify());
                                                                        cx.notify();
                                                                    });
                                                                });
                                                            })));

                                                            cx.notify();
                                                        });
                                                    }
                                                }))
                                        )                                )
                        )
                )
                .child(section_header("Developer", sub_text))
                .child(
                    settings_card(card_bg)
                        .child(action_row(
                            "Nuke Database",
                            "Delete all dictionary data and settings",
                            false,
                            theme,
                            cx.listener(|_, _, _, cx| {
                                SettingsState::nuke_database(cx);
                            }),
                        ))
                )
                .child(
                    div()
                        .mt_4()
                        .text_xs()
                        .text_center()
                        .text_color(rgb(sub_text))
                        .child("Ported from ycd-rs")
                )
        )
        .when(show_modal, |this| {
            this.child(render_add_profile_modal(state, router_handle, theme, cx))
        })
}

fn render_add_profile_modal(
    state: &Entity<SettingsState>,
    router_handle: Entity<super::Router>,
    theme: MaterialTheme,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let state_read = state.read(cx);
    let name_value = state_read.new_profile_name.text.clone();
    let cursor_pos = state_read.new_profile_name.cursor;

    PopupModal::new(theme)
        .position(ModalPosition::Center)
        .on_close({
            let state = state.clone();
            move |_, cx| {
                let _ = state.update(cx, |s, cx| {
                    s.show_add_profile_modal = false;
                    cx.notify();
                });
            }
        })
        .child(
            div()
                .w(px(300.0))
                .p_6()
                .rounded_sm()
                .bg(rgb(theme.surface_container_high))
                .flex()
                .flex_col()
                .gap_4()
                .child(div().text_lg().text_color(rgb(theme.on_surface)).child("Add Profile"))
                .child(
                    TextInput::new("profile-name", theme)
                        .label("Profile Name")
                        .placeholder("Enter name...")
                        .value(&name_value)
                        .cursor(cursor_pos)
                        .focused(true)
                        .on_tap_notify({
                            let state = state.clone();
                            let router_handle = router_handle.clone();
                            let async_cx = cx.to_async();
                            move |event, cx| {
                                let state = state.clone();
                                let router_handle = router_handle.clone();
                                let async_cx = async_cx.clone();
                                
                                // Position cursor
                                let _ = state.update(cx, |s, cx| {
                                    s.new_profile_name.cursor = gpui_mobile::components::material::text_input::calculate_cursor_offset(
                                        &s.new_profile_name.text,
                                        px(14.0).into(),
                                        event.position.x.as_f32() - 12.0, // TEXT_START_X
                                        cx
                                    );
                                    cx.notify();
                                });

                                gpui_mobile::show_keyboard_with_type(KeyboardType::Default);
                                gpui_mobile::set_text_input_callback(Some(Box::new(move |text| {
                                    let state = state.clone();
                                    let router_handle = router_handle.clone();
                                    let text = text.to_string();
                                    let _ = async_cx.update(move |cx| {
                                        let _ = state.update(cx, |s, cx| {
                                            if text == "\x08" {
                                                s.new_profile_name.delete_at_cursor();
                                            } else if text != "\n" {
                                                s.new_profile_name.insert_at_cursor(&text);
                                            }
                                            cx.notify();
                                        });
                                        let _ = router_handle.update(cx, |r, cx| {
                                            let _ = r.search_state.update(cx, |_, cx| cx.notify());
                                            cx.notify();
                                        });
                                    });
                                })));
                            }
                        })
                        .render(cx)
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap_2()
                        .child(
                            TextButton::new("Cancel", theme)
                                .on_click(cx.listener({
                                    let state = state.clone();
                                    move |_, _, _, cx| {
                                        state.update(cx, |s, cx| {
                                            s.show_add_profile_modal = false;
                                            gpui_mobile::hide_keyboard();
                                            cx.notify();
                                        });
                                    }
                                }))
                        )
                        .child(
                            FilledButton::new("Create", theme)
                                .on_click(cx.listener({
                                    let state = state.clone();
                                    move |_, _, _, cx| {
                                        state.update(cx, |s, cx| {
                                            s.add_profile(cx);
                                        });
                                    }
                                }))
                        )
                )
        )
}

fn section_header(title: &str, color_val: u32) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(rgb(color_val))
        .px_1()
        .child(title.to_string().to_uppercase())
}

fn settings_card(bg: u32) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .rounded_sm()
        .bg(rgb(bg))
}

fn toggle_row(
    title: &str,
    description: &str,
    is_on: bool,
    text_color: u32,
    sub_text: u32,
    theme: MaterialTheme,
    handler: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let toggle_bg = if is_on { theme.primary } else { theme.on_surface_variant };
    let toggle_label = if is_on { "ON" } else { "OFF" };
    let toggle_text = if is_on { theme.on_primary } else { theme.on_surface };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .py_3()
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(text_color))
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(sub_text))
                        .child(description.to_string()),
                ),
        )
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(rgb(toggle_bg))
                .text_xs()
                .text_color(rgb(toggle_text))
                .child(toggle_label),
        )
        .on_mouse_down(gpui::MouseButton::Left, handler)
}

fn action_row(
    title: &str,
    description: &str,
    active: bool,
    theme: MaterialTheme,
    handler: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .py_3()
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(if active { theme.primary } else { theme.on_surface }))
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.on_surface_variant))
                        .child(description.to_string()),
                ),
        )
        .child(if active {
            div().text_sm().text_color(rgb(theme.primary)).child("✓")
        } else {
            div().text_sm().text_color(rgb(theme.on_surface_variant)).child("→")
        })
        .on_mouse_down(gpui::MouseButton::Left, handler)
}
