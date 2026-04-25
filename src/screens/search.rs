use gpui::{
    div, px, rgb, App, AppContext, AsyncApp, Context, Div, Entity, InteractiveElement, IntoElement,
    MouseDownEvent, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Task,
    WeakEntity,
};
use gpui_mobile::components::material::search_bar::SearchBar;
use gpui_mobile::components::material::{Dropdown, MaterialTheme, SelectableTextView, TextField};
use gpui_mobile::{set_text_input_callback, show_keyboard};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use yomichan_rs::{SearchResult, TermDictionaryEntry, TermSearchResultsSegment};

use super::Router;
use crate::{GlobalPendingCards, GlobalYomichan};

pub static CLEANUP_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[[:punct:]]").unwrap());

pub struct SearchState {
    pub query: TextField,
    pub focused: bool,
    pub search_results: Option<SearchResult>,
    pub search_task: Option<Task<()>>,
    pub selected_term_index: Option<usize>,
    pub view_cache: HashMap<String, Entity<SelectableTextView>>,
    pub profile_dropdown_open: bool,
}

impl SearchState {
    pub fn new(_window: &mut gpui::Window, _cx: &mut Context<Self>) -> Self {
        Self {
            query: TextField::new(""),
            focused: false,
            search_results: None,
            search_task: None,
            selected_term_index: None,
            view_cache: HashMap::new(),
            profile_dropdown_open: false,
        }
    }

    fn queue_search(&mut self, term: &str, cx: &mut Context<Self>, immediate: bool) {
        self.search_task.take();

        let trimmed_term = term.trim();
        let clean_term = CLEANUP_REGEX.replace_all(trimmed_term, "");
        let term = clean_term.to_string();

        if term.is_empty() {
            self.search_results = None;
            cx.notify();
            return;
        }

        let new_search_task = cx.spawn(
            move |this_handle: WeakEntity<SearchState>, cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    if !immediate {
                        let _ = portable_async_sleep::async_sleep(
                            std::time::Duration::from_millis(200),
                        )
                        .await;
                    }

                    let ycd = cx.read_global(|g: &GlobalYomichan, _cx| g.0.clone());

                    let results = cx
                        .background_executor()
                        .spawn(async move { ycd.write().search(&term) })
                        .await;

                    let _ = this_handle.update(&mut cx, |this, cx| {
                        this.search_results = results.ok();
                        this.view_cache.clear();
                        if this.selected_term_index.is_none() && this.search_results.is_some() {
                            this.selected_term_index = Some(0);
                        }
                        cx.notify();
                    });
                }
            },
        );
        self.search_task = Some(new_search_task);
    }

    pub fn get_or_create_view(
        &mut self,
        key: &str,
        text: SharedString,
        theme: MaterialTheme,
        lookup_handler: impl Fn(&str, &mut App) + 'static,
        cx: &mut impl AppContext,
    ) -> Entity<SelectableTextView> {
        if let Some(view) = self.view_cache.get(key) {
            return view.clone();
        }

        let view = cx.new(|cx| {
            let mut view = SelectableTextView::new(text, theme, cx);
            view.on_lookup(lookup_handler);
            view
        });
        self.view_cache.insert(key.to_string(), view.clone());
        view
    }
}

