//! Cold nested mana payment frames. Production resolves immediately; reflexive
//! triggers wait for priority after the enclosing spell/ability has been paid.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManaAnnouncement {
    pub announcement: Announcement,
    pub pending: Pending,
    pub(super) quality: Option<(ObjectRef, mana_lots::Quality)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct ManaAnnouncementView {
    pub item: SpellView,
    pub group: Option<u16>,
    pub cost: Option<ManaCost>,
    pub counters: Option<Counters>,
    pub payment: Option<Mana>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn suspend_mana_announcement(&mut self) {
        let Some(announcement) = self.state.announcing.take() else {
            return;
        };
        assert!(matches!(
            self.state.pending,
            Pending::PayCosts { .. } | Pending::Evidence { .. }
        ));
        let quality = self
            .state
            .mana_lots
            .as_mut()
            .and_then(|m| m.announcement_quality.take());
        let pending = self.state.pending.clone();
        let frames = self.state.mana_announcement_stack.get_or_insert_default();
        frames
            .try_reserve(1)
            .capacity("nested mana payment allocation exhausted");
        frames.push(ManaAnnouncement {
            announcement,
            pending,
            quality,
        });
    }
    pub(super) fn restore_mana_announcement(&mut self) -> bool {
        let Some(frames) = self.state.mana_announcement_stack.as_mut() else {
            return false;
        };
        let frame = frames.pop().expect("nonempty mana payment stack");
        if frames.is_empty() {
            self.state.mana_announcement_stack = None;
        }
        if let Some(quality) = frame.quality {
            self.state
                .mana_lots
                .get_or_insert_default()
                .announcement_quality = Some(quality);
        }
        self.state.announcing = Some(frame.announcement);
        self.state.pending = frame.pending;
        true
    }
    pub(super) fn committed_mana_reflexive(&mut self, item: &SpellView) {
        self.committed_activation_observers(item);
        if let Some((_, [Instruction::AddMana(_), Instruction::Reflexive { ability }])) =
            item.program()
        {
            self.queue_reflexive(item, *ability, None);
        }
    }
    pub(super) fn mana_announcement_views(&self, viewer: Player) -> Vec<ManaAnnouncementView> {
        self.state
            .mana_announcement_stack
            .as_ref()
            .into_iter()
            .flat_map(|v| v.iter())
            .map(|frame| {
                let a = &frame.announcement;
                let owns = self.shares_information(viewer, a.item.controller);
                let mut item = a.item.clone();
                if !owns {
                    item.redact_uncommitted_targets();
                }
                let goal = owns.then(|| self.funding_goal(a));
                ManaAnnouncementView {
                    item,
                    group: owns.then_some(a.group),
                    cost: goal.map(|g| ManaCost {
                        colored: g.required,
                        generic: g.generic,
                    }),
                    counters: owns.then(|| a.counter_payment.clone()),
                    payment: owns.then_some(a.payment).flatten(),
                }
            })
            .collect()
    }
    pub(super) fn mana_announcement_bytes(&self) -> usize {
        self.state
            .mana_announcement_stack
            .as_ref()
            .map_or(0, |frames| {
                std::mem::size_of::<Vec<ManaAnnouncement>>()
                    + frames
                        .iter()
                        .map(|f| {
                            let a = &f.announcement;
                            std::mem::size_of::<ManaAnnouncement>()
                                + f.pending.owned_bytes()
                                + a.item.owned_bytes()
                                + a.counter_payment.owned_bytes()
                                + a.free_hand_permission
                                    .as_ref()
                                    .map_or(0, |_| std::mem::size_of::<ObjectRef>())
                                + casualty::bytes(&a.casualty)
                                + a.cast_costs.as_ref().map_or(0, |c| c.bytes())
                                + a.convoke.as_ref().map_or(0, |v| v.bytes())
                                + a.power_up_costs.as_ref().map_or(0, |v| v.bytes())
                                + a.evidence.as_ref().map_or(0, |v| v.bytes())
                                + a.forage.as_ref().map_or(0, |v| v.bytes())
                                + a.additional_group.as_ref().map_or(0, |v| {
                                    std::mem::size_of::<Vec<ObjectRef>>()
                                        + v.len() * std::mem::size_of::<ObjectRef>()
                                })
                                + a.creature_counter_cost.as_ref().map_or(0, |v| {
                                    std::mem::size_of::<Vec<(ObjectRef, CounterKind)>>()
                                        + v.len() * std::mem::size_of::<(ObjectRef, CounterKind)>()
                                })
                        })
                        .sum::<usize>()
            })
    }
}

/// An activated instance, independent of its mutable characteristics or costs.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ManaActivationKey {
    pub source: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    pub borrowed_slot: Option<u64>,
    pub self_grant: Option<u64>,
    pub source_origin: Option<u64>,
    pub grant: Option<(ObjectRef, Card, Option<u64>)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn mana_activation_key(&self) -> Option<ManaActivationKey> {
        let StackKind::ActivatedAbility { ability } = self.kind else {
            return None;
        };
        let source = self.source?;
        Some(ManaActivationKey {
            source: source.object,
            definition: source.card,
            ability,
            borrowed_slot: self.borrowed_slot,
            self_grant: self
                .self_granted_activation
                .as_ref()
                .map(|row| row.grant_timestamp),
            source_origin: self.source_ability_origin,
            grant: self
                .attachment_trigger
                .as_ref()
                .map(|row| (row.grantor, row.granting_card, row.origin)),
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn active_mana_activation_keys(&self) -> Vec<ManaActivationKey> {
        self.state
            .announcing
            .iter()
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|rows| rows.iter().map(|row| &row.announcement)),
            )
            .filter(|row| row.production.is_some())
            .filter_map(|row| row.item.mana_activation_key())
            .collect()
    }
    pub(super) fn deferred_mana_parents(&self) -> Vec<ManaAnnouncement> {
        let mut parents = self
            .state
            .mana_announcement_stack
            .as_deref()
            .cloned()
            .unwrap_or_default();
        if let Some(parent) = &self.state.announcing {
            parents.push(ManaAnnouncement {
                announcement: parent.clone(),
                pending: self.state.pending.clone(),
                quality: self
                    .state
                    .mana_lots
                    .as_ref()
                    .and_then(|lots| lots.announcement_quality),
            });
        }
        parents
    }
}
