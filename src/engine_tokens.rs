//! Fixed token creation batches and snapshot-owned reusable object slots.
use super::*;

impl Game {
    pub(super) fn create_tokens(
        &mut self,
        controller: Player,
        token: Card,
        amount: u128,
        tapped: bool,
    ) {
        self.stage_token_creation(
            controller,
            token,
            amount,
            tapped,
            replacements::TokenCompletion::None,
        );
    }
    #[allow(clippy::too_many_arguments)] // Optional capture avoids allocating refs for ordinary token batches.
    pub(super) fn create_token_batch_capture(
        &mut self,
        controller: Player,
        token: Card,
        amount: u128,
        tapped: bool,
        defenders: Option<&[CombatDefender]>,
        capture: bool,
    ) -> Vec<ObjectRef> {
        self.create_token_groups(
            controller,
            &[TokenGroupView {
                token,
                amount,
                tapped,
            }],
            defenders,
            capture,
        )
    }
    pub(super) fn create_token_groups(
        &mut self,
        controller: Player,
        groups: &[TokenGroupView],
        defenders: Option<&[CombatDefender]>,
        capture: bool,
    ) -> Vec<ObjectRef> {
        self.create_token_groups_attached(controller, groups, defenders, capture, None)
    }
    pub(super) fn create_token_groups_attached(
        &mut self,
        controller: Player,
        groups: &[TokenGroupView],
        defenders: Option<&[CombatDefender]>,
        capture: bool,
        attachment: Option<ObjectRef>,
    ) -> Vec<ObjectRef> {
        self.create_token_groups_bound(controller, groups, defenders, capture, attachment, None)
    }
    pub(super) fn create_token_groups_per_recipient(
        &mut self,
        controller: Player,
        groups: &[TokenGroupView],
        recipients: &[ObjectRef],
    ) -> Vec<ObjectRef> {
        assert!(recipients.len() <= groups.len());
        self.create_token_groups_bound(controller, groups, None, false, None, Some(recipients))
    }
    #[allow(clippy::too_many_arguments)] // Bindings preserve the allocation-free ordinary token path.
    fn create_token_groups_bound(
        &mut self,
        controller: Player,
        groups: &[TokenGroupView],
        defenders: Option<&[CombatDefender]>,
        capture: bool,
        attachment: Option<ObjectRef>,
        recipients: Option<&[ObjectRef]>,
    ) -> Vec<ObjectRef> {
        let recipient = |index| {
            recipients
                .and_then(|r| r.get(index))
                .copied()
                .or(attachment)
        };
        // Keep original group indices when a recipient becomes stale. Replacement
        // additions have no recipient and must never slide into a removed slot.
        let valid: Option<Vec<bool>> = (attachment.is_some() || recipients.is_some()).then(|| {
            groups
                .iter()
                .enumerate()
                .map(|(index, group)| {
                    group.token.role().is_none()
                        || recipient(index)
                            .is_some_and(|r| self.aura_can_enchant(group.token, controller, r))
                })
                .collect()
        });
        let accepted = |index| valid.as_ref().is_none_or(|v| v[index]);
        let timestamp = if (attachment.is_some() || recipients.is_some())
            && groups
                .iter()
                .enumerate()
                .any(|(index, g)| accepted(index) && g.token.role().is_some() && g.amount > 0)
        {
            Some(self.next_role_timestamp())
        } else {
            None
        };
        let total = groups
            .iter()
            .enumerate()
            .filter(|(index, _)| accepted(*index))
            .try_fold(0u128, |n, (_, g)| {
                assert!(g.token.is_token(), "token instruction requires a factory");
                n.checked_add(g.amount)
            })
            .capacity("token quantity exhausted");
        let amount = usize::try_from(total).capacity("token count representation exhausted");
        let fresh = amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&end| end <= u128::from(CardId::MAX) + 1)
            .capacity("object slots exhausted");
        if amount == 0 {
            return Vec::new();
        }
        if let Some(d) = defenders {
            let attackers = groups
                .iter()
                .enumerate()
                .filter(|(index, g)| accepted(*index) && g.token.is_creature())
                .map(|(_, g)| usize::try_from(g.amount).capacity("attacker quantity exhausted"))
                .sum::<usize>();
            assert_eq!(d.len(), attackers);
            self.state
                .attackers
                .try_reserve(amount)
                .capacity("attacking token allocation exhausted");
        }
        // Replacement effects inspect the pre-event battlefield once, before
        // any token in this simultaneous batch is installed as a new source.