pub fn render(
    search_state: &Entity<SearchState>,
    router_handle: Entity<super::Router>,
    router: &super::Router,
    cx: &mut Context<super::Router>,
) -> impl IntoElement {
    super::form::drain_pending_text();
    let dark_mode = router.dark_mode;
    let theme = MaterialTheme::from_appearance(dark_mode);

    let (query, cursor_pos, focused, results, selected_index, profile_open): (
        String,
        usize,
        bool,
        Option<SearchResult>,
        Option<usize>,
        bool,
    ) = {
        let state = search_state.read(cx);
        (
            state.query.text.clone(),
            state.query.cursor,
            state.focused,
            state.search_results.clone(),
            state.selected_term_index,
            state.profile_dropdown_open,
        )
    };

    let global_yomichan = cx.global::<GlobalYomichan>().clone();
    let (profiles, current_profile_idx) = {
        let ycd = global_yomichan.read();
        let opts_ptr = ycd.options();
        let opts = opts_ptr.read();
        (
            opts.profiles.keys().cloned().collect::<Vec<String>>(),
            opts.current_profile,
        )
    };
    let current_profile_name = profiles
        .get(current_profile_idx)
        .cloned()
        .unwrap_or_else(|| "Default".to_string());

    let search_state_handle = search_state.clone();
    let search_state_handle_for_trailing = search_state.clone();

    div()
        .id("search-page")
        .flex()
        .flex_col()
        .size_full()
        .gap_2()
        .px_2()
        .py_2()
        .child(
            SearchBar::new(theme)
                .query(query.clone())
                .cursor(cursor_pos)
                .focused(focused)
                .leading_element({
                    let mut dropdown = Dropdown::new(
                        current_profile_name
                            .chars()
                            .next()
                            .unwrap_or('P')
                            .to_string(),
                        theme,
                    )
                    .open(profile_open)
                    .on_toggle(cx.listener({
                        let search_state = search_state.clone();
                        move |_, _, _, cx| {
                            search_state.update(cx, |s, cx| {
                                s.profile_dropdown_open = !s.profile_dropdown_open;
                                cx.notify();
                            });
                        }
                    }));

                    for name in &profiles {
                        let name_clone = name.clone();
                        let search_state = search_state.clone();
                        let router_handle = router_handle.clone();
                        dropdown = dropdown.item(name, move |_, _, cx| {
                            super::settings::SettingsState::switch_profile(
                                name_clone.clone(),
                                &router_handle,
                                cx,
                            );
                            let _ = search_state.update(cx, |state, cx| {
                                state.profile_dropdown_open = false;
                                state.view_cache.clear();
                                cx.notify();
                            });
                        });
                    }
                    dropdown
                })
                .on_tap(cx.listener(move |_, event: &MouseDownEvent, _, cx| {
                    let search_state_handle = search_state_handle.clone();
                    let async_cx = cx.to_async();

                    // Position cursor accurately from tap
                    let _ = search_state_handle.update(cx, |state, cx| {
                        state.focused = true;
                        // SearchBar text starts at approx x=56 (icon + gap)
                        let text_x = event.position.x.as_f32() - 56.0;
                        state.query.cursor =
                            gpui_mobile::components::material::text_input::calculate_cursor_offset(
                                &state.query.text,
                                px(16.0).into(), // SearchBar text is base size
                                text_x,
                                cx,
                            );
                        cx.notify();
                    });

                    show_keyboard();
                    set_text_input_callback(Some(Box::new(move |text| {
                        if text == "\n" {
                            gpui_mobile::hide_keyboard();
                            return;
                        }

                        let search_state_handle = search_state_handle.clone();
                        let text = text.to_string();
                        let async_cx = async_cx.clone();

                        async_cx.update(move |cx| {
                            let _ = search_state_handle.update(cx, |state, cx| {
                                if text == "\x08" {
                                    state.query.delete_at_cursor();
                                } else {
                                    state.query.insert_at_cursor(&text);
                                }
                                let q = state.query.text.clone();
                                state.queue_search(&q, cx, true);
                                cx.notify();
                            });
                        });
                    })));
                }))
                .on_trailing_tap(cx.listener(move |_, _, _, cx| {
                    search_state_handle_for_trailing.update(cx, |state, cx| {
                        state.query.text.clear();
                        state.query.cursor = 0;
                        state.query.selection = None;
                        state.search_results = None;
                        state.selected_term_index = None;
                        state.view_cache.clear();
                        cx.notify();
                    });
                })),
        )
        .child(render_segment_selector(search_state, theme, cx))
        .child(
            div()
                .id("search-results")
                .flex_1()
                .overflow_scroll()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener({
                        let search_state = search_state.clone();
                        move |_, _, _, cx| {
                            search_state.update(cx, |s, cx| {
                                s.focused = false;
                                cx.notify();
                            });
                            gpui_mobile::hide_keyboard();
                        }
                    }),
                )
                .child(if let Some(results) = results {
                    if let Some(selected_index) = selected_index {
                        if let Some(segment) = results.segments.get(selected_index) {
                            let search_state = search_state.clone();
                            let mut dictionary_entries = Vec::new();
                            for (entry_idx, entry) in segment.entries.iter().enumerate() {
                                dictionary_entries.push(render_dictionary_entry(
                                    entry,
                                    entry_idx,
                                    theme,
                                    &search_state,
                                    cx,
                                ));
                            }
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .px_2()
                                .children(dictionary_entries)
                        } else {
                            div().px_4().child("Select a word above")
                        }
                    } else {
                        div().px_4().child("Select a word above")
                    }
                } else {
                    div().px_4().child("No results found.")
                }),
        )
}

struct SegmentSelector {
    search_state: Entity<SearchState>,
    segments: Vec<TermSearchResultsSegment>,
}
impl SegmentSelector {
    fn new(search_state: &Entity<SearchState>, segments: Vec<TermSearchResultsSegment>) -> Self {
        Self {
            search_state: search_state.clone(),
            segments,
        }
    }
}
// impl Render for SegmentSelector {}

