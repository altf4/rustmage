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
        assert!(token.is_token(), "token instruction requires a factory");
        // Quantities are resolved once before any newcomer exists. Fail before
        // mutation if the entire simultaneous batch cannot fit the identity arena.
        let amount = usize::try_from(amount).capacity("token count representation exhausted");
        let fresh = amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&end| end <= u128::from(CardId::MAX) + 1)
            .capacity("object slots exhausted");
        if amount == 0 {
            return;
        }
        // Replacement effects inspect the pre-event battlefield once, before
        // any token in this simultaneous batch is installed as a new source.
        let tapped = tapped || self.opponent_entry_requires_tapped(token, controller);
        let mut events = Vec::new();
        events
            .try_reserve_exact(amount)
            .capacity("token batch allocation exhausted");
        self.state
            .objects
            .try_reserve(fresh)
            .capacity("object allocation exhausted");
        for _ in 0..amount {
            let (id, generation) = match self.state.free_tokens.pop() {
                Some(id) => {
                    let old = &self.state.objects[id as usize];
                    assert_eq!(old.zone, Zone::Outside);
                    assert!(old.card.is_token());
                    (id, old.generation)
                }
                None => (
                    CardId::try_from(self.state.objects.len()).capacity("object slots exhausted"),
                    0,
                ),
            };
            let object = Object {
                card: token,
                owner: controller,
                zone: Zone::Outside,
                generation,
                tapped: false,
                summoning_sick: false,
                damage: 0,
                damaged_this_turn: false,
                dealt_damage_this_turn: false,
                deathtouch_since_sba: false,
                growth: Stats::default(),
                gained_keywords: Keywords::NONE,
                counters: Counters::default(),
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
            self.register_battlefield_sources(id);
            events.push(ZoneChange {
                before,
                after: self.object_ref(id),
                destination: Zone::Battlefield,
                moved: true,
                sacrificed_by: None,
            });
        }
        // Every newcomer exists before any entry event inspects the battlefield.
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn cease_token(&mut self, id: CardId) {
        let object = &mut self.state.objects[id as usize];
        assert!(object.card.is_token());
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
        object.counters = Counters::default();
        object.used_abilities = 0;
        object.attached_to = None;
        // No zone-change event: this game object ceases to exist (CR111.7).
        self.state.free_tokens.push(id);
    }
}
