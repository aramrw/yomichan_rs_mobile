use crate::{GlobalPendingCards, GlobalYomichan};
use gpui::{
    div, prelude::*, rgb, AsyncApp, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, WeakEntity,
};
use gpui_mobile::components::material::{FilledButton, MaterialTheme, TopAppBar};

use super::{Router, Screen};

pub struct AnkiRouter {
    pub pushing: bool,
}

impl AnkiRouter {
    pub fn new() -> Self {
        Self { pushing: false }
    }

    pub fn push_all_to_anki(
        this: &Entity<Self>,
        global_yomichan: GlobalYomichan,
        pending_cards: GlobalPendingCards,
        cx: &mut Context<Router>,
    ) {
        let this_weak = this.downgrade();
        let yomichan = global_yomichan.clone();
        let cards_handle = pending_cards.clone();

        let _ = this.update(cx, |s, cx| {
            s.pushing = true;
            cx.notify();
        });

        cx.spawn(move |_router: WeakEntity<Router>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let yomichan = yomichan.clone();
            let cards_handle = cards_handle.clone();
            let this = this_weak.clone();

            async move {
                let cards = cards_handle.read().clone();
                let mut success_count = 0;
                let mut errors = Vec::new();

                {
                    let ycd = yomichan.read();
                    let anki = ycd.anki();
                    for entry in &cards {
                        match anki.add_entry(entry, None) {
                            Ok(_) => success_count += 1,
                            Err(e) => errors.push(e.to_string()),
                        }
                    }
                }

                if errors.is_empty() {
                    cards_handle.write().clear();
                    log::info!("Successfully pushed {} cards to Anki", success_count);
                } else {
                    log::error!("Errors pushing to Anki: {:?}", errors);
                }

                this.update(&mut cx, |s, cx| {
                    s.pushing = false;
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }
}

pub fn render(
    state: &Entity<AnkiRouter>,
    router: &Router,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let dark_mode = router.dark_mode;
    let theme = MaterialTheme::from_appearance(dark_mode);
    let pending_cards = cx.global::<GlobalPendingCards>().clone();
    let drafts = pending_cards.read().clone();
    let pushing = state.read(cx).pushing;

    let mut list = Vec::new();
    for entry in &drafts {
        let term = entry
            .headwords
            .first()
            .map(|h| h.term.clone())
            .unwrap_or_default();
        let reading = entry
            .headwords
            .first()
            .map(|h| h.reading.clone())
            .unwrap_or_default();
        list.push(
            div()
                .flex()
                .flex_col()
                .p_3()
                .bg(rgb(theme.surface_container))
                .rounded_sm()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.secondary))
                        .child(reading),
                )
                .child(div().font_weight(gpui::FontWeight::BOLD).child(term)),
        );
    }

    let mut app_bar = TopAppBar::small("Anki Drafts", theme).trailing_icon(
        "☰",
        cx.listener(|router: &mut Router, _, _, cx| {
            router.navigate_to(Screen::AnkiSettings);
            cx.notify();
        }),
    );

    if router.can_go_back() {
        app_bar = app_bar.leading_icon(
            "«",
            cx.listener(|router: &mut Router, _, _, cx| {
                router.go_back();
                cx.notify();
            }),
        );
    }

    let global_yomichan = cx.global::<GlobalYomichan>().clone();

    div()
        .id("anki-router")
        .flex()
        .flex_col()
        .size_full()
        .child(app_bar)
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .p_4()
                .gap_4()
                .child(
                    div()
                        .id("drafts-list")
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .overflow_scroll()
                        .children(if list.is_empty() {
                            vec![div()
                                .child("No drafts yet. Go to Search to add some!")
                                .into_any_element()]
                        } else {
                            list.into_iter().map(|e| e.into_any_element()).collect()
                        }),
                )
                .child(
                    FilledButton::new(
                        if pushing {
                            "Pushing..."
                        } else {
                            "Push to Anki"
                        },
                        theme,
                    )
                    .id("push-btn")
                    .disabled(pushing || drafts.is_empty())
                    .on_click(cx.listener({
                        let state = state.clone();
                        let gy = global_yomichan.clone();
                        let pc = pending_cards.clone();
                        move |_, _, _, cx| {
                            AnkiRouter::push_all_to_anki(&state, gy.clone(), pc.clone(), cx);
                        }
                    })),
                ),
        )
        .into_any_element()
}