fn render_segment_selector(
    search_state: &Entity<SearchState>,
    theme: MaterialTheme,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let (results, selected_index) = {
        let state = search_state.read(cx);
        (state.search_results.clone(), state.selected_term_index)
    };
    match results {
        Some(results) => div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_2()
            .px_2()
            .py_2()
            .children(
                results
                    .segments
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, segment)| {
                        if segment.text.trim().is_empty() {
                            return None;
                        }

                        let is_selected = Some(i) == selected_index;
                        let search_state_handle = search_state.clone();
                        let text = segment.text;

                        Some(render_segment(
                            i,
                            text.clone(),
                            is_selected,
                            theme,
                            search_state.clone(),
                            cx,
                        ))
                    }),
            ),
        None => div(),
    }
}

fn render_segment(
    index: usize,
    text: String,
    is_selected: bool,
    theme: MaterialTheme,
    search_state: Entity<SearchState>,
    cx: &mut Context<Router>,
) -> Div {
    div()
        .px_2()
        .py_0p5()
        .rounded_sm()
        .bg(rgb(if is_selected {
            theme.primary_container
        } else {
            theme.surface_container_high
        }))
        .text_color(rgb(if is_selected {
            theme.on_primary_container
        } else {
            theme.on_surface
        }))
        .text_sm()
        .font_weight(gpui::FontWeight::BOLD)
        .child(text)
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |_, _, _, cx| {
                let _ = search_state.update(cx, |state, cx| {
                    state.selected_term_index = Some(index);
                    state.view_cache.clear();
                    cx.notify();
                });
            }),
        )
}

fn add_to_anki_btn(
    entry: &TermDictionaryEntry,
    cx: &mut Context<Router>,
    theme: MaterialTheme,
) -> Div {
    let entry_clone = entry.clone();
    let pending_cards = cx.global::<GlobalPendingCards>().clone();
    div()
        .h(px(35.0))
        .px_0p5()
        .py_0p5()
        .rounded_sm()
        .bg(rgb(theme.outline))
        .text_color(rgb(theme.on_primary))
        .child("🃏")
        .hover(|sf| sf.opacity(0.5))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |_, _, _, _| {
                let mut cards = pending_cards.write();
                cards.push(entry_clone.clone());
                log::info!("Added entry to Anki drafts");
            }),
        )
}

fn render_dictionary_entry(
    entry: &yomichan_rs::TermDictionaryEntry,
    entry_idx: usize,
    theme: MaterialTheme,
    search_state: &Entity<SearchState>,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let headword = entry.headwords.first();
    let term = headword.map(|h| h.term.clone()).unwrap_or_default();
    let reading = headword.map(|h| h.reading.clone()).unwrap_or_default();

    let search_state_handle_for_lookup = search_state.clone();
    let lookup_handler = move |text: &str, cx: &mut App| {
        let text = text.to_string();
        let _ = search_state_handle_for_lookup.update(cx, |state, cx| {
            state.query.text = text.clone();
            state.query.cursor = text.len();
            state.queue_search(&text, cx, true);
            cx.notify();
        });
    };

    let search_state_handle = search_state.clone();

    let mut definitions = Vec::new();
    for (def_idx, def) in entry.definitions.iter().enumerate() {
        for (gloss_idx, gloss) in def.entries.iter().enumerate() {
            let lookup_handler = lookup_handler.clone();
            let key = format!("entry-{}-def-{}-gloss-{}", entry_idx, def_idx, gloss_idx);
            let text = SharedString::from(gloss.plain_text.clone());

            let view = search_state_handle.update(cx, |state: &mut SearchState, cx| {
                state.get_or_create_view(&key, text, theme, lookup_handler, cx)
            });

            definitions.push(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .text_lg()
                    .text_color(rgb(theme.on_surface))
                    .child(view),
            );
        }
    }

    let reading_key = format!("entry-{}-reading", entry_idx);
    let term_key = format!("entry-{}-term", entry_idx);

    let reading_view = search_state_handle.update(cx, |state: &mut SearchState, cx| {
        state.get_or_create_view(
            &reading_key,
            SharedString::from(reading),
            theme,
            lookup_handler.clone(),
            cx,
        )
    });

    let term_view = search_state_handle.update(cx, |state: &mut SearchState, cx| {
        state.get_or_create_view(
            &term_key,
            SharedString::from(term),
            theme,
            lookup_handler.clone(),
            cx,
        )
    });

    let add_to_anki_btn = add_to_anki_btn(entry, cx, theme);

    div()
        .flex()
        .flex_col()
        .bg(rgb(theme.surface_container_high))
        .p_4()
        .rounded_sm()
        .gap_2()
        .child(
            div()
                .flex()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_lg()
                                .text_color(rgb(theme.secondary))
                                .child(reading_view),
                        )
                        .child(
                            div()
                                .text_3xl()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(theme.primary))
                                .child(term_view),
                        ),
                )
                .child(add_to_anki_btn),
        )
        .children(definitions)
}
