//! Attacking entry batches and source-independent, owned next-end-step abilities.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedSacrificeView {
    pub destroy: bool,
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub tokens: Vec<ObjectRef>,
}
impl DelayedSacrificeView {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.tokens.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedBatch {
    pub view: DelayedSacrificeView,
    keywords: Keywords,
    tapped: bool,
    facts: DamageFacts,
    attachment: Option<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MobilizePlan {
    groups: Vec<TokenGroupView>,
    recipes: Vec<Option<CopyRecipe>>,
    token: Card,
    delayed_sacrifice: bool,
    amount: usize,
    defenders: Vec<CombatDefender>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MobilizeChoiceView {
    pub source: ObjectRef,
    pub token: Card,
    pub tokens: Vec<TokenGroupView>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub copy_recipes: Vec<Option<CopyRecipe>>,
    /// Creature tokens requiring attacking-defender choices; other tokens still enter.
    pub amount: usize,
    pub delayed_sacrifice: bool,
    pub defenders: Vec<CombatDefender>,
}
impl Game {
    pub(super) fn mobilize_choice(&self) -> Option<MobilizeChoiceView> {
        let f = self.state.mobilize.as_ref()?;
        Some(MobilizeChoiceView {
            source: self.state.resolving.as_ref()?.item.object,
            token: f.token,
            tokens: f.groups.clone(),
            copy_recipes: f.recipes.clone(),
            amount: f.amount,
            delayed_sacrifice: f.delayed_sacrifice,
            defenders: f.defenders.clone(),
        })
    }
    fn mobilize_defender_valid(&self, p: Player, d: CombatDefender) -> bool {
        match d {
            CombatDefender::Player(q) => q == p.other(),
            CombatDefender::Planeswalker(r) => {
                self.state.objects.get(r.card as usize).is_some_and(|o| {
                    o.generation == r.generation
                        && o.zone == Zone::Battlefield
                        && o.controller == p.other()
                        && self.current_kinds(r.card) & 64 != 0
                })
            }
        }
    }
    pub(super) fn visit_mobilize_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let p = self.state.resolving.as_ref().unwrap().item.controller;
        let f = self.state.mobilize.as_ref().unwrap();
        if f.defenders
            .iter()
            .any(|&d| !self.mobilize_defender_valid(p, d))
        {
            return false;
        }
        if emit(Action::ChooseMobilizeDefender {
            defender: CombatDefender::Player(p.other()),
        }) {
            return true;
        }
        for (id, o) in self.state.objects.iter().enumerate() {
            if o.zone == Zone::Battlefield
                && o.controller == p.other()
                && self.current_kinds(id as CardId) & 64 != 0
                && emit(Action::ChooseMobilizeDefender {
                    defender: CombatDefender::Planeswalker(self.object_ref(id as CardId)),
                })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn start_mobilize(&mut self, token: Card, count: u128, item: &SpellView) -> bool {
        self.stage_token_creation(
            item.controller,
            token,
            count,
            true,
            replacements::TokenCompletion::Mobilize,
        );
        self.advance_replacements(false)
    }
    pub(super) fn start_mobilize_groups(
        &mut self,
        groups: Vec<TokenGroupView>,
        recipes: Vec<Option<CopyRecipe>>,
        delayed_sacrifice: bool,
    ) -> bool {
        let item = &self.state.resolving.as_ref().unwrap().item;
        let token = groups[0].token;
        let total = groups
            .iter()
            .try_fold(0u128, |n, g| n.checked_add(g.amount))
            .capacity("mobilize count exhausted");
        let total_amount =
            usize::try_from(total).capacity("mobilize count representation exhausted");
        let amount = usize::try_from(
            groups
                .iter()
                .filter(|g| g.token.is_creature())
                .try_fold(0u128, |n, g| n.checked_add(g.amount))
                .capacity("attacking token quantity exhausted"),
        )
        .capacity("mobilize count representation exhausted");
        let fresh = total_amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&n| n <= u128::from(CardId::MAX) + 1)
            .capacity("mobilize object slots exhausted");
        if total_amount == 0 {
            return false;
        }
        let mut defenders = Vec::new();
        defenders
            .try_reserve_exact(amount)
            .capacity("mobilize defender allocation exhausted");
        let choice = amount > 0
            && self.state.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == item.controller.other()
                    && self.current_kinds(id as CardId) & 64 != 0
            });
        if !choice {
            defenders.resize(amount, CombatDefender::Player(item.controller.other()));
        }
        self.state.mobilize = Some(Box::new(MobilizePlan {
            token,
            delayed_sacrifice,
            groups,
            recipes,
            amount,
            defenders,
        }));
        if choice {
            self.state.pending = Pending::Mobilize {
                player: item.controller,
            };
            true
        } else {
            self.finish_mobilize();
            false
        }
    }
    pub(super) fn choose_mobilize_defender(&mut self, defender: CombatDefender) {
        let f = self.state.mobilize.as_mut().unwrap();
        f.defenders.push(defender);
        if f.defenders.len() == f.amount {
            self.finish_mobilize();
            self.resume_token_entries(true);
        }
    }
    fn finish_mobilize(&mut self) {
        let f = self.state.mobilize.take().unwrap();
        assert_eq!(f.amount, f.defenders.len());
        let item = self.state.resolving.as_ref().unwrap().item.clone();
        assert!(
            f.defenders
                .iter()
                .all(|&d| self.mobilize_defender_valid(item.controller, d))
        );
        if f.recipes.is_empty() {
            let tokens =
                self.create_token_groups(item.controller, &f.groups, Some(&f.defenders), true);
            if f.delayed_sacrifice {
                self.record_mobilize_tokens(&item, tokens);
            }
            return;
        }
        let tokens = {
            self.create_copy_groups_bound(
                item.controller,
                &f.groups,
                &f.recipes,
                Some(&f.defenders),
                &[],
            )
        };
        self.defer_token_completion(
            item.controller,
            if f.delayed_sacrifice {
                replacements::TokenCompletion::MobilizeDelayed(Box::new(item))
            } else {
                replacements::TokenCompletion::None
            },
            tokens,
        );
    }
    pub(super) fn record_mobilize_tokens(&mut self, item: &SpellView, tokens: Vec<ObjectRef>) {
        self.record_delayed_tokens(item, tokens, false);
    }
    pub(super) fn record_delayed_tokens(
        &mut self,
        item: &SpellView,
        tokens: Vec<ObjectRef>,
        destroy: bool,
    ) {
        if tokens.is_empty() {
            return;
        }
        let id = self.next_stack_id();
        let batch = DelayedBatch {
            view: DelayedSacrificeView {
                destroy,
                id,
                source: item.object,
                source_card: item.source_card,
                controller: item.controller,
                tokens,
            },
            keywords: item.source_keywords,
            tapped: item.source_tapped,
            facts: item.source_facts,
            attachment: item.source_attachment,
        };
        let batches = self
            .state
            .delayed_sacrifices
            .get_or_insert_with(|| Box::new(Vec::new()));
        batches
            .try_reserve(1)
            .capacity("delayed sacrifice allocation exhausted");
        batches.push(batch);
    }
    pub(super) fn queue_delayed_sacrifices(&mut self) {
        let Some(batches) = self.state.delayed_sacrifices.take() else {
            return;
        };
        self.state
            .pending_triggers
            .try_reserve(batches.len())
            .capacity("delayed trigger allocation exhausted");
        for b in *batches {
            let id = self.next_stack_id();
            let source = b.view.source;
            let card = b.view.source_card;
            let controller = b.view.controller;
            self.state.pending_triggers.push(SpellView {
                sneak: None,
                source_move: None,
                death_event: None,
                creature_action_event: None,
                transform_stamp: None,
                source_chosen_type: None,
                target_controller_lki: None,
                gift: None,
                reflexive_subject: None,
                crew: None,
                cast_event: None,
                copy_event: None,
                source_copy_recipe: None,
                target_allocations: None,
                mana_copy_sources: None,
                is_copy: false,
                delayed_sacrifice: Some(Box::new(b.view)),
                delayed_warp_exile: None,
                return_link: None,
                x: None,
                attachment_trigger: None,
                cast: None,
                entry_batch_event: None,
                entering_object: None,
                evidence_collected: false,
                id,
                kind: StackKind::DelayedSacrifice,
                source_card: card,
                source_keywords: b.keywords,
                source_tapped: b.tapped,
                source_attachment: b.attachment,
                source_facts: b.facts,
                related_stack: None,
                damage_event: None,
                creature_combat_event: None,
                discard_event: None,
                draw_event: None,
                life_event: None,
                source_counter_lki: None,
                sacrificed_toughness: None,
                card: source.card,
                object: source,
                targets: Vec::new(),
                mode: None,
                controller,
            });
        }
    }
    pub(super) fn sacrifice_delayed_tokens(&mut self, item: &SpellView) {
        let d = item
            .delayed_sacrifice
            .as_ref()
            .expect("delayed sacrifice context");
        if d.destroy {
            let ids: Vec<_> = d
                .tokens
                .iter()
                .filter(|r| {
                    self.on_battlefield(r.card)
                        && self.object_ref(r.card) == **r
                        && !self.keywords(r.card).contains(Keywords::INDESTRUCTIBLE)
                })
                .map(|r| r.card)
                .collect();
            let sources = self.death_trigger_sources();
            let events = self.move_objects_raw(&ids, Zone::Graveyard);
            self.finish_zone_events(&events, &sources);
            return;
        }
        let sacrifices: Vec<_> = d
            .tokens
            .iter()
            .filter_map(|r| {
                self.state
                    .objects
                    .get(r.card as usize)
                    .filter(|o| {
                        o.zone == Zone::Battlefield
                            && o.generation == r.generation
                            && o.controller == d.controller
                    })
                    .map(|_| (d.controller, r.card))
            })
            .collect();
        if !sacrifices.is_empty() {
            self.sacrifice_batch(&sacrifices);
        }
    }
    pub(super) fn mobilize_bytes(&self) -> usize {
        self.state.mobilize.as_ref().map_or(0, |f| {
            std::mem::size_of::<MobilizePlan>()
                + f.defenders.len() * std::mem::size_of::<CombatDefender>()
                + f.groups.len() * std::mem::size_of::<TokenGroupView>()
                + f.recipes.len() * std::mem::size_of::<Option<CopyRecipe>>()
        }) + self.state.delayed_sacrifices.as_ref().map_or(0, |b| {
            std::mem::size_of::<Vec<DelayedBatch>>()
                + b.iter()
                    .map(|d| {
                        std::mem::size_of::<DelayedBatch>()
                            + d.view.tokens.len() * std::mem::size_of::<ObjectRef>()
                    })
                    .sum::<usize>()
        })
    }
}
