//! Fixed token creation batches and snapshot-owned reusable object slots.
use super::*;

impl Game {
    pub(super) fn create_tokens(
        &mut self,
        controller: Player,
        token: Card,
        amount: u16,
        tapped: bool,
    ) {
        assert!(token.is_token(), "token instruction requires a factory");
        let mut events = Vec::with_capacity(amount as usize);
        for _ in 0..amount {
            let (id, generation) = match self.state.free_tokens.pop() {
                Some(id) => {
                    let old = &self.state.objects[id as usize];
                    assert_eq!(old.zone, Zone::Outside);
                    assert!(old.card.is_token());
                    (id, old.generation)
                }
                None => (
                    CardId::try_from(self.state.objects.len()).expect("object slots exhausted"),
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
                deathtouch_since_sba: false,
                growth: Stats::default(),
                gained_keywords: Keywords::NONE,
                counters: Counters::default(),
                used_abilities: 0,
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
                .expect("object generation exhausted");
            object.tapped = tapped;
            object.summoning_sick = true;
            self.register_static_source(id);
            events.push(ZoneChange {
                before,
                after: self.object_ref(id),
                destination: Zone::Battlefield,
                moved: true,
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
            .expect("object generation exhausted");
        object.tapped = false;
        object.summoning_sick = false;
        object.damage = 0;
        object.deathtouch_since_sba = false;
        object.growth = Stats::default();
        object.gained_keywords = Keywords::NONE;
        object.counters = Counters::default();
        object.used_abilities = 0;
        // No zone-change event: this game object ceases to exist (CR111.7).
        self.state.free_tokens.push(id);
    }
}