        let mut events = Vec::new();
        events
            .try_reserve_exact(amount)
            .capacity("token batch allocation exhausted");
        self.state
            .objects
            .try_reserve(fresh)
            .capacity("object allocation exhausted");
        let mut index = 0;
        for (group_index, group) in groups.iter().enumerate() {
            if !accepted(group_index) {
                continue;
            }
            let attachment = recipient(group_index);
            let token = group.token;
            let tapped = group.tapped || self.opponent_entry_requires_tapped(token, controller);
            for _ in 0..group.amount {
                let (id, generation) = match self.state.free_tokens.pop() {
                    Some(id) => {
                        let old = &self.state.objects[id as usize];
                        assert_eq!(old.zone, Zone::Outside);
                        assert!(old.is_token() || old.is_spell_copy());
                        (id, old.generation)
                    }
                    None => (
                        CardId::try_from(self.state.objects.len())
                            .capacity("object slots exhausted"),
                        0,
                    ),
                };
                let object = Object {
                    card: token,
                    copy_kind: CopyKind::None,
                    owner: controller,
                    controller,
                    zone: Zone::Outside,
                    generation,
                    tapped: false,
                    summoning_sick: false,
                    flashback: false,
                    damage: 0,
                    damaged_this_turn: false,
                    dealt_damage_this_turn: false,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: InitialCounters::default(),
                    used_abilities: 0,
                    attached_to: None,
                };
                if id as usize == self.state.objects.len() {
                    self.state.objects.push(object);
                } else {
                    self.state.objects[id as usize] = object;
                }
                let before = self.object_view(id);
                let object = &mut self.state.objects[id as usize];
                object.zone = Zone::Battlefield;
                object.generation = generation
                    .checked_add(1)
                    .capacity("object generation exhausted");
                object.tapped = tapped;
                object.summoning_sick = true;
                if token.is_creature()
                    && let Some(d) = defenders
                {
                    self.state.attackers.push(Attacker {
                        card: id,
                        defender: Some(d[index]),
                        blocked: false,
                    });
                    index += 1;
                }
                if token.role().is_some() {
                    let recipient = attachment.expect("Role creation requires an attachment");
                    self.state.objects[id as usize].attached_to = Some(recipient);
                    self.record_role(self.object_ref(id), recipient, timestamp.unwrap());
                }
                self.record_nonland_entry(controller, token);
                self.register_battlefield_sources(id);
                events.push(ZoneChange {
                    gift: None,
                    before,
                    after: self.object_ref(id),
                    destination: Zone::Battlefield,
                    moved: true,
                    sacrificed_by: None,
                    evoked: false,
                    cast_payment: None,
                    offspring_paid: false,
                    kicked: false,
                    bargained: false,
                    x: 0,
                    evidence_collected: false,
                });
            }
        }
        // Every newcomer exists before any entry event inspects the battlefield.
        self.finish_zone_events(&events, &[]);
        if capture {
            events.iter().map(|e| e.after).collect()
        } else {
            Vec::new()
        }
    }

    pub(super) fn cease_token(&mut self, id: CardId) {
        self.clear_copy_recipe(self.object_ref(id));
        let object = &mut self.state.objects[id as usize];
        assert!(object.is_token());
        assert!(!matches!(object.zone, Zone::Battlefield | Zone::Outside));
        if object.zone == Zone::Library {
            self.state.players[object.owner.index()]
                .library
                .retain(|&card| card != id);
        }
        self.state.exile_links.retain(|link| {
            link.exiled
                != ObjectRef {
                    card: id,
                    generation: object.generation,
                }
        });
        object.zone = Zone::Outside;
        object.generation = object
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        object.tapped = false;
        object.summoning_sick = false;
        object.damage = 0;
        object.damaged_this_turn = false;
        object.dealt_damage_this_turn = false;
        object.deathtouch_since_sba = false;
        object.growth = Stats::default();
        object.gained_keywords = Keywords::NONE;
        object.counters = InitialCounters::default();
        object.used_abilities = 0;
        object.attached_to = None;
        // No zone-change event: this game object ceases to exist (CR111.7).
        self.set_counters(id, Counters::default());
        self.state.free_tokens.push(id);
    }
}
