//! Private subset revelation, captured-controller choices and exact temporary exile.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum HandChoiceKind {
    DeployCreatures,
    RevealSubset,
    DiscardRevealed,
    ExileRevealedOrCreature,
    ExileRevealedLinked,
    /// Choose a controlled permanent of the requested subtype or reveal a hand card.
    Behold,
}
#[derive(Clone, Debug, Serialize)]
pub struct HandChoiceView {
    pub kind: HandChoiceKind,
    pub actor: Player,
    pub victim: Option<Player>,
    pub required: usize,
    pub selected: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
    pub optional: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HandInteraction {
    kind: HandChoiceKind,
    actor: Player,
    controller: Player,
    victim: Option<Player>,
    source: ObjectRef,
    required: usize,
    selected: Vec<ObjectRef>,
    candidates: Vec<ObjectRef>,
}
impl HandInteraction {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.selected.len() + self.candidates.len()) * std::mem::size_of::<ObjectRef>()
    }
}
/// Collect exact non-targeting behold witnesses in an actual or projected world.
/// Objects in the simultaneous entry batch are not yet on the battlefield.
pub(super) fn behold_candidates<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    subtype: &str,
    entering: &[ObjectRef],
) -> Vec<ObjectRef> {
    (0..world.game().state.objects.len())
        .filter_map(|index| {
            let id = index as CardId;
            let object = world.object(id);
            let reference = world.reference(id);
            if entering.contains(&reference) {
                return None;
            }
            let matches = match object.zone {
                Zone::Battlefield => {
                    object.controller == actor
                        && world.subtypes(id).has(world.definition(id), subtype)
                }
                Zone::Hand => {
                    object.owner == actor && object.is_card() && object.card.has_subtype(subtype)
                }
                _ => false,
            };
            matches.then_some(reference)
        })
        .collect()
}
impl Game {
    pub(super) fn start_behold_search_result(
        &mut self,
        item: &SpellView,
        subtype: &'static str,
    ) -> bool {
        let Some(source) = item.source else { return false; };
        let candidates = behold_candidates(self, item.controller, subtype, &[]);
        if candidates.is_empty() {
            self.state.resolving.as_mut().unwrap().search_result = None;
            return false;
        }
        self.set_hand_interaction(HandInteraction {
            kind: HandChoiceKind::Behold,
            actor: item.controller,
            controller: item.controller,
            victim: Some(item.controller),
            source: source.object,
            required: 0,
            selected: Vec::new(),
            candidates,
        });
        true
    }
    fn exact_hand(&self, victim: Player) -> Vec<ObjectRef> {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Hand && o.owner == victim && o.is_card())
            .map(|(i, _)| self.object_ref(i as CardId))
            .collect()
    }
    fn reveal_hand_refs(&mut self, objects: &[ObjectRef]) {
        for &reference in objects {
            self.remember_public_hand(reference);
        }
    }
    fn set_hand_interaction(&mut self, interaction: HandInteraction) {
        let actor = interaction.actor;
        assert!(
            self.state
                .linked_choices
                .as_ref()
                .is_none_or(|s| s.hand.is_none())
        );
        self.state.linked_choices.get_or_insert_default().hand = Some(Box::new(interaction));
        self.state.pending = Pending::HandCardChoice { player: actor };
    }
    pub(super) fn start_hand_subset_discard(
        &mut self,
        item: &SpellView,
        amount: AmountValue,
    ) -> bool {
        let Some(source) = item.source else { return false; };
        let Some(victim) = self.program_players(PlayerSet::Targeted, item).next() else {
            return false;
        };
        let hand = self.exact_hand(victim);
        let required = self.item_amount(amount, item).min(hand.len() as u128) as usize;
        if required == 0 {
            return false;
        }
        let mut interaction = HandInteraction {
            kind: HandChoiceKind::RevealSubset,
            actor: victim,
            controller: item.controller,
            victim: Some(victim),
            source: source.object,
            required,
            selected: Vec::new(),
            candidates: hand,
        };
        if interaction.candidates.len() == required {
            self.reveal_hand_refs(&interaction.candidates);
            interaction.kind = HandChoiceKind::DiscardRevealed;
            interaction.actor = item.controller;
            interaction.required = 1;
        }
        self.set_hand_interaction(interaction);
        true
    }
    pub(super) fn start_revealed_hand_exile(&mut self, item: &SpellView) -> bool {
        let Some(source) = item.source else { return false; };
        let victim = item.targets.iter().find_map(|b| {
            if b.group == 0 {
                if let Target::Player(p) = b.target {
                    Some(p)
                } else {
                    None
                }
            } else {
                None
            }
        });
        let hand = victim.map_or_else(Vec::new, |p| self.exact_hand(p));
        self.reveal_hand_refs(&hand);
        if !self.on_battlefield(source.object.card)
            || self.object_ref(source.object.card) != source.object
        {
            return false;
        }
        let mut candidates: Vec<_> = hand
            .into_iter()
            .filter(|r| !self.state.objects[r.card as usize].card.is_land())
            .collect();
        candidates.extend(
            item.targets
                .iter()
                .filter(|b| b.group == 1 && self.same_target_object(**b))
                .filter_map(|b| b.object),
        );
        if candidates.is_empty() {
            return false;
        }
        self.set_hand_interaction(HandInteraction {
            kind: HandChoiceKind::ExileRevealedOrCreature,
            actor: item.controller,
            controller: item.controller,
            victim,
            source: source.object,
            required: 0,
            selected: Vec::new(),
            candidates,
        });
        true
    }
    pub(super) fn start_revealed_hand_exile_linked(&mut self, item: &SpellView) -> bool {
        let Some(victim) = self.program_players(PlayerSet::Targeted, item).next() else {
            return false;
        };
        let hand = self.exact_hand(victim);
        self.reveal_hand_refs(&hand);
        let candidates: Vec<_> = hand
            .into_iter()
            .filter(|r| !self.state.objects[r.card as usize].card.is_land())
            .collect();
        if candidates.is_empty() {
            return false;
        }
        self.set_hand_interaction(HandInteraction {
            kind: HandChoiceKind::ExileRevealedLinked,
            actor: item.controller,
            controller: item.controller,
            victim: Some(victim),
            source: item.source.expect("object-backed packet").object,
            required: 1,
            selected: Vec::new(),
            candidates,
        });
        true
    }
    fn hand_choice_candidate(&self, interaction: &HandInteraction, reference: ObjectRef) -> bool {
        if !interaction.candidates.contains(&reference)
            || interaction.selected.contains(&reference)
            || self.object_ref(reference.card) != reference
        {
            return false;
        }
        let o = &self.state.objects[reference.card as usize];
        match interaction.kind {
            HandChoiceKind::DeployCreatures => unreachable!(),
            HandChoiceKind::RevealSubset | HandChoiceKind::DiscardRevealed => {
                o.zone == Zone::Hand && Some(o.owner) == interaction.victim && o.is_card()
            }
            HandChoiceKind::ExileRevealedLinked => {
                o.zone == Zone::Hand
                    && Some(o.owner) == interaction.victim
                    && !o.is_token()
                    && !o.card.is_land()
            }
            HandChoiceKind::ExileRevealedOrCreature => {
                self.on_battlefield(interaction.source.card)
                    && self.object_ref(interaction.source.card) == interaction.source
                    && (o.zone == Zone::Battlefield
                        || (o.zone == Zone::Hand
                            && o.is_card()
                            && Some(o.owner) == interaction.victim
                            && !o.card.is_land()))
            }
            HandChoiceKind::Behold => {
                (o.zone == Zone::Battlefield && o.controller == interaction.controller)
                    || (o.zone == Zone::Hand && o.owner == interaction.controller && o.is_card())
            }
        }
    }
    pub(super) fn visit_hand_card_choices(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if self.hand_deployment().is_some() {
            return self.visit_hand_deployment(emit);
        }
        let interaction = self
            .state
            .linked_choices
            .as_ref()
            .unwrap()
            .hand
            .as_ref()
            .unwrap();
        if matches!(
            interaction.kind,
            HandChoiceKind::ExileRevealedOrCreature | HandChoiceKind::Behold
        ) && emit(Action::ChooseHandCard { object: None })
        {
            return true;
        }
        interaction
            .candidates
            .iter()
            .copied()
            .filter(|&r| self.hand_choice_candidate(interaction, r))
            .any(|object| {
                emit(Action::ChooseHandCard {
                    object: Some(object),
                })
            })
    }
    pub(super) fn choose_hand_card(&mut self, object: Option<ObjectRef>) {
        if self.hand_deployment().is_some() {
            self.choose_hand_deployment(object);
            return;
        }
        let mut interaction = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .hand
            .take()
            .unwrap();
        if interaction.kind == HandChoiceKind::RevealSubset {
            interaction.selected.push(object.unwrap());
            if interaction.selected.len() == interaction.required {
                self.reveal_hand_refs(&interaction.selected);
                interaction.candidates = std::mem::take(&mut interaction.selected);
                interaction.kind = HandChoiceKind::DiscardRevealed;
                interaction.actor = interaction.controller;
                interaction.required = 1;
            }
            self.set_hand_interaction(*interaction);
            return;
        }
        if let Some(object) = object {
            match interaction.kind {
                HandChoiceKind::DiscardRevealed => {
                    self.discard_batch_caused(&[object.card], None, Some(interaction.controller))
                }
                HandChoiceKind::ExileRevealedOrCreature => {
                    self.exile_objects_until_source_leaves(interaction.source, &[object])
                }
                HandChoiceKind::ExileRevealedLinked => {
                    let event = self.move_object(object.card, Zone::Exile);
                    if event.moved && event.destination == Zone::Exile {
                        let ability_origin = self
                            .state
                            .resolving
                            .as_ref()
                            .unwrap()
                            .item
                            .source_ability_origin;
                        self.state
                            .linked_choices
                            .get_or_insert_default()
                            .null_cast_links
                            .push(linked_choices::NullCastLink {
                                source: interaction.source,
                                exiled: event.after,
                                ability_origin,
                            });
                    }
                }
                HandChoiceKind::Behold => {
                    if self.state.objects[object.card as usize].zone == Zone::Hand {
                        self.reveal_hand_refs(&[object]);
                    }
                    if let Some(land) = self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .search_result
                        .as_ref()
                        .and_then(|r| r.entered)
                        && self.object_ref(land.card) == land
                        && self.on_battlefield(land.card)
                    {
                        self.untap_objects(&[land.card], untap_rules::UntapCause::Effect);
                    }
                }
                HandChoiceKind::RevealSubset | HandChoiceKind::DeployCreatures => unreachable!(),
            }
        }
        if interaction.kind == HandChoiceKind::Behold {
            self.state.resolving.as_mut().unwrap().search_result = None;
        }
        self.run_program();
    }
    pub(super) fn hand_choice_view(&self, viewer: Player) -> Option<HandChoiceView> {
        if !matches!(self.state.pending,Pending::HandCardChoice{player} if self.shares_information(viewer,player))
        {
            return None;
        }
        if self.hand_deployment().is_some() {
            return self.hand_deployment_view(viewer);
        }
        let interaction = self.state.linked_choices.as_ref()?.hand.as_ref()?;
        Some(HandChoiceView {
            kind: interaction.kind,
            actor: interaction.actor,
            victim: interaction.victim,
            required: interaction.required,
            selected: interaction.selected.clone(),
            candidates: interaction
                .candidates
                .iter()
                .copied()
                .filter(|&r| self.hand_choice_candidate(interaction, r))
                .collect(),
            optional: matches!(
                interaction.kind,
                HandChoiceKind::ExileRevealedOrCreature | HandChoiceKind::Behold
            ),
        })
    }
}
