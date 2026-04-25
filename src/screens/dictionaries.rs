use super::Router;
use crate::GlobalYomichan;
use gpui::{
    div, rgb, AppContext, AsyncApp, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement, Styled,
};
use gpui_mobile::components::material::MaterialTheme;
use gpui_mobile::packages::file_selector::{open_file, OpenFileOptions, TypeGroup};
use std::path::PathBuf;
use yomichan_rs::settings::core::DictionaryOptions;

pub struct DictionariesState {}

impl DictionariesState {
    pub fn new(_window: &mut gpui::Window, _cx: &mut Context<Self>) -> Self {
        Self {}
    }

    pub fn toggle_dictionary_enabled(dict_name: String, new_state: bool, cx: &mut Context<Router>) {
        let global_yomichan = cx.global::<GlobalYomichan>().clone();
        // Update in memory
        {
            let options_ptr = global_yomichan.read().options();
            let profile_ptr = options_ptr.read().get_current_profile().unwrap().clone();
            let mut profile_guard = profile_ptr.write();
            if let Some((_, dict_options)) = profile_guard
                .options_mut()
                .dictionaries_mut()
                .iter_mut()
                .find(|(_, opt)| opt.name == dict_name)
            {
                dict_options.enabled = new_state;
            }
        }

        // Persist
        cx.spawn(move |_, cx: &mut AsyncApp| {
            let global_yomichan = global_yomichan.clone();
            async move {
                let _ = global_yomichan.read().update_options();
            }
        })
        .detach();

        cx.notify();
    }

    pub fn remove_dictionary(name: String, cx: &mut Context<Router>) {
        log::info!("Starting removal of dictionary: {}", name);
        let global_yomichan = cx.global::<GlobalYomichan>().clone();
        {
            let ycd = global_yomichan.write();
            match ycd.remove_dictionary(&name) {
                Ok(_) => log::info!("Successfully removed dictionary: {}", name),
                Err(e) => log::error!("Failed to remove dictionary {}: {}", name, e),
            }
        }
        cx.notify();
    }

    pub fn set_language(lang: String, cx: &mut Context<Router>) {
        let global_yomichan = cx.global::<GlobalYomichan>().clone();
        {
            let ycd = global_yomichan.write();
            log::info!("Setting language to {}", lang);
            let _ = ycd.set_language(&lang);
        }
        // Save after dropping the write lock
        log::info!("Updating options");
        let _ = global_yomichan.read().update_options();
        cx.notify();
    }

