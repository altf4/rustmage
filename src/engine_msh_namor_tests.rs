//! Whole Namor through paid actions; frozen symbols and all-zone Merfolk CDA.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
use crate::cards::{CastFilter, ChoiceColor};
const NAME: &str = "Namor the Sub-Mariner";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game) {
    for _ in 0..30 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                        | Action::ChooseTeamwork { pay: false }
                        | Action::FinishTeamwork
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap_or_else(|| panic!("unfinished {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("unfinished");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..220 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn namor(g: &mut Game, p: Player) -> CardId {
    paid(g, p, NAME, &[])
}
fn blue(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::MshBlueMerfolkToken
                && o.is_token())
            .then_some(id as CardId)
        })
        .collect()
}
fn exact(g: &Game, id: CardId) {
    let o = g.object_view(id);
    assert_eq!(o.name(), "Merfolk Token");
    assert_eq!(o.card, Card::MshBlueMerfolkToken);
    assert_eq!((o.power, o.toughness, o.colors), (1, 1, 2));
    assert_eq!(g.current_kinds(id), 1);
    assert!(o.has_subtype("Merfolk"));
    assert_eq!(o.keywords, Keywords::NONE);
    assert!(!o.tapped);
    assert!(o.summoning_sick);
    assert!(!g.state.attackers.iter().any(|a| a.card == id));
    assert!(!o.card.is_legendary());
}
fn event(g: &Game, n: CardId) -> &CastPaymentEvent {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|s| s.card == n && s.kind == StackKind::TriggeredAbility { ability: 0 })
        .and_then(|s| s.cast_event.as_deref())
        .expect("Namor owns actual cast event")
}
fn trigger(g: &mut Game, n: CardId) {
    until(g, |g| {
        g.state
            .stack
            .iter()
            .any(|s| s.card == n && s.kind == StackKind::TriggeredAbility { ability: 0 })
    });
}
fn declare(g: &mut Game, id: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn xspell(g: &mut Game, p: Player, value: u32) -> CardId {
    // Keep the complete interval small so the shared indexed helper enumerates
    // the chosen scalar; ordinary clients can submit every point in larger domains.
    if g.state.players[p.index()]
        .mana
        .iter()
        .map(|v| u64::from(*v))
        .sum::<u64>()
        > 64
    {
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, value];
    }
    let id = named(g, p, "Mind Spring", Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value });
    finish(g);
    id
}
#[test]
fn msh_namor_paid_body_cost_and_missing_blue_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAME, Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        let o = g.object_view(id);
        assert_eq!((o.power, o.toughness), (1, 4));
        assert!(o.keywords.contains(Keywords::FLYING));
        assert!(o.card.is_legendary());
        for s in ["Mutant", "Merfolk", "Villain"] {
            assert!(o.has_subtype(s));
        }
        assert!(blue(&g, p).is_empty());
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_namor_blue_spell_symbols_resolve_before_triggering_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Confiscate", Zone::Hand);
        declare(&mut g, spell, &[Target::Card(target)]);
        assert!(blue(&g, p).is_empty());
        assert_eq!(
            event(&g, n).card.mana_symbols_of_color(ChoiceColor::Blue),
            2
        );
        until(&mut g, |g| blue(g, p).len() == 2);
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert_eq!(g.object_view(target).controller, p.other());
        for id in blue(&g, p) {
            exact(&g, id);
        }
        assert_eq!(g.stats(n).power, 3);
        settle(&mut g);
        assert_eq!(g.object_view(target).controller, p);
    }
}
#[test]
fn msh_namor_hybrid_red_payment_still_counts_two_blue_symbols() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        let spell = named(&mut g, p, "Teach by Example", Zone::Hand);
        declare(&mut g, spell, &[]);
        assert_eq!(event(&g, n).payment, [0, 0, 0, 2, 0, 0]);
        assert_eq!(
            event(&g, n).card.mana_symbols_of_color(ChoiceColor::Blue),
            2
        );
        settle(&mut g);
        assert_eq!(blue(&g, p).len(), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_namor_twobrid_generic_payment_and_counter_preserve_one_symbol() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        let spell = named(&mut g, p, "Rakshasa's Bargain", Zone::Hand);
        declare(&mut g, spell, &[]);
        trigger(&mut g, n);
        assert_eq!(event(&g, n).payment, [0, 0, 0, 0, 0, 6]);
        assert_eq!(
            event(&g, n).card.mana_symbols_of_color(ChoiceColor::Blue),
            1
        );
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        let counter = named(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(blue(&g, p).len(), 1);
    }
}
#[test]
fn msh_namor_paid_blue_generic_mana_does_not_make_symbols() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        namor(&mut g, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        let spell = named(&mut g, p, "Vibranium Energy Daggers", Zone::Hand);
        declare(&mut g, spell, &[]);
        settle(&mut g);
        assert!(blue(&g, p).is_empty());
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_namor_noncreature_blue_permanent_qualifies_creature_does_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        paid(&mut g, p, "Picklock Prankster", &[]);
        assert!(blue(&g, p).is_empty());
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Witness Protection", &[Target::Card(target)]);
        assert_eq!(blue(&g, p).len(), 1);
        assert_eq!(g.stats(n).power, 2);
    }
}
#[test]
fn msh_namor_free_and_reduced_casts_keep_printed_symbols() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            let mut g = setup(p);
            namor(&mut g, p);
            if free {
                let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let spell = named(&mut g, p, "Confiscate", Zone::Hand);
                named(&mut g, p, "Omniscience", Zone::Battlefield);
                g.state.players[p.index()].mana = [0; 6];
                checked(
                    &mut g,
                    Action::AnnounceFree {
                        card: spell,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                finish(&mut g);
            } else {
                named(&mut g, p, "Archmage of Runes", Zone::Battlefield);
                g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
                xspell(&mut g, p, 1);
            }
            settle(&mut g);
            assert_eq!(blue(&g, p).len(), 2);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_namor_x_does_not_change_printed_symbol_quantity() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 7] {
            let mut g = setup(p);
            let n = namor(&mut g, p);
            let spell = xspell(&mut g, p, x);
            assert_eq!(
                event(&g, n).card.mana_symbols_of_color(ChoiceColor::Blue),
                2
            );
            assert!(
                g.state
                    .stack
                    .iter()
                    .any(|s| s.card == spell && s.x == Some(x))
            );
            settle(&mut g);
            assert_eq!(blue(&g, p).len(), 2);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(x)
            );
        }
    }
}
#[test]
fn msh_namor_adventure_uses_noncreature_chosen_face() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let spell = named(&mut g, p, "Picklock Prankster", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: spell });
        finish(&mut g);
        assert_eq!(
            event(&g, n).card,
            Card::from_name("Picklock Prankster")
                .unwrap()
                .adventure_spell()
                .unwrap()
        );
        until(&mut g, |g| blue(g, p).len() == 1);
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        // Stop after the Namor packet; the unrelated library selection is not needed here.
    }
}
#[test]
fn msh_namor_source_theft_departure_and_blink_do_not_redirect_tokens() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let n = namor(&mut g, p);
            let spell = xspell(&mut g, p, 0);
            trigger(&mut g, n);
            let original = event(&g, n).clone();
            match change {
                0 => {
                    let r = g.object_ref(n);
                    g.change_controller(r, p.other());
                }
                1 => {
                    g.move_object(n, Zone::Exile);
                }
                _ => {
                    g.move_object(n, Zone::Exile);
                    g.move_object(n, Zone::Battlefield);
                }
            }
            assert_eq!(*event(&g, n), original);
            settle(&mut g);
            assert_eq!(blue(&g, p).len(), 2);
            assert!(blue(&g, p.other()).is_empty());
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_namor_paid_counter_of_spell_does_not_cancel_original_token_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let spell = xspell(&mut g, p, 0);
        trigger(&mut g, n);
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        let counter = named(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(blue(&g, p).len(), 2);
    }
}
#[test]
fn msh_namor_same_physical_spell_recast_has_separate_frozen_packets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let victim = named(&mut g, p.other(), "Shock", Zone::Hand);
        checked(&mut g, Action::Pass);
        cast(&mut g, victim, None, &[Target::Player(p)]);
        checked(&mut g, Action::Pass);
        let spell = named(&mut g, p, "Negate", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Spell(victim)]);
        trigger(&mut g, n);
        let first = event(&g, n).clone();
        // Bounded public bounce fixture; recasting this instant is legal above
        // the still-pending old Namor packet and original Shock.
        g.move_object(spell, Zone::Hand);
        g.state
            .stack
            .retain(|s| !(s.card == spell && s.kind == StackKind::Spell));
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        cast(&mut g, spell, None, &[Target::Spell(victim)]);
        let packets: Vec<_> = g
            .state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .filter_map(|s| s.cast_event.as_deref())
            .filter(|e| e.card == first.card)
            .collect();
        assert_eq!(packets.len(), 2);
        assert_ne!(packets[0].stack_id, packets[1].stack_id);
        assert_ne!(packets[0].object, packets[1].object);
        settle(&mut g);
        assert_eq!(blue(&g, p).len(), 2);
    }
}
#[test]
fn msh_namor_cda_all_zones_and_current_controller_then_owner() {
    for p in [Player::P0, Player::P1] {
        for zone in [
            Zone::Hand,
            Zone::Library,
            Zone::Stack,
            Zone::Graveyard,
            Zone::Exile,
        ] {
            let mut g = setup(p);
            let n = named(&mut g, p, NAME, zone);
            let own = named(&mut g, p, "Attuma, Atlantean Warlord", Zone::Battlefield);
            named(
                &mut g,
                p.other(),
                "Attuma, Atlantean Warlord",
                Zone::Battlefield,
            );
            assert_eq!((g.stats(n).power, g.stats(n).toughness), (1, 4));
            g.move_object(own, Zone::Exile);
            assert_eq!(g.stats(n).power, 0);
        }
        let mut g = setup(p);
        let n = named(&mut g, p, NAME, Zone::Battlefield);
        named(&mut g, p, "Attuma, Atlantean Warlord", Zone::Battlefield);
        g.create_tokens(p.other(), Card::MshBlueMerfolkToken, 3, false);
        assert_eq!(
            g.characteristic_query()
                .base_stats_without_hand(n, None)
                .power,
            2
        );
        let r = g.object_ref(n);
        g.change_controller(r, p.other());
        assert_eq!(g.stats(n).power, 4);
        g.move_object(n, Zone::Graveyard);
        assert_eq!(g.object_view(n).controller, p);
        assert_eq!(g.stats(n).power, 1);
    }
}
struct KindredWorld<'a> {
    g: &'a Game,
    id: CardId,
}
impl CharacteristicWorld for KindredWorld<'_> {
    fn game(&self) -> &Game {
        self.g
    }
    fn object(&self, id: CardId) -> &Object {
        &self.g.state.objects[id as usize]
    }
    fn life(&self, p: Player) -> i128 {
        self.g.state.players[p.index()].life
    }
    fn gained_life(&self, _: Player) -> bool {
        false
    }
    fn attacking(&self, _: CardId) -> bool {
        false
    }
    fn kinds(&self, id: CardId) -> u16 {
        if id == self.id {
            128
        } else {
            self.g.current_kinds(id)
        }
    }
}
#[test]
fn msh_namor_changelings_and_noncreature_merfolk_count_without_creature_filter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let other = paid(&mut g, p, "Barkform Harvester", &[]);
        assert_eq!(g.stats(n).power, 2);
        assert!(g.has_subtype(other, "Merfolk"));
        let world = KindredWorld { g: &g, id: other };
        assert_eq!(world.kinds(other) & 1, 0);
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .stats(n)
                .power,
            2
        );
    }
}
#[test]
fn msh_namor_ability_removal_restores_cda_and_suppresses_future_casts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        g.create_tokens(p, Card::MshBlueMerfolkToken, 2, false);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(n)]);
        let existing = blue(&g, p).len();
        assert_eq!((g.stats(n).power, g.stats(n).toughness), (1, 1));
        assert!(!g.keywords(n).contains(Keywords::FLYING));
        paid(&mut g, p, "Vibranium Energy Daggers", &[]);
        assert_eq!(blue(&g, p).len(), existing);
        let draw = xspell(&mut g, p, 0);
        settle(&mut g);
        assert_eq!(g.object_view(draw).zone, Zone::Graveyard);
        assert_eq!(blue(&g, p).len(), existing);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(n).contains(Keywords::FLYING));
        assert_eq!(g.stats(n).power, 4);
    }
}
#[test]
fn msh_namor_cda_precedes_base_setter_lord_and_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        named(&mut g, p, "Attuma, Atlantean Warlord", Zone::Battlefield);
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(n)]);
        assert_eq!((g.stats(n).power, g.stats(n).toughness), (4, 6));
        let item = g.cast_preview(p, n, Zone::Battlefield, g.object_ref(n), 0, [0; 6]);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((5, 5)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        assert_eq!((g.stats(n).power, g.stats(n).toughness), (7, 7));
    }
}
#[test]
fn msh_namor_simultaneous_departures_freeze_intact_cda_lki_in_both_orders() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let n = named(&mut g, p, NAME, Zone::Battlefield);
            let lord = named(&mut g, p, "Attuma, Atlantean Warlord", Zone::Battlefield);
            g.create_tokens(p, Card::MshBlueMerfolkToken, 1, false);
            let t = blue(&g, p)[0];
            assert_eq!(g.stats(n).power, 4);
            let before = g.object_view(n);
            assert!(g.queue_trigger(&before, 0));
            let mut ids = vec![n, lord, t];
            if reverse {
                ids.reverse();
            }
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            assert_eq!(
                events
                    .iter()
                    .find(|e| e.before.id == n)
                    .unwrap()
                    .before
                    .power,
                4
            );
            assert_eq!(
                g.state.pending_triggers.last().unwrap().source_facts.power,
                4
            );
            assert_eq!(g.stats(n).power, 0);
        }
    }
}
#[test]
fn msh_namor_flying_changes_actual_block_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let flyer = named(&mut g, p.other(), "Aerial Doombot", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: n });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: n,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: flyer,
                attacker: n,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        settle(&mut g);
    }
}
#[test]
fn msh_namor_prepared_card_copy_is_an_actual_paid_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let source = paid(&mut g, p, "Emeritus of Ideation", &[]);
        let copy = g
            .observe(p)
            .preparations
            .iter()
            .find(|v| v.permanent == g.object_ref(source))
            .unwrap()
            .spell_copy;
        checked(&mut g, Action::AnnounceExile { card: copy.card });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        finish(&mut g);
        assert_eq!(
            event(&g, n).card.mana_symbols_of_color(ChoiceColor::Blue),
            1
        );
        settle(&mut g);
        assert_eq!(blue(&g, p).len(), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
    }
}
#[test]
fn msh_namor_stack_spell_copy_is_not_an_additional_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        namor(&mut g, p);
        let a = named(&mut g, p, "Teach by Example", Zone::Hand);
        declare(&mut g, a, &[]);
        settle(&mut g);
        assert_eq!(blue(&g, p).len(), 2);
        let b = xspell(&mut g, p, 0);
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(blue(&g, p).len(), 4);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
    }
}
#[test]
fn msh_namor_token_replacement_uses_exact_blue_recipe_and_updates_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        xspell(&mut g, p, 0);
        settle(&mut g);
        assert_eq!(blue(&g, p).len(), 4);
        for id in blue(&g, p) {
            exact(&g, id);
        }
        assert_eq!(g.stats(n).power, 5);
        let old = Card::from_name("@token/2cff3b0b2f34a41fce4cd46b").unwrap();
        assert_eq!(old.index(), 5247);
        assert_eq!(old.name(), "Merfolk Token");
        assert_eq!(old.colors(), 3);
        assert_eq!(Card::from_name("Merfolk Token"), None);
    }
}
#[test]
fn msh_namor_hidden_hand_cda_and_draw_keep_full_nonactor_observation_private() {
    for p in [Player::P0, Player::P1] {
        let mut a = setup(p);
        named(&mut a, p, NAME, Zone::Hand);
        let mut b = setup(p);
        named(&mut b, p, "Bear Cub", Zone::Hand);
        assert_eq!(
            serde_json::to_value(a.observe(p.other())).unwrap(),
            serde_json::to_value(b.observe(p.other())).unwrap()
        );
        let mut a = setup(p);
        let mut b = setup(p);
        namor(&mut a, p);
        namor(&mut b, p);
        named(&mut a, p, "Forest", Zone::Library);
        named(&mut b, p, "Shock", Zone::Library);
        xspell(&mut a, p, 1);
        xspell(&mut b, p, 1);
        settle(&mut a);
        settle(&mut b);
        assert_eq!(
            serde_json::to_value(a.observe(p.other())).unwrap(),
            serde_json::to_value(b.observe(p.other())).unwrap()
        );
        assert_ne!(
            serde_json::to_value(a.observe(p)).unwrap(),
            serde_json::to_value(b.observe(p)).unwrap()
        );
    }
}
#[test]
fn msh_namor_no_event_or_tokens_until_all_targets_and_payment_are_complete() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        let spell = named(&mut g, p, "Confiscate", Zone::Hand);
        let bad = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let good = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bad),
            },
        );
        assert!(g.state.pending_triggers.is_empty());
        assert!(blue(&g, p).is_empty());
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(good),
            },
        );
        finish(&mut g);
        assert_eq!(
            event(&g, n).card.mana_symbols_of_color(ChoiceColor::Blue),
            2
        );
        settle(&mut g);
        assert_eq!(blue(&g, p).len(), 2);
    }
}
#[test]
fn msh_namor_orphaned_packets_raw_index_restore_keep_frozen_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = namor(&mut g, p);
        xspell(&mut g, p, 0);
        trigger(&mut g, n);
        let before = g.snapshot();
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let action = Action::Pass;
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let (mut raw, mut indexed) = (g.clone(), g.clone());
        drop(g);
        raw.apply(actor, action.clone()).unwrap();
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for child in [&mut raw, &mut indexed] {
            child.restore(&before).unwrap();
            reject(child, Action::ChooseX { value: 1 });
            checked(child, action.clone());
            child.move_object(n, Zone::Exile);
            settle(child);
            assert_eq!(blue(child, p).len(), 2);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn msh_namor_teamwork_cannot_use_future_tokens_and_live_power_pays_exact_threshold() {
    for p in [Player::P0, Player::P1] {
        for enough in [false, true] {
            let mut g = setup(p);
            let n = namor(&mut g, p);
            if enough {
                g.create_tokens(p, Card::MshBlueMerfolkToken, 1, false);
            }
            let baseline = blue(&g, p).len();
            let victim = named(&mut g, p.other(), "Shock", Zone::Hand);
            if g.actor() != Some(p.other()) {
                checked(&mut g, Action::Pass);
            }
            cast(&mut g, victim, None, &[Target::Player(p)]);
            if g.actor() != Some(p) {
                checked(&mut g, Action::Pass);
            }
            let spell = named(&mut g, p, "We Say Thee Nay!", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            if !enough {
                reject(&mut g, Action::ChooseTeamwork { pay: true });
            }
            checked(&mut g, Action::ChooseTeamwork { pay: enough });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Spell(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            if enough {
                let clause = g
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .cast_costs
                    .as_ref()
                    .unwrap()
                    .clauses
                    .iter()
                    .position(|c| matches!(c.clause, CastCostClause::Teamwork(_)))
                    .unwrap() as u16;
                checked(&mut g, Action::ChooseCastCost { clause });
                let r = g.object_ref(n);
                checked(&mut g, Action::ChooseTeammate { object: r });
                checked(&mut g, Action::FinishTeamwork);
            }
            finish(&mut g);
            assert_eq!(blue(&g, p).len(), baseline);
            until(&mut g, |g| blue(g, p).len() == baseline + 1);
            assert_eq!(g.object_view(n).tapped, enough);
        }
    }
}
#[test]
fn msh_namor_symbol_retention_is_opted_nested_and_contextual_filter_safe() {
    let nested = AmountValue::Sum(&[
        AmountValue::Fixed(1),
        AmountValue::Multiple {
            value: &AmountValue::CastSpellManaSymbols(ChoiceColor::Blue),
            factor: 2,
        },
    ]);
    assert!(cast_symbol_quantities::selected(&[
        Instruction::CreateTokensScaled {
            token: Card::MshBlueMerfolkToken,
            amount: nested,
            tapped: false
        }
    ]));
    assert!(cast_symbol_quantities::selected(&[Instruction::Modal(&[
        SpellMode {
            text: "test",
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::CastSpellManaSymbols(ChoiceColor::Blue)
            )]
        }
    ])]));
    assert!(!cast_symbol_quantities::selected(&[
        Instruction::DrawScaled(PlayerSet::Controller, AmountValue::SpellColors)
    ]));
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = named(&mut g, p, NAME, Zone::Battlefield);
        let view = g.object_view(n);
        assert_eq!(
            g.amount_value(
                AmountValue::CastSpellManaSymbols(ChoiceColor::Blue),
                view.reference,
                p
            ),
            0
        );
        let filter = CastFilter::ManaSymbols {
            filter: &CastFilter::SourceChosenColor,
            color: ChoiceColor::Blue,
        };
        assert!(!g.cast_filter_matches_value(
            Some(p),
            filter,
            Card::from_name("Opt").unwrap(),
            1,
            p,
            &view,
            &[]
        ));
        let mut chosen = view.clone();
        chosen.entry_choice = Some(EntryChoiceValue::Color(ChoiceColor::Blue));
        assert!(g.cast_filter_matches_value(
            Some(p),
            filter,
            Card::from_name("Opt").unwrap(),
            1,
            p,
            &chosen,
            &[]
        ));
    }
}
