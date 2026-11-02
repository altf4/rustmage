//! Attacking entry batches and source-independent, owned next-end-step abilities.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedSacrificeView {
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub tokens: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl DelayedSacrificeView {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.tokens.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedBatch {
    pub view: DelayedSacrificeView,
    keywords: Keywords,
    tapped: bool,
    pub(super) facts: DamageFacts,
    attachment: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MobilizePlan {
    groups: Vec<TokenGroupView>,
    token: Card,
    amount: usize,
    defenders: Vec<CombatDefender>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MobilizeChoiceView {
    pub source: ObjectRef,
    pub token: Card,
    pub tokens: Vec<TokenGroupView>,
    pub amount: usize,
    pub defenders: Vec<CombatDefender>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn mobilize_choice(&self) -> Option<MobilizeChoiceView> {
        let f = self.state.mobilize.as_ref()?;
        Some(MobilizeChoiceView {
            source: self.state.resolving.as_ref()?.item.source?.object,
            token: f.token,
            tokens: f.groups.clone(),
            amount: f.amount,
            defenders: f.defenders.clone(),
        })
    }
    fn mobilize_defender_valid(&self, p: Player, d: CombatDefender) -> bool {
        match d {
            CombatDefender::Player(q) => q == p.other(),
            CombatDefender::Battle(r) => self.battle_can_be_attacked_by(r, p),
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
            let battle = self.object_ref(id as CardId);
            if self.battle_can_be_attacked_by(battle, p)
                && emit(Action::ChooseMobilizeDefender {
                    defender: CombatDefender::Battle(battle),
                })
            {
                return true;
            }
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
    pub(super) fn start_mobilize_groups(&mut self, groups: Vec<TokenGroupView>) -> bool {
        let item = &self.state.resolving.as_ref().unwrap().item;
        let token = groups[0].token;
        let total = groups
            .iter()
            .try_fold(0u128, |n, g| n.checked_add(g.amount))
            .capacity("mobilize count exhausted");
        let amount = usize::try_from(total).capacity("mobilize count representation exhausted");
        let fresh = amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&n| n <= u128::from(CardId::MAX) + 1)
            .capacity("mobilize object slots exhausted");
        if amount == 0 {
            return false;
        }
        let mut defenders = Vec::new();
        defenders
            .try_reserve_exact(amount)
            .capacity("mobilize defender allocation exhausted");
        let choice = self.state.objects.iter().enumerate().any(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == item.controller.other()
                && self.current_kinds(id as CardId) & 64 != 0
        });
        if !choice {
            defenders.resize(amount, CombatDefender::Player(item.controller.other()));
        }
        self.state.mobilize = Some(Box::new(MobilizePlan {
            token,
            groups,
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
            self.run_program();
        }
    }
    fn finish_mobilize(&mut self) {
        let f = self.state.mobilize.take().unwrap();
        assert_eq!(f.amount, f.defenders.len());
        let item = self.state.resolving.as_ref().unwrap().item.clone();
        let Some(source) = item.source else {
            return;
        };
        assert!(
            f.defenders
                .iter()
                .all(|&d| self.mobilize_defender_valid(item.controller, d))
        );
        let tokens = self.create_token_groups(item.controller, &f.groups, Some(&f.defenders), true);
        if tokens.is_empty() {
            return;
        }
        let id = self.next_stack_id();
        let batch = DelayedBatch {
            view: DelayedSacrificeView {
                id,
                source: source.object,
                source_card: source.card,
                controller: item.controller,
                tokens,
            },
            keywords: item.source_keywords,
            tapped: item.source_tapped,
            facts: source.facts,
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
    pub(super) fn finish_temporary_copy(&mut self, item: &SpellView, tokens: Vec<ObjectRef>) {
        let Some(source) = item.source else {
            return;
        };
        let ids = tokens.iter().map(|r| r.card).collect::<Vec<_>>();
        // Molten Duplication grants haste independently; Fire Crystal's copies
        // obtain haste only from its ordinary continuous battlefield ability.
        if item.program().is_none_or(|(_, program)| {
            !program.contains(&Instruction::CreateCopyWithDelayedSacrifice)
        }) {
            self.grant_keyword_effects(&ids, source.object, Keywords::HASTE);
        }
        self.register_delayed_sacrifice(item, tokens);
    }
    pub(super) fn register_delayed_sacrifice(&mut self, item: &SpellView, tokens: Vec<ObjectRef>) {
        if tokens.is_empty() {
            return;
        }
        let Some(source) = item.source else {
            return;
        };
        let id = self.next_stack_id();
        let batch = DelayedBatch {
            view: DelayedSacrificeView {
                id,
                source: source.object,
                source_card: source.card,
                controller: item.controller,
                tokens,
            },
            keywords: item.source_keywords,
            tapped: item.source_tapped,
            facts: source.facts,
            attachment: item.source_attachment,
        };
        self.state
            .delayed_sacrifices
            .get_or_insert_with(|| Box::new(Vec::new()))
            .push(batch);
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
                source: Some(StackSourceView {
                    id: source.card,
                    object: source,
                    card,
                    facts: b.facts,
                }),
                source_less_rule: None,
                additional_triggers: 0,
                source_move: None,
                borrowed_slot: None,
                self_granted_activation: None,
                target_controller_lki: None,
                crime_qualifying_target: false,
                target_kinds_lki: None,
                target_type_slots: None,
                gift: None,
                reflexive_subject: None,
                crew: None,
                cast_event: None,
                cast_creature_subjects: None,
                copy_event: None,
                captured_artifact_copy: None,
                source_copy_recipe: None,
                source_ability_origin: None,
                target_allocations: None,
                mana_copy_sources: None,
                is_copy: false,
                ninjutsu_defender: None,
                delayed_sacrifice: Some(Box::new(b.view)),
                delayed_warp_exile: None,
                return_link: None,
                x: None,
                loyalty_x: None,
                reflexive_target_limit: None,
                attachment_trigger: None,
                cast: None,
                entering_object: None,
                evidence_collected: false,
                id,
                kind: StackKind::DelayedSacrifice,

                split_second: false,
                source_keywords: b.keywords,
                source_tapped: b.tapped,
                mana_tap_bonus: None,
                source_attachment: b.attachment,
                source_transform: None,

                related_stack: None,
                damage_event: None,
                damage_received: None,
                creature_combat_event: None,
                discard_event: None,
                draw_event: None,
                life_event: None,
                source_counter_payment: None,
                loyalty_counters_paid: 0,
                exiled_symbol_cards: None,
                craft_materials: None,
                paid_additional_cost: None,
                exiled_trigger_cards: None,
                source_counter_lki: None,
                source_quantity_lki: None,
                event_player: None,
                event_object: None,
                manifest_event: None,
                sacrificed_toughness: None,
                sacrificed_mana_value: None,
                sacrificed_power: None,

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