    pub fn import_dictionary(
        dictionaries_state: &Entity<DictionariesState>,
        cx: &mut Context<Router>,
    ) {
        let options = OpenFileOptions {
            accept_type_groups: vec![TypeGroup {
                label: "Yomichan Dictionary".into(),
                extensions: vec!["zip".into()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let global_yomichan = cx.global::<GlobalYomichan>().clone();
        let dictionaries_state = dictionaries_state.clone();

        cx.spawn(move |_, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let global_yomichan = global_yomichan.clone();
            async move {
                match open_file(options).await {
                    Ok(Some(file)) => {
                        let path = PathBuf::from(file.path);
                        // import_dictionaries takes a slice of paths.
                        // We run it on the background executor to avoid freezing the UI.
                        log::info!("[import]: {:?}", path);

                        let weak_state = dictionaries_state.downgrade();
                        cx.spawn(move |cx: &mut AsyncApp| {
                            let mut cx = cx.clone();
                            let global_yomichan = global_yomichan.clone();
                            async move {
                                let result = cx
                                    .background_executor()
                                    .spawn({
                                        let global_yomichan = global_yomichan.clone();
                                        async move {
                                            global_yomichan.read().import_dictionaries(&[path])
                                        }
                                    })
                                    .await;

                                match result {
                                    Ok(_) => {
                                        log::info!("import completed!");
                                        let _ = global_yomichan.read().update_options();
                                        weak_state
                                            .update(
                                                &mut cx,
                                                |_, cx: &mut Context<'_, DictionariesState>| {
                                                    cx.notify();
                                                },
                                            )
                                            .ok();
                                    }
                                    Err(e) => log::error!("import failed:\n  {}", e),
                                }
                            }
                        })
                        .detach();
                    }
                    Err(e) => {
                        log::error!("File picker error: {}", e);
                    }
                    _ => {}
                }
            }
        })
        .detach();
    }
}

pub fn render(
    state: &Entity<DictionariesState>,
    router: &Router,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let theme = MaterialTheme::from_appearance(router.dark_mode);

    let global_yomichan = cx.global::<GlobalYomichan>().clone();
    let (dictionaries, current_lang) = {
        let ycd = global_yomichan.read();
        let options_ptr = ycd.options();
        let profile_ptr = options_ptr.read().get_current_profile().unwrap().clone();
        let profile = profile_ptr.read();
        (
            profile.options().dictionaries.clone(),
            profile.options().general().language.clone(),
        )
    };

    let state = state.clone();

    div()
        .id("dictionaries-page")
        .flex()
        .flex_col()
        .size_full()
        .gap_4()
        .px_4()
        .py_4()
        .overflow_y_scroll()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(theme.on_surface))
                        .child("Dictionaries"),
                )
                .child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(rgb(theme.primary))
                        .text_color(rgb(theme.on_primary))
                        .rounded_sm()
                        .child("Import")
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |_, _, _, cx| {
                                DictionariesState::import_dictionary(&state, cx);
                            }),
                        ),
                ),
        )
        // ── Language Section ─────────────────────────────────────────────
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.on_surface_variant))
                        .child("LANGUAGE"),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .child(lang_chip("Japanese", "ja", &current_lang, theme, cx))
                        .child(lang_chip("English", "en", &current_lang, theme, cx))
                        .child(lang_chip("Spanish", "es", &current_lang, theme, cx)),
                ),
        )
        // ── Dictionaries List ───────────────────────────────────────────
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .children(
                    dictionaries
                        .into_iter()
                        .map(|(_, opt): (String, DictionaryOptions)| {
                            render_dictionary_card(opt, theme, cx)
                        }),
                ),
        )
}

fn lang_chip(
    label: &str,
    iso: &str,
    current: &str,
    theme: MaterialTheme,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let active = current == iso;
    let iso = iso.to_string();
    div()
        .px_3()
        .py_1()
        .rounded_sm()
        .bg(rgb(if active {
            theme.primary_container
        } else {
            theme.surface_container_high
        }))
        .text_color(rgb(if active {
            theme.on_primary_container
        } else {
            theme.on_surface
        }))
        .text_sm()
        .child(label.to_string())
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |_, _, _, cx| {
                DictionariesState::set_language(iso.clone(), cx);
            }),
        )
}

fn render_dictionary_card(
    opt: DictionaryOptions,
    theme: MaterialTheme,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let name = opt.name.clone();
    let name_clone = name.clone();
    let enabled = opt.enabled;

    div()
        .flex()
        .flex_col()
        .bg(rgb(theme.surface_container_high))
        .p_4()
        .rounded_sm()
        .gap_3()
        .child(
            div()
                .text_lg()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(theme.on_surface))
                .child(name.clone()),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(rgb(if enabled {
                            theme.primary
                        } else {
                            theme.on_surface_variant
                        }))
                        .text_xs()
                        .text_color(rgb(if enabled {
                            theme.on_primary
                        } else {
                            theme.on_surface
                        }))
                        .child(if enabled { "ON" } else { "OFF" })
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |_, _, _, cx| {
                                DictionariesState::toggle_dictionary_enabled(
                                    name.clone(),
                                    !enabled,
                                    cx,
                                );
                            }),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(rgb(theme.error_container))
                        .text_xs()
                        .text_color(rgb(theme.on_error_container))
                        .child("REMOVE")
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |_, _, _, cx| {
                                DictionariesState::remove_dictionary(name_clone.clone(), cx);
                            }),
                        ),
                ),
        )
}
