use gpui::{
    div, Context, Entity, IntoElement, ParentElement, Render, Styled, InteractiveElement,
    WeakEntity, AsyncApp, rgb, prelude::*
};
use gpui_mobile::components::material::{MaterialTheme, TopAppBar, Dropdown, FilledButton};
use super::{Router};
use yomichan_rs::settings::core::{AnkiField, FieldIndex, AnkiTermFieldType};

pub struct AnkiSettingsState {
    pub deck_names: Vec<String>,
    pub model_names: Vec<String>,
    pub selected_deck: Option<usize>,
    pub selected_model: Option<usize>,
    pub model_fields: Vec<String>,
    pub field_mappings: Vec<Option<AnkiField>>,
    pub loading: bool,
    pub deck_dropdown_open: bool,
    pub model_dropdown_open: bool,
    pub field_dropdowns_open: Vec<bool>,
}

impl AnkiSettingsState {
    pub fn new(_window: &mut gpui::Window, _cx: &mut Context<Self>) -> Self {
        Self {
            deck_names: Vec::new(),
            model_names: Vec::new(),
            selected_deck: None,
            selected_model: None,
            model_fields: Vec::new(),
            field_mappings: Vec::new(),
            loading: true,
            deck_dropdown_open: false,
            model_dropdown_open: false,
            field_dropdowns_open: Vec::new(),
        }
    }

