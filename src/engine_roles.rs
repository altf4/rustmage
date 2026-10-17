//! Snapshot-owned Role timestamps and predefined attachment creation (CR303.7/704.5).
use super::*;
use crate::cards::RoleKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoleTimestampView {
    pub aura: ObjectRef,
    pub recipient: ObjectRef,
    pub timestamp: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoleGroupView {
    pub controller: Player,
    pub recipient: ObjectRef,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RoleState {
    clock: u64,
    pub attached: Vec<RoleTimestampView>,
    pub base_stats: Vec<BaseStatsEffectView>,
}
impl RoleState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.attached.len() * std::mem::size_of::<RoleTimestampView>()
            + self.base_stats.len() * std::mem::size_of::<BaseStatsEffectView>()
    }
}
impl Game {
    pub(super) fn queue_reflexive(
        &mut self,
        item: &SpellView,
        ability: u16,
        subject: Option<ObjectRef>,
    ) {
        let mut reflexive = item.clone();
        reflexive.id = self.next_stack_id();
        reflexive.kind = StackKind::TriggeredAbility { ability };
        reflexive.targets.clear();
        reflexive.mode = None;
        reflexive.reflexive_subject =
            subject.map(|object| Box::new(ReflexiveSubjectView { object }));
        self.state.pending_triggers.push(reflexive);
    }
    pub(super) fn resolve_role_reflexive(&mut self, item: &SpellView, token: Card, ability: u16) {
        assert!(
            token.role().is_some(),
            "Role-success instruction requires a Role factory"
        );
        for bound in &item.targets {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
            {
                let recipient = self.object_ref(id);
                if self.aura_can_enchant(token, item.controller, recipient) {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        1,
                        false,
                        replacements::TokenCompletion::RoleReflexive(Box::new(
                            replacements::RoleReflexiveCompletion {
                                recipient,
                                token,
                                item: item.clone(),
                                ability,
                            },
                        )),
                    );
                }
            }
        }
    }
    pub(super) fn resolve_roles_matching(
        &mut self,
        item: &SpellView,
        token: Card,
        filter: CreatureFilter,
    ) {
        assert!(token.role().is_some(), "predefined Role factory required");
        let recipients = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, _)| {
                let id = id as CardId;
                let recipient = self.object_ref(id);
                (self.creature_matches(filter, item.controller, item.object, id)
                    && self.aura_can_enchant(token, item.controller, recipient))
                .then_some(recipient)
            })
            .collect();
        self.stage_role_creation(item.controller, token, recipients);
    }

    pub(super) fn next_role_timestamp(&mut self) -> u64 {
        let r = self.state.roles.get_or_insert_with(Default::default);
        r.clock = r.clock.checked_add(1).capacity("Role timestamps exhausted");
        r.clock
    }
    pub(super) fn record_role(&mut self, aura: ObjectRef, recipient: ObjectRef, timestamp: u64) {
        let r = self.state.roles.get_or_insert_with(Default::default);
        r.attached.retain(|v| v.aura != aura);
        r.attached.push(RoleTimestampView {
            aura,
            recipient,
            timestamp,
        });
    }
    pub(super) fn forget_role(&mut self, aura: ObjectRef) {
        if let Some(r) = &mut self.state.roles {
            r.attached.retain(|v| v.aura != aura);
        }
    }
    pub(super) fn cursed_role_timestamp(&self, recipient: ObjectRef) -> Option<u64> {
        self.state.roles.as_ref().and_then(|r| {
            r.attached
                .iter()
                .filter(|v| {
                    v.recipient == recipient
                        && self.on_battlefield(v.aura.card)
                        && self.object_ref(v.aura.card) == v.aura
                        && self.state.objects[v.aura.card as usize].card.role()
                            == Some(RoleKind::Cursed)
                        && self.state.objects[v.aura.card as usize].attached_to == Some(recipient)
                })
                .map(|v| v.timestamp)
                .max()
        })
    }
    pub(super) fn resolve_role(&mut self, item: &SpellView, token: Card, subject: Subject) {
        assert!(
            token.role().is_some(),
            "Role instruction requires a predefined Role factory"
        );
        let recipients: Vec<_> = match subject {
            Subject::Source => (self.on_battlefield(item.card)
                && self.object_ref(item.card) == item.object)
                .then_some(item.object)
                .into_iter()
                .collect(),
            Subject::Attached => self
                .attached_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Entering => self
                .entering_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Targets => item
                .targets
                .iter()
                .filter(|&&b| self.same_target_object(b))
                .filter_map(|b| {
                    if let Target::Card(id) = b.target {
                        Some(self.object_ref(id))
                    } else {
                        None
                    }
                })
                .collect(),
        };
        for recipient in recipients {
            if self.aura_can_enchant(token, item.controller, recipient) {
                self.stage_token_creation(
                    item.controller,
                    token,
                    1,
                    false,
                    replacements::TokenCompletion::RoleAttach(recipient),
                );
            }
        }
    }
    pub(super) fn create_attached_role_groups(
        &mut self,
        controller: Player,
        groups: &[TokenGroupView],
        recipient: ObjectRef,
    ) -> Vec<ObjectRef> {
        self.create_token_groups_attached(controller, groups, None, false, Some(recipient))
    }
    /// All roles participate even if another SBA would simultaneously remove one.
    /// Only malformed/illegal attachments are excluded by the enchantment rule.
    pub(super) fn collect_role_choices(&self, deaths: &mut Vec<CardId>) -> Vec<RoleGroupView> {
        let Some(state) = &self.state.roles else {
            return Vec::new();
        };
        let mut groups: Vec<(Player, ObjectRef, Vec<&RoleTimestampView>)> = Vec::new();
        for v in &state.attached {
            if !self.on_battlefield(v.aura.card) || self.object_ref(v.aura.card) != v.aura {
                continue;
            }
            let aura = &self.state.objects[v.aura.card as usize];
            if aura.attached_to != Some(v.recipient)
                || !self.aura_can_enchant(aura.card, aura.controller, v.recipient)
            {
                continue;
            }
            if let Some(g) = groups
                .iter_mut()
                .find(|g| g.0 == aura.controller && g.1 == v.recipient)
            {
                g.2.push(v);
            } else {
                groups.push((aura.controller, v.recipient, vec![v]));
            }
        }
        let mut choices = Vec::new();
        for (controller, recipient, roles) in groups {
            if roles.len() < 2 {
                continue;
            }
            let newest = roles.iter().map(|v| v.timestamp).max().unwrap();
            deaths.extend(
                roles
                    .iter()
                    .filter(|v| v.timestamp < newest)
                    .map(|v| v.aura.card),
            );
            let objects: Vec<_> = roles
                .iter()
                .filter(|v| v.timestamp == newest)
                .map(|v| v.aura)
                .collect();
            if objects.len() > 1 {
                choices.push(RoleGroupView {
                    controller,
                    recipient,
                    objects,
                    keep: None,
                });
            }
        }
        choices
    }
    pub(super) fn keep_role(&mut self, object: ObjectRef) {
        let batch = self.state.sba_batch.as_mut().unwrap();
        batch
            .roles
            .iter_mut()
            .find(|g| g.keep.is_none() && g.objects.contains(&object))
            .unwrap()
            .keep = Some(object);
        self.advance_sba_choices();
    }
}