    pub fn apply_existing_mappings(&mut self, mappings: Vec<AnkiTermFieldType>) {
        self.field_mappings = vec![None; self.model_fields.len()];
        for mapping in mappings {
            match mapping {
                AnkiTermFieldType::Term(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::Term);
                    }
                }
                AnkiTermFieldType::Reading(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::Reading);
                    }
                }
                AnkiTermFieldType::Sentence(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::Sentence);
                    }
                }
                AnkiTermFieldType::Definition(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::Definition);
                    }
                }
                AnkiTermFieldType::TermAudio(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::TermAudio);
                    }
                }
                AnkiTermFieldType::SentenceAudio(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::SentenceAudio);
                    }
                }
                AnkiTermFieldType::Image(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::Image);
                    }
                }
                AnkiTermFieldType::Frequency(name) => {
                    if let Some(i) = self.model_fields.iter().position(|f| f == &name) {
                        self.field_mappings[i] = Some(AnkiField::Frequency);
                    }
                }
            }
        }
    }

    pub fn refresh_anki_data(
        this: &Entity<Self>,
        global_yomichan: crate::GlobalYomichan,
        cx: &mut Context<Router>,
    ) {
        let this = this.downgrade();
        let yomichan = global_yomichan.clone();

        cx.spawn(move |_router: WeakEntity<Router>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let yomichan = yomichan.clone();
            let this = this.clone();
            async move {
                let (decks, models, selected_deck, selected_model, model_fields, existing_mappings) = {
                    let ycd = yomichan.read();
                    let anki = ycd.anki();
                    let _ = anki.update_all_anki_maps();
                    
                    let decks = anki.deck_names();
                    let models = anki.model_names();
                    
                    let (sd, sm, mappings) = {
                        let profile_ptr = anki.options().read().get_current_profile().ok();
                        let fields = profile_ptr.and_then(|p| p.read().anki_options().anki_fields().clone());
                        (
                            fields.as_ref().map(|f| *f.selected_deck()), 
                            fields.as_ref().map(|f| *f.selected_model()),
                            fields.map(|f| f.fields().clone()).unwrap_or_default()
                        )
                    };
                    
                    let m_fields = sm.map(|idx| anki.field_names(idx)).unwrap_or_default();
                    
                    (decks, models, sd, sm, m_fields, mappings)
                };

                this.update(&mut cx, |s, cx| {
                    s.deck_names = decks;
                    s.model_names = models;
                    s.selected_deck = selected_deck;
                    s.selected_model = selected_model;
                    s.model_fields = model_fields;
                    s.apply_existing_mappings(existing_mappings);
                    
                    s.field_dropdowns_open = vec![false; s.model_fields.len()];
                    s.loading = false;
                    cx.notify();
                }).ok();
            }
        }).detach();
    }
    
    pub fn select_deck(this: &Entity<Self>, idx: usize, global_yomichan: crate::GlobalYomichan, cx: &mut Context<Router>) {
        let ycd = global_yomichan.read();
        let _ = ycd.anki().select_deck(idx);
        this.update(cx, |s, _| {
            s.selected_deck = Some(idx);
            s.deck_dropdown_open = false;
        });
        cx.notify();
    }

    pub fn select_model(this: &Entity<Self>, idx: usize, global_yomichan: crate::GlobalYomichan, cx: &mut Context<Router>) {
        let (fields, existing_mappings) = {
            let ycd = global_yomichan.read();
            let anki = ycd.anki();
            let _ = anki.select_model(idx);
            let model_fields = anki.field_names(idx);

            let mappings = {
                let profile = anki.options().read().get_current_profile().ok();
                let fields = profile.and_then(|p| p.read().anki_options().anki_fields().clone());
                fields.map(|f| f.fields().clone()).unwrap_or_default()
            };

            (model_fields, mappings)
        };
        
        this.update(cx, |s, _| {
            s.selected_model = Some(idx);
            s.model_fields = fields;
            s.apply_existing_mappings(existing_mappings);
            s.field_dropdowns_open = vec![false; s.model_fields.len()];
            s.model_dropdown_open = false;
        });
        cx.notify();
    }

    pub fn set_mapping(this: &Entity<Self>, field_idx: usize, field_type: AnkiField, cx: &mut Context<Router>) {
        this.update(cx, |s, _| {
            if field_idx < s.field_mappings.len() {
                s.field_mappings[field_idx] = Some(field_type);
                s.field_dropdowns_open[field_idx] = false;
            }
        });
        cx.notify();
    }

    pub fn save_settings(this: &Entity<Self>, global_yomichan: crate::GlobalYomichan, cx: &mut Context<Router>) {
        let state = this.read(cx);
        let mut mappings = Vec::new();
        for (i, mapping) in state.field_mappings.iter().enumerate() {
            if let Some(ft) = mapping {
                let m = match ft {
                    AnkiField::Term => FieldIndex::Term(i),
                    AnkiField::Reading => FieldIndex::Reading(i),
                    AnkiField::Sentence => FieldIndex::Sentence(i),
                    AnkiField::Definition => FieldIndex::Definition(i),
                    AnkiField::TermAudio => FieldIndex::TermAudio(i),
                    AnkiField::SentenceAudio => FieldIndex::SentenceAudio(i),
                    AnkiField::Image => FieldIndex::Image(i),
                    AnkiField::Frequency => FieldIndex::Frequency(i),
                };
                mappings.push(m);
            }
        }

        let ycd = global_yomichan.read();
        if let Err(e) = ycd.anki().set_field_mappings(&mappings) {
            log::error!("Failed to set field mappings: {:?}", e);
        }
        if let Err(e) = ycd.update_options() {
            log::error!("Failed to update options: {:?}", e);
        }
        log::info!("Anki settings saved");
    }
}

pub fn render(
    state: &Entity<AnkiSettingsState>,
    router: &Router,
    cx: &mut Context<Router>,
) -> impl IntoElement {
    let dark_mode = router.dark_mode;
    let theme = MaterialTheme::from_appearance(dark_mode);
    let state_read = state.read(cx);

    let mut app_bar = TopAppBar::small("Anki Settings", theme);
    if router.can_go_back() {
        app_bar = app_bar.leading_icon("«", cx.listener(|router: &mut Router, _, _, cx| {
            router.go_back();
            cx.notify();
        }));
    }

    if state_read.loading {
        return div().child("Loading...").into_any_element();
    }

    let deck_options = state_read.deck_names.clone();
    let model_options = state_read.model_names.clone();
    let global_yomichan = cx.global::<crate::GlobalYomichan>().clone();

    let available_fields = vec![
        ("Term", AnkiField::Term),
        ("Reading", AnkiField::Reading),
        ("Sentence", AnkiField::Sentence),
        ("Definition", AnkiField::Definition),
        ("Term Audio", AnkiField::TermAudio),
        ("Sentence Audio", AnkiField::SentenceAudio),
        ("Image", AnkiField::Image),
        ("Frequency", AnkiField::Frequency),
    ];

    div()
        .id("anki-settings")
        .flex()
        .flex_col()
        .size_full()
        .child(app_bar)
        .child(
            div()
                .id("anki-settings-scroll")
                .flex_1()
                .overflow_scroll()
                .flex()
                .flex_col()
                .p_4()
                .gap_4()
                .child(
            div()
                .child("Deck:")
                .child({
                    let mut dropdown = Dropdown::new(
                        state_read.selected_deck.and_then(|i| deck_options.get(i)).cloned().unwrap_or("Select Deck".to_string()),
                        theme
                    )
                    .open(state_read.deck_dropdown_open)
                    .on_toggle(cx.listener({
                        let state = state.clone();
                        move |_, _, _, cx| {
                            let _ = state.update(cx, |s, cx| {
                                s.deck_dropdown_open = !s.deck_dropdown_open;
                                cx.notify();
                            });
                        }
                    }));
                    
                    for (i, name) in deck_options.iter().enumerate() {
                        let name_val = name.clone();
                        let state = state.clone();
                        let gy = global_yomichan.clone();
                        dropdown = dropdown.item(&name_val, cx.listener(move |_, _, _, cx| {
                            AnkiSettingsState::select_deck(&state, i, gy.clone(), cx);
                        }));
                    }
                    dropdown
                })
        )
        .child(
            div()
                .child("Model:")
                .child({
                    let mut dropdown = Dropdown::new(
                        state_read.selected_model.and_then(|i| model_options.get(i)).cloned().unwrap_or("Select Model".to_string()),
                        theme
                    )
                    .open(state_read.model_dropdown_open)
                    .on_toggle(cx.listener({
                        let state = state.clone();
                        move |_, _, _, cx| {
                            let _ = state.update(cx, |s, cx| {
                                s.model_dropdown_open = !s.model_dropdown_open;
                                cx.notify();
                            });
                        }
                    }));
                    
                    for (i, name) in model_options.iter().enumerate() {
                        let name_val = name.clone();
                        let state = state.clone();
                        let gy = global_yomichan.clone();
                        dropdown = dropdown.item(&name_val, cx.listener(move |_, _, _, cx| {
                            AnkiSettingsState::select_model(&state, i, gy.clone(), cx);
                        }));
                    }
                    dropdown
                })
        )
        .child(if !state_read.model_fields.is_empty() {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child("Field Mapping:")
                .children(state_read.model_fields.iter().enumerate().map(|(i, field_name)| {
                    let state = state.clone();
                    let field_name = field_name.clone();
                    let current_mapping = state_read.field_mappings[i];
                    let dropdown_open = state_read.field_dropdowns_open[i];
                    
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(field_name)
                        .child({
                            let mut dropdown = Dropdown::new(
                                current_mapping.map(|f| format!("{:?}", f)).unwrap_or("None".to_string()),
                                theme
                            )
                            .open(dropdown_open)
                            .on_toggle(cx.listener({
                                let state = state.clone();
                                move |_, _, _, cx| {
                                    let _ = state.update(cx, |s, cx| {
                                        s.field_dropdowns_open[i] = !s.field_dropdowns_open[i];
                                        cx.notify();
                                    });
                                }
                            }));
                            
                            for (label, field_type) in available_fields.iter() {
                                let label = label.to_string();
                                let field_type = *field_type;
                                let state = state.clone();
                                dropdown = dropdown.item(&label, cx.listener(move |_, _, _, cx| {
                                    AnkiSettingsState::set_mapping(&state, i, field_type, cx);
                                }));
                            }
                            dropdown
                        })
                }))
                .child(
                    FilledButton::new("Save Mappings", theme)
                        .id("save-anki")
                        .on_click(cx.listener({
                            let state = state.clone();
                            let gy = global_yomichan.clone();
                            move |_, _, _, cx| {
                                AnkiSettingsState::save_settings(&state, gy.clone(), cx);
                            }
                        }))
                )
        } else {
            div()
        })
        )
        .into_any_element()
}
