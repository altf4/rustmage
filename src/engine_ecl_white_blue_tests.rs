//! Whole-card ECL rules scenarios using paid actions and owned indexed branches.
use super::ability_timeline_tests::{checked, settle, setup};
use super::tests::{cast, spawn};
use super::*;
use crate::ChoiceColor;

fn mare(g: &mut Game, p: Player) -> CardId {
    let id = spawn(g, p, Card::from_name("Loch Mare").unwrap(), Zone::Hand);
    g.state.players[p.index()].mana[1] = 20;
    cast(g, id, None);
    settle(g);
    assert_eq!(g.object_view(id).counters.minus_one, 3);
    id
}
fn activate(g: &mut Game, source: CardId, ability_index: u16, target_id: Option<CardId>) {
    let action = g.legal_actions().into_iter().find(|a| matches!(a,
        Action::Activate {card, ability, target: None, ..} | Action::BeginActivate {card, ability, ..}
        if *card == source && *ability == ability_index))
        .expect("paid activation available");
    checked(g, action);
    if let Some(id) = target_id {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
}

#[test]
fn loch_mare_paid_draw_tap_stun_and_no_counter_legality() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = mare(&mut g, player);
        let target = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == player && o.zone == Zone::Hand)
            .count();
        activate(&mut g, source, 0, None);
        assert_eq!(g.object_view(source).counters.minus_one, 2);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == player && o.zone == Zone::Hand)
                .count(),
            hand + 1
        );
        activate(&mut g, source, 1, Some(target));
        assert_eq!(g.object_view(source).counters.minus_one, 0);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        assert_eq!(g.object_view(target).counters.stun, 1);
        assert!(
            !g.legal_actions().iter().any(|a| matches!(a,
            Action::Activate { card, .. } | Action::BeginActivate { card, .. } if *card == source))
        );
    }
}

#[test]
fn loch_mare_mixed_counter_payment_atomicity_and_departed_source() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    g.set_counters(
        source,
        Counters::of(CounterKind::Stun, 1).with(CounterKind::Soul, 1),
    );
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::BeginActivate {card, ability: 1, counter_payment, ..}
        if *card == source && counter_payment.get(CounterKind::Stun) == 1
            && counter_payment.get(CounterKind::Soul) == 1)
        })
        .unwrap();
    let mut forged = action.clone();
    if let Action::BeginActivate {
        counter_payment, ..
    } = &mut forged
    {
        *counter_payment = Counters::of(CounterKind::Page, 2);
    }
    let before = g.snapshot();
    assert_eq!(g.apply(Player::P0, forged), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    checked(&mut g, action);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    g.move_object(source, Zone::Graveyard);
    settle(&mut g);
    assert!(g.object_view(target).tapped);
    assert_eq!(g.object_view(target).counters.stun, 1);
}

#[test]
fn loch_mare_invalid_target_keeps_paid_cost_and_does_not_stun_reentry() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    activate(&mut g, source, 1, Some(target));
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert!(!g.object_view(target).tapped);
    assert_eq!(g.object_view(target).counters.stun, 0);
    assert_eq!(g.object_view(source).counters.minus_one, 1);
}

fn cast_named(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    cast(g, id, None);
    settle(g);
    id
}

#[test]
fn encumbered_reejerey_removes_on_tap_only_while_counter_exists() {
    let mut g = setup(Player::P0);
    let id = cast_named(&mut g, Player::P0, "Encumbered Reejerey");
    assert_eq!(g.object_view(id).counters.minus_one, 3);
    for left in (0..3).rev() {
        g.tap_objects(&[id]);
        settle(&mut g);
        assert_eq!(g.object_view(id).counters.minus_one, left);
        // Becoming tapped requires an actual untapped-to-tapped event.
        g.tap_objects(&[id]);
        assert!(g.state.pending_triggers.is_empty());
        g.untap_objects(&[id], untap_rules::UntapCause::Effect);
    }
    g.tap_objects(&[id]);
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(
        g.stats(id),
        Stats {
            power: 5,
            toughness: 4
        }
    );
}

#[test]
fn reluctant_dounguard_qualifies_only_other_owned_entries_and_exact_source() {
    let mut g = setup(Player::P0);
    let id = cast_named(&mut g, Player::P0, "Reluctant Dounguard");
    assert_eq!(g.object_view(id).counters.minus_one, 2);
    let other = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Hand);
    g.move_object(other, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.minus_one, 2);
    cast_named(&mut g, Player::P0, "Grizzly Bears");
    assert_eq!(g.object_view(id).counters.minus_one, 1);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    g.prepare_triggers();
    // The queued trigger cannot remove a counter from a new incarnation.
    g.move_object(id, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.minus_one, 2);
    cast_named(&mut g, Player::P0, "Grizzly Bears");
    cast_named(&mut g, Player::P0, "Grizzly Bears");
    assert_eq!(g.object_view(id).counters.minus_one, 0);
    let last = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Hand);
    g.move_object(last, Zone::Battlefield);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn gallant_fowlknight_boosts_resolution_group_and_expires_at_cleanup() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let fowl = cast_named(&mut g, Player::P0, "Gallant Fowlknight");
    assert_eq!(g.stats(bear).power, 3);
    assert_eq!(g.stats(enemy).power, 2);
    assert_eq!(g.stats(fowl).power, 4);
    assert!(g.keywords(fowl).contains(Keywords::FIRST_STRIKE));
    assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
    let late = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Hand);
    g.move_object(late, Zone::Battlefield);
    assert_eq!(g.stats(late).power, 2);
    g.move_object(fowl, Zone::Graveyard);
    assert_eq!(g.stats(bear).power, 3);
    g.finish_cleanup();
    assert_eq!(g.stats(bear).power, 2);
}

fn spell(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    super::ability_timeline_tests::cast(g, id, None, targets);
    id
}
fn changelings(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card
                    == Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap()
        })
        .count()
}

#[test]
fn rime_chill_vivid_cost_zero_targets_and_partial_target_failure() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        spawn(&mut g, player, Card::GrizzlyBears, Zone::Battlefield);
        spawn(
            &mut g,
            player,
            Card::from_name("Gallant Fowlknight").unwrap(),
            Zone::Battlefield,
        );
        // Two colors, not two permanents or colored mana symbols: {6}{U} becomes {4}{U}.
        g.state.players[player.index()].mana = [0, 5, 0, 0, 0, 0];
        let hand = g.hand_count(player);
        spell(&mut g, player, "Rime Chill", &[]);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.hand_count(player), hand + 1);
        g.state.players[player.index()].mana = [0, 5, 0, 0, 0, 0];
        let a = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        let b = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        spell(
            &mut g,
            player,
            "Rime Chill",
            &[Target::Card(a), Target::Card(b)],
        );
        g.move_object(a, Zone::Hand);
        settle(&mut g);
        assert!(g.object_view(b).tapped);
        assert_eq!(g.object_view(b).counters.stun, 1);
        assert_eq!(g.hand_count(player), hand + 2);
    }
}

#[test]
fn rime_chill_all_illegal_targets_fizzles_draw() {
    let mut g = setup(Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let before = g.hand_count(Player::P0);
    spell(&mut g, Player::P0, "Rime Chill", &[Target::Card(target)]);
    g.move_object(target, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), before);
}

#[test]
fn tributary_vaulter_restricts_targets_and_boost_survives_source_departure() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Tributary Vaulter");
    let recipient = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Silvergill Peddler").unwrap(),
        Zone::Battlefield,
    );
    let enemy = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Silvergill Peddler").unwrap(),
        Zone::Battlefield,
    );
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    g.tap_objects(&[source]);
    g.prepare_triggers();
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        checked(&mut g, action);
    }
    for bad in [source, enemy, bear] {
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bad)
        }));
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    g.move_object(source, Zone::Graveyard);
    let base = g.stats(recipient).power;
    settle(&mut g);
    assert_eq!(g.stats(recipient).power, base + 2);
    g.finish_cleanup();
    assert_eq!(g.stats(recipient).power, base);
}

#[test]
fn personify_returns_foreign_owned_creature_and_creates_real_changeling() {
    let mut g = setup(Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    g.record_control(g.object_ref(target), Player::P0, None);
    g.add_counter(target, CounterKind::PlusOne, 2);
    let old = g.object_ref(target);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(target)]);
    settle(&mut g);
    let view = g.object_view(target);
    assert_eq!(view.zone, Zone::Battlefield);
    assert_eq!(view.controller, Player::P1);
    assert_ne!(view.reference, old);
    assert!(view.counters.is_empty());
    assert_eq!(changelings(&g, Player::P0), 1);
    let token = g
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    == Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap()
        })
        .unwrap() as CardId;
    for subtype in ["Merfolk", "Kithkin", "Coward", "Elf"] {
        assert!(g.object_view(token).has_subtype(subtype));
    }
    assert_eq!(
        g.stats(token),
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert_eq!(g.object_view(token).colors, 0);
}

#[test]
fn personify_tokens_do_not_return_and_invalid_targets_make_no_token() {
    let mut g = setup(Player::P0);
    g.create_tokens(
        Player::P0,
        Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap(),
        1,
        false,
    );
    let token = g
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    == Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap()
        })
        .unwrap() as CardId;
    let old = g.object_ref(token);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(token)]);
    settle(&mut g);
    assert_eq!(changelings(&g, Player::P0), 1);
    assert_ne!(g.object_ref(token), old);
    let target = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(target)]);
    g.move_object(target, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(changelings(&g, Player::P0), 1);
}

#[test]
fn crib_swap_tracks_resolution_controller_and_invalid_incarnation() {
    let mut g = setup(Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let crib = spell(&mut g, Player::P0, "Crib Swap", &[Target::Card(target)]);
    assert!(Card::from_name("Crib Swap").unwrap().has_subtype("Merfolk"));
    g.record_control(g.object_ref(target), Player::P0, None);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
    assert_eq!(changelings(&g, Player::P0), 1);
    assert_eq!(changelings(&g, Player::P1), 0);
    assert_eq!(g.object_view(crib).zone, Zone::Graveyard);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Crib Swap", &[Target::Card(target)]);
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(changelings(&g, Player::P0), 1);
    assert_eq!(changelings(&g, Player::P1), 0);
}

#[test]
fn personify_finishes_returned_entry_choice_before_creating_token() {
    let mut g = setup(Player::P0);
    let mare = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Diamond Mare").unwrap(),
        Zone::Battlefield,
    );
    spell(&mut g, Player::P0, "Personify", &[Target::Card(mare)]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    assert_eq!(changelings(&g, Player::P0), 0);
    // Dispose the source branch while the returned card's own entry is suspended.
    let mut g = {
        let child = g.clone();
        drop(g);
        child
    };
    checked(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    settle(&mut g);
    assert_eq!(
        g.object_view(mare).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Blue))
    );
    assert_eq!(changelings(&g, Player::P0), 1);
}

#[test]
fn noggle_preserves_artifact_type_removes_abilities_color_and_creature_types() {
    let mut g = setup(Player::P0);
    let target = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Diamond Mare").unwrap(),
        Zone::Battlefield,
    );
    g.add_counter(target, CounterKind::PlusOne, 2);
    // Flash works during an opponent's main phase.
    g.state.active = Player::P1;
    spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(target)],
    );
    settle(&mut g);
    let aura = g
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield && o.card == Card::from_name("Noggle the Mind").unwrap()
        })
        .unwrap() as CardId;
    let view = g.object_view(target);
    assert_eq!(view.kinds & 17, 17);
    assert_eq!(view.colors, 0);
    assert!(view.has_subtype("Noggle"));
    assert!(!view.has_subtype("Horse"));
    assert_eq!(
        g.stats(target),
        Stats {
            power: 3,
            toughness: 3
        }
    );
    assert!(!g.printed_abilities_available(target));
    g.move_object(aura, Zone::Graveyard);
    assert!(g.object_view(target).has_subtype("Horse"));
    assert!(g.printed_abilities_available(target));
    assert_eq!(
        g.stats(target),
        Stats {
            power: 3,
            toughness: 5
        }
    );
}

fn hold_target(g: &mut Game, target: CardId) -> CardId {
    let source = spell(g, Player::P0, "Liminal Hold", &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        checked(g, a);
    }
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    source
}
#[test]
fn liminal_hold_optional_target_life_and_owner_return() {
    let mut g = setup(Player::P0);
    cast_named(&mut g, Player::P0, "Liminal Hold");
    assert_eq!(g.state.players[0].life, 22);
    let target = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.record_control(g.object_ref(target), Player::P1, None);
    let source = hold_target(&mut g, target);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
    assert_eq!(g.state.players[0].life, 24);
    g.move_object(source, Zone::Graveyard);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.object_view(target).controller, Player::P0);
}
#[test]
fn liminal_hold_source_departure_still_gains_life_but_invalid_target_fizzles() {
    for source_departure in [false, true] {
        let mut g = setup(Player::P0);
        let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
        let source = hold_target(&mut g, target);
        if source_departure {
            g.move_object(source, Zone::Graveyard);
        } else {
            g.move_object(target, Zone::Hand);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if source_departure {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        assert_eq!(
            g.state.players[0].life,
            if source_departure { 22 } else { 20 }
        );
    }
}

#[test]
fn noggle_removes_colored_flying_vigilance_and_reverses_on_aura_departure() {
    let mut g = setup(Player::P0);
    let angel = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Serra Angel").unwrap(),
        Zone::Battlefield,
    );
    assert_ne!(g.object_view(angel).colors, 0);
    assert!(g.keywords(angel).contains(Keywords::FLYING));
    let aura = spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(angel)],
    );
    settle(&mut g);
    assert_eq!(g.object_view(angel).colors, 0);
    assert_eq!(
        g.stats(angel),
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert!(
        !g.keywords(angel)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    g.move_object(aura, Zone::Graveyard);
    assert_ne!(g.object_view(angel).colors, 0);
    assert_eq!(
        g.stats(angel),
        Stats {
            power: 4,
            toughness: 4
        }
    );
    assert!(
        g.keywords(angel)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
}

fn guardian_setup() -> (Game, CardId) {
    let mut g = setup(Player::P1);
    g.state.priority = Player::P0;
    let source = cast_named(&mut g, Player::P0, "Glen Elendra Guardian");
    assert_eq!(g.object_view(source).counters.minus_one, 1);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    assert_eq!(
        g.stats(source),
        Stats {
            power: 2,
            toughness: 3
        }
    );
    (g, source)
}
fn guardian_counter(g: &mut Game, source: CardId, victim: CardId) {
    checked(g, Action::Pass);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::BeginActivate {card, ability: 0, ..} if *card == source)
        })
        .unwrap();
    checked(g, action);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Spell(victim),
        },
    );
}
#[test]
fn glen_elendra_guardian_flash_paid_counter_draw_and_source_departure() {
    for departure in [false, true] {
        let (mut g, source) = guardian_setup();
        let bolt = spell(
            &mut g,
            Player::P1,
            "Lightning Bolt",
            &[Target::Player(Player::P0)],
        );
        let hand = g.hand_count(Player::P1);
        guardian_counter(&mut g, source, bolt);
        assert_eq!(g.object_view(source).counters.minus_one, 0);
        if departure {
            g.move_object(source, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(g.object_view(bolt).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].life, 20);
        assert_eq!(g.hand_count(Player::P1), hand + 1);
    }
}
#[test]
fn glen_elendra_guardian_any_named_counter_pays_and_uncounterable_still_draws() {
    let (mut g, source) = guardian_setup();
    g.set_counters(source, Counters::of(CounterKind::Soul, 1));
    spawn(
        &mut g,
        Player::P1,
        Card::from_name("Sphinx of the Final Word").unwrap(),
        Zone::Battlefield,
    );
    let bolt = spell(
        &mut g,
        Player::P1,
        "Lightning Bolt",
        &[Target::Player(Player::P0)],
    );
    let hand = g.hand_count(Player::P1);
    guardian_counter(&mut g, source, bolt);
    assert!(g.object_view(source).counters.is_empty());
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P1), hand + 1);
    assert_eq!(g.state.players[0].life, 17);
}
#[test]
fn glen_elendra_guardian_cannot_target_creature_spells() {
    let (mut g, source) = guardian_setup();
    spell(&mut g, Player::P1, "Grizzly Bears", &[]);
    checked(&mut g, Action::Pass);
    assert!(!g.legal_actions().iter().any(|a| matches!(a,
        Action::BeginActivate {card, ..} | Action::Activate {card, ..} if *card == source)));
}

#[test]
fn immediate_exile_programs_wait_for_departure_linked_entry_choices() {
    for name in ["Personify", "Crib Swap"] {
        let mut g = setup(Player::P0);
        let returning = spawn(
            &mut g,
            Player::P1,
            Card::from_name("Diamond Mare").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[returning as usize].tapped = true;
        let exiler = cast_named(&mut g, Player::P0, "Henchbots");
        assert_eq!(g.object_view(returning).zone, Zone::Exile);
        spell(&mut g, Player::P0, name, &[Target::Card(exiler)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(
            g.state.pending,
            Pending::EntryValue { player: Player::P1 }
        ));
        assert_eq!(g.object_view(exiler).zone, Zone::Exile);
        assert_eq!(changelings(&g, Player::P0), 0);
        assert!(g.state.resolving.as_ref().unwrap().continuation.is_some());
        let snapshot = g.snapshot();
        let bytes = g.state_bytes();
        let mut child = g.clone();
        drop(g);
        child.restore(&snapshot).unwrap();
        assert_eq!(child.state_bytes(), bytes);
        checked(
            &mut child,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Blue),
            },
        );
        settle(&mut child);
        assert_eq!(child.object_view(returning).zone, Zone::Battlefield);
        assert_eq!(
            child.object_view(exiler).zone,
            if name == "Personify" {
                Zone::Battlefield
            } else {
                Zone::Exile
            }
        );
        assert_eq!(changelings(&child, Player::P0), 1);
    }
}

fn thirst_choice(creature: bool) -> (Game, CardId) {
    let mut g = setup(Player::P0);
    let hand = spawn(
        &mut g,
        Player::P0,
        if creature {
            Card::GrizzlyBears
        } else {
            Card::Forest
        },
        Zone::Hand,
    );
    spell(&mut g, Player::P0, "Thirst for Identity", &[]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(matches!(
        g.state.pending,
        Pending::EffectChoice { player: Player::P0 }
    ));
    assert!(g.observe(Player::P1).effect_choice.is_none());
    assert_eq!(g.hand_count(Player::P0), 4);
    (g, hand)
}
#[test]
fn thirst_for_identity_can_discard_one_creature_and_rejects_noncreature_atomically() {
    let (mut g, creature) = thirst_choice(true);
    let land = g
        .state
        .objects
        .iter()
        .position(|o| o.owner == Player::P0 && o.zone == Zone::Hand && o.card == Card::Forest)
        .unwrap() as CardId;
    let before = g.snapshot();
    assert_eq!(
        g.apply(Player::P0, Action::Discard { card: land }),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    checked(&mut g, Action::Discard { card: creature });
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(Player::P0), 3);
}
#[test]
fn thirst_for_identity_private_phase_hides_creature_availability_and_decline_discards_two() {
    let (mut with_creature, creature) = thirst_choice(true);
    let (without_creature, _) = thirst_choice(false);
    assert_eq!(
        serde_json::to_value(with_creature.observe(Player::P1)).unwrap(),
        serde_json::to_value(without_creature.observe(Player::P1)).unwrap()
    );
    assert!(
        !without_creature
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Discard { .. }))
    );
    assert!(
        without_creature
            .legal_actions()
            .contains(&Action::DeclineEffectPayment)
    );
    checked(&mut with_creature, Action::DeclineEffectPayment);
    assert!(matches!(
        with_creature.state.pending,
        Pending::EffectDiscard { remaining: 2, .. }
    ));
    checked(&mut with_creature, Action::Discard { card: creature });
    // The two mandatory selections remain private and commit as one discard batch.
    assert_eq!(with_creature.object_view(creature).zone, Zone::Hand);
    let land = with_creature
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    checked(&mut with_creature, land);
    settle(&mut with_creature);
    assert_eq!(with_creature.hand_count(Player::P0), 2);
}
#[test]
fn thirst_for_identity_no_creature_cannot_auto_pay_and_fallback_handles_short_hand() {
    let (mut g, keep) = thirst_choice(false);
    // An independent fixture narrows the mandatory shortage path after checking
    // the printed spell's draw-three/private optional payment above.
    let remove: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| o.owner == Player::P0 && o.zone == Zone::Hand && *id as CardId != keep)
        .map(|(id, _)| id as CardId)
        .collect();
    for id in remove {
        g.move_object(id, Zone::Exile);
    }
    checked(&mut g, Action::DeclineEffectPayment);
    checked(&mut g, Action::Discard { card: keep });
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), 0);
    assert_eq!(g.object_view(keep).zone, Zone::Graveyard);
}

#[test]
fn illusion_spinners_as_though_flash_requires_controlled_faerie_including_changeling() {
    let mut g = setup(Player::P0);
    g.state.active = Player::P1;
    let spell_id = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Illusion Spinners").unwrap(),
        Zone::Hand,
    );
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell_id })
    );
    spawn(
        &mut g,
        Player::P1,
        Card::from_name("Glamermite").unwrap(),
        Zone::Battlefield,
    );
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell_id })
    );
    g.create_tokens(
        Player::P0,
        Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap(),
        1,
        false,
    );
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell_id })
    );
    super::ability_timeline_tests::cast(&mut g, spell_id, None, &[]);
    settle(&mut g);
    assert_eq!(g.object_view(spell_id).zone, Zone::Battlefield);
    assert!(
        g.keywords(spell_id)
            .contains(Keywords::FLYING.union(Keywords::HEXPROOF))
    );
    assert!(!g.keywords(spell_id).contains(Keywords::FLASH));
}
#[test]
fn illusion_spinners_untapped_hexproof_exact_incarnation_and_ability_removal() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Illusion Spinners");
    let original = g.object_ref(source);
    let enemy_spell = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Unsummon").unwrap(),
        Zone::Hand,
    );
    g.state.priority = Player::P1;
    assert!(!g.target_matches(
        TargetRule::Creature,
        Player::P1,
        g.object_ref(enemy_spell),
        Target::Card(source)
    ));
    g.tap_objects(&[source]);
    assert!(!g.keywords(source).contains(Keywords::HEXPROOF));
    assert!(g.target_matches(
        TargetRule::Creature,
        Player::P1,
        g.object_ref(enemy_spell),
        Target::Card(source)
    ));
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    assert!(g.keywords(source).contains(Keywords::HEXPROOF));
    g.state.priority = Player::P0;
    spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(source)],
    );
    settle(&mut g);
    assert!(
        !g.keywords(source)
            .contains(Keywords::HEXPROOF.union(Keywords::FLYING))
    );
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_eq!(
        g.stat_value(StatValue::SourceUntapped, original, Player::P0),
        0
    );
    assert_eq!(
        g.stat_value(StatValue::SourceUntapped, g.object_ref(source), Player::P0),
        1
    );
    assert!(g.keywords(source).contains(Keywords::HEXPROOF));
}

#[test]
fn adept_watershaper_other_tapped_owned_creatures_survive_lethal_damage_until_source_leaves() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Adept Watershaper");
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    g.tap_objects(&[source, bear, enemy]);
    assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
    assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
    assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    spell(&mut g, Player::P0, "Lightning Bolt", &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    g.move_object(source, Zone::Graveyard);
    g.state_based_actions();
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
}
#[test]
fn adept_watershaper_control_changes_untapping_and_ability_removal_update_grants() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Adept Watershaper");
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    g.tap_objects(&[bear, enemy]);
    assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    g.record_control(g.object_ref(source), Player::P1, None);
    assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    assert!(g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
    g.untap_objects(&[enemy], untap_rules::UntapCause::Effect);
    assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
    g.tap_objects(&[enemy]);
    spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(source)],
    );
    settle(&mut g);
    assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
}

fn equip_bark(g: &mut Game, recipient: CardId) -> CardId {
    let bark = cast_named(g, Player::P0, "Bark of Doran");
    checked(
        g,
        Action::AnnounceActivate {
            card: bark,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    settle(g);
    bark
}
fn bark_combat(g: &mut Game, attacker: CardId, blockers: &[CardId]) {
    g.state.objects[attacker as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: attacker });
    checked(g, Action::FinishAttackers);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    for &blocker in blockers {
        checked(g, Action::Block { blocker, attacker });
    }
    checked(g, Action::FinishBlockers);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
}
#[test]
fn bark_of_doran_paid_equip_toughness_combat_and_greater_power_branch() {
    for pump in [false, true] {
        let mut g = setup(Player::P0);
        let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        equip_bark(&mut g, bear);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert_eq!(g.combat_damage_amount(bear), 3);
        if pump {
            spell(&mut g, Player::P0, "Sure Strike", &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 5,
                    toughness: 3
                }
            );
            assert_eq!(g.combat_damage_amount(bear), 5);
        }
        bark_combat(&mut g, bear, &[]);
        assert_eq!(g.state.players[1].life, if pump { 15 } else { 17 });
    }
}
#[test]
fn bark_of_doran_blocker_and_multiblock_trample_use_toughness_amount() {
    let mut g = setup(Player::P0);
    let blocker = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    equip_bark(&mut g, blocker);
    let attacker = spawn(&mut g, Player::P1, Card::HillGiant, Zone::Battlefield);
    g.state.active = Player::P1;
    g.state.priority = Player::P1;
    bark_combat(&mut g, attacker, &[blocker]);
    assert_eq!(g.object_view(attacker).zone, Zone::Graveyard);
    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    let mut g = setup(Player::P0);
    let attacker = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Darksteel Colossus").unwrap(),
        Zone::Battlefield,
    );
    equip_bark(&mut g, attacker);
    let a = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let b = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    bark_combat(&mut g, attacker, &[a, b]);
    assert_eq!(g.damage_assignment_range().unwrap().maximum, 12);
    checked(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(a),
            amount: 2,
        },
    );
    checked(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(b),
            amount: 2,
        },
    );
    assert_eq!(g.state.players[1].life, 12);
    assert_eq!(g.stats(attacker).power, 11);
}
#[test]
fn bark_of_doran_equip_ownership_timing_and_attachment_departure_are_exact() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let bark = equip_bark(&mut g, bear);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::AnnounceActivate { card, .. } if *card == bark)
        })
        .unwrap();
    checked(&mut g, action);
    let before = g.snapshot();
    assert_eq!(
        g.apply(
            Player::P0,
            Action::ChooseTarget {
                target: Target::Card(enemy)
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    settle(&mut g);
    g.state.step = Step::Upkeep;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Activate {card,..} | Action::BeginActivate {card,..} | Action::AnnounceActivate {card,..} if *card==bark))
    );
    g.state.step = Step::Main1;
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    assert_eq!(g.combat_damage_amount(bear), 2);
    g.move_object(bark, Zone::Graveyard);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 2,
            toughness: 2
        }
    );
}

#[test]
fn bark_of_doran_rule_survives_recipient_ability_removal_and_stops_with_source_abilities() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let bark = equip_bark(&mut g, bear);
    spell(&mut g, Player::P0, "Noggle the Mind", &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.combat_damage_amount(bear), 2);
    bark_combat(&mut g, bear, &[]);
    assert_eq!(g.state.players[1].life, 18);
    g.state.players[0].mana = [30; 6];
    spell(&mut g, Player::P0, "Dive Down", &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.combat_damage_amount(bear), 5);
    spell(&mut g, Player::P0, "Stop Cold", &[Target::Card(bark)]);
    settle(&mut g);
    assert!(!g.continuous_fragment_available(bark, |fragment| matches!(
        fragment,
        crate::cards::StaticAbilityFragment::AttachmentCombatToughnessIfGreater
    )));
    assert_eq!(g.combat_damage_amount(bear), 1);
}
#[test]
fn bark_of_doran_double_strike_rechecks_current_toughness_in_each_damage_step() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    equip_bark(&mut g, bear);
    let equipment = cast_named(&mut g, Player::P0, "Fireshrieker");
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    settle(&mut g);
    bark_combat(&mut g, bear, &[]);
    assert_eq!(g.state.players[1].life, 17);
    g.state.players[0].mana = [30; 6];
    spell(&mut g, Player::P0, "Snakeskin Veil", &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.combat_damage_amount(bear), 4);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert_eq!(g.state.players[1].life, 13);
}

#[test]
fn bark_of_doran_unrelated_equipment_does_not_change_combat_assignment() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let equipment = cast_named(&mut g, Player::P0, "Fireshrieker");
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    settle(&mut g);
    spell(&mut g, Player::P0, "Dive Down", &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.stats(bear).toughness, 5);
    assert_eq!(g.combat_damage_amount(bear), 2);
    bark_combat(&mut g, bear, &[]);
    assert_eq!(g.state.players[1].life, 18);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert_eq!(g.state.players[1].life, 16);
}

fn finish_cast_choices(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets))
            })
            .unwrap();
        checked(g, action);
    }
    panic!("cast did not finish");
}
fn reject_atomic(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn pyrrhic_strike_paid_intent_requires_both_and_unpaid_requires_exactly_one() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for single in [0, 1] {
                let mut g = setup(p);
                let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                let artifact = spawn(
                    &mut g,
                    p.other(),
                    Card::from_name("Goldvein Pick").unwrap(),
                    Zone::Battlefield,
                );
                let creature = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
                let id = spawn(
                    &mut g,
                    p,
                    Card::from_name("Pyrrhic Strike").unwrap(),
                    Zone::Hand,
                );
                checked(&mut g, Action::AnnounceCast { card: id });
                let object = paid.then(|| g.object_ref(witness));
                checked(&mut g, Action::ChooseOptionalAdditionalCost { object });
                assert_eq!(g.state.pending, Pending::Modes { player: p });
                reject_atomic(
                    &mut g,
                    Action::ChooseMode {
                        mode: if paid { single } else { 2 },
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseMode {
                        mode: if paid { 2 } else { single },
                    },
                );
                if paid || single == 0 {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(artifact),
                        },
                    );
                }
                if paid || single == 1 {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(creature),
                        },
                    );
                }
                finish_cast_choices(&mut g);
                assert_eq!(
                    g.state
                        .stack
                        .last()
                        .unwrap()
                        .cast
                        .unwrap()
                        .optional_additional,
                    Some(paid)
                );
                assert_eq!(
                    g.object_view(witness).zone,
                    if paid {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                settle(&mut g);
                assert_eq!(
                    g.object_view(artifact).zone,
                    if paid || single == 0 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                assert_eq!(
                    g.object_view(creature).zone,
                    if paid || single == 1 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}
#[test]
fn pyrrhic_strike_paid_route_requires_a_completable_pair_before_declaration() {
    for only_artifact in [false, true] {
        let mut g = setup(Player::P0);
        let witness = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        let target = spawn(
            &mut g,
            Player::P1,
            if only_artifact {
                Card::from_name("Goldvein Pick").unwrap()
            } else {
                Card::HillGiant
            },
            Zone::Battlefield,
        );
        let id = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Pyrrhic Strike").unwrap(),
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceCast { card: id });
        let object = g.object_ref(witness);
        reject_atomic(
            &mut g,
            Action::ChooseOptionalAdditionalCost {
                object: Some(object),
            },
        );
        checked(
            &mut g,
            Action::ChooseOptionalAdditionalCost { object: None },
        );
        let mode = if only_artifact { 0 } else { 1 };
        reject_atomic(&mut g, Action::ChooseMode { mode: 1 - mode });
        checked(&mut g, Action::ChooseMode { mode });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}
#[test]
fn pyrrhic_strike_separate_target_occurrences_and_partial_illegality() {
    for same in [false, true] {
        let mut g = setup(Player::P0);
        let witness = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        let artifact = spawn(
            &mut g,
            Player::P1,
            Card::from_name("Juggernaut").unwrap(),
            Zone::Battlefield,
        );
        let creature = if same {
            artifact
        } else {
            spawn(&mut g, Player::P1, Card::HillGiant, Zone::Battlefield)
        };
        let id = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Pyrrhic Strike").unwrap(),
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceCast { card: id });
        let object = g.object_ref(witness);
        checked(
            &mut g,
            Action::ChooseOptionalAdditionalCost {
                object: Some(object),
            },
        );
        checked(&mut g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        finish_cast_choices(&mut g);
        if !same {
            g.move_object(artifact, Zone::Hand);
            g.move_object(artifact, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(artifact).zone,
            if same {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
#[test]
fn wild_unraveling_blight_or_mana_routes_normal_and_free_casts() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            for mana_route in [false, true] {
                let mut g = setup(p.other());
                if free {
                    spawn(
                        &mut g,
                        p,
                        Card::from_name("Omniscience").unwrap(),
                        Zone::Battlefield,
                    );
                }
                let witness =
                    (!mana_route).then(|| spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield));
                let victim = if free {
                    spell(&mut g, p.other(), "Grizzly Bears", &[])
                } else {
                    spell(&mut g, p.other(), "Lightning Bolt", &[Target::Player(p)])
                };
                checked(&mut g, Action::Pass);
                let target = Target::Spell(victim);
                let id = spawn(
                    &mut g,
                    p,
                    Card::from_name("Wild Unraveling").unwrap(),
                    Zone::Hand,
                );
                g.state.players[p.index()].mana =
                    [0, if free { 0 } else { 2 }, 0, 0, 0, u32::from(mana_route)];
                checked(
                    &mut g,
                    if free {
                        Action::AnnounceFree {
                            card: id,
                            face: CastFace::Normal,
                            flash_surcharge: false,
                        }
                    } else {
                        Action::AnnounceCast { card: id }
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseSpellCost {
                        cost: u16::from(mana_route),
                    },
                );
                reject_atomic(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Spell(id),
                    },
                );
                checked(&mut g, Action::ChooseTarget { target });
                if let Some(witness) = witness {
                    let object = g.object_ref(witness);
                    checked(&mut g, Action::ChooseAdditionalCost { object });
                }
                finish_cast_choices(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                if let Some(witness) = witness {
                    assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
                }
                settle(&mut g);
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].life, 20);
            }
        }
    }
}
#[test]
fn evershrikes_gift_paid_aura_blight_recursion_and_exact_source_binding() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let aura = spell(&mut g, p, "Evershrike's Gift", &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 3,
                    toughness: 2
                }
            );
            assert!(g.keywords(bear).contains(Keywords::FLYING));
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::AnnounceActivate {card,..} if *card == aura))
            );
            g.move_object(aura, Zone::Graveyard);
            assert_eq!(g.stats(bear).power, 2);
            assert!(!g.keywords(bear).contains(Keywords::FLYING));
            g.state.step = Step::Upkeep;
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::AnnounceActivate {card,..} if *card == aura))
            );
            g.state.step = Step::Main1;
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: aura,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(bear);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish_cast_choices(&mut g);
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
            if depart {
                g.move_object(aura, Zone::Exile);
                g.move_object(aura, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(aura).zone,
                if depart { Zone::Graveyard } else { Zone::Hand }
            );
        }
    }
}
#[test]
fn wild_unraveling_cannot_waive_additional_cost_and_uncounterable_spells_still_resolve() {
    for free in [false, true] {
        let mut g = setup(Player::P1);
        if free {
            spawn(
                &mut g,
                Player::P0,
                Card::from_name("Omniscience").unwrap(),
                Zone::Battlefield,
            );
        }
        spell(
            &mut g,
            Player::P1,
            "Lightning Bolt",
            &[Target::Player(Player::P0)],
        );
        checked(&mut g, Action::Pass);
        let id = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Wild Unraveling").unwrap(),
            Zone::Hand,
        );
        g.state.players[0].mana = [0, if free { 0 } else { 2 }, 0, 0, 0, 0];
        let action = if free {
            Action::AnnounceFree {
                card: id,
                face: CastFace::Normal,
                flash_surcharge: false,
            }
        } else {
            Action::AnnounceCast { card: id }
        };
        reject_atomic(&mut g, action);
    }
    let mut g = setup(Player::P1);
    spawn(
        &mut g,
        Player::P1,
        Card::from_name("Sphinx of the Final Word").unwrap(),
        Zone::Battlefield,
    );
    let bolt = spell(
        &mut g,
        Player::P1,
        "Lightning Bolt",
        &[Target::Player(Player::P0)],
    );
    checked(&mut g, Action::Pass);
    let id = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Wild Unraveling").unwrap(),
        Zone::Hand,
    );
    checked(&mut g, Action::AnnounceCast { card: id });
    checked(&mut g, Action::ChooseSpellCost { cost: 1 });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(bolt),
        },
    );
    finish_cast_choices(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 17);
}

fn glamer_trigger(g: &mut Game, p: Player) -> CardId {
    let source = spawn(g, p, Card::from_name("Glamer Gifter").unwrap(), Zone::Hand);
    cast(g, source, None);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    g.prepare_triggers();
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        checked(g, action);
    }
    source
}
#[test]
fn glamer_gifter_flash_flying_and_optional_other_creature_trigger() {
    let mut g = setup(Player::P1);
    g.state.priority = Player::P0;
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let source = glamer_trigger(&mut g, Player::P0);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 1,
            toughness: 2
        }
    );
    assert!(
        g.keywords(source)
            .contains(Keywords::FLASH.union(Keywords::FLYING))
    );
    reject_atomic(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    checked(&mut g, Action::FinishTargets);
    settle(&mut g);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 1,
            toughness: 2
        }
    );
    assert!(!g.has_subtype(source, "Goblin"));
    assert_eq!(g.stats(bear).power, 2);
}
#[test]
fn glamer_gifter_base_and_all_types_obey_layering_ability_removal_and_cleanup() {
    let mut g = setup(Player::P0);
    let recipient = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Juggernaut").unwrap(),
        Zone::Battlefield,
    );
    g.set_counters(recipient, Counters::of(CounterKind::PlusOne, 1));
    let source = glamer_trigger(&mut g, Player::P0);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    g.move_object(source, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(
        g.stats(recipient),
        Stats {
            power: 5,
            toughness: 5
        }
    );
    assert!(g.current_kinds(recipient) & 16 != 0);
    assert!(!g.has_subtype(recipient, "Equipment"));
    for subtype in ["Goblin", "Faerie", "Merfolk", "Coward", "Construct"] {
        assert!(g.has_subtype(recipient, subtype));
    }
    let aura = spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(recipient)],
    );
    settle(&mut g);
    assert!(g.has_subtype(recipient, "Noggle"));
    assert!(!g.has_subtype(recipient, "Faerie"));
    assert_eq!(
        g.stats(recipient),
        Stats {
            power: 2,
            toughness: 2
        }
    );
    g.move_object(aura, Zone::Graveyard);
    assert!(g.has_subtype(recipient, "Faerie"));
    spell(&mut g, Player::P0, "Stop Cold", &[Target::Card(recipient)]);
    settle(&mut g);
    assert!(!g.printed_abilities_available(recipient));
    assert!(g.has_subtype(recipient, "Faerie"));
    assert_eq!(
        g.stats(recipient),
        Stats {
            power: 5,
            toughness: 5
        }
    );
    g.finish_cleanup();
    assert!(!g.has_subtype(recipient, "Faerie"));
    assert!(g.has_subtype(recipient, "Juggernaut"));
    assert_eq!(
        g.stats(recipient),
        Stats {
            power: 6,
            toughness: 4
        }
    );
}
#[test]
fn glamer_gifter_target_incarnation_must_remain_legal_and_effect_does_not_follow_reentry() {
    for before_resolution in [false, true] {
        let mut g = setup(Player::P0);
        let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        glamer_trigger(&mut g, Player::P0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if !before_resolution {
            settle(&mut g);
            assert_eq!(g.stats(bear).power, 4);
            assert!(g.has_subtype(bear, "Faerie"));
        }
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.has_subtype(bear, "Faerie"));
    }
}
#[test]
fn wanderbrine_trapper_tap_costs_are_distinct_nontargeted_and_source_independent() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let source = cast_named(&mut g, p, "Wanderbrine Trapper");
            let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let land = spawn(&mut g, p, Card::Forest, Zone::Battlefield);
            let target = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
            let activation = Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            };
            reject_atomic(&mut g, activation.clone());
            g.state.objects[source as usize].summoning_sick = false;
            checked(&mut g, activation);
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(witness),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            for invalid in [source, land, target] {
                let object = g.object_ref(invalid);
                reject_atomic(&mut g, Action::ChooseAdditionalCost { object });
            }
            let object = g.object_ref(witness);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish_cast_choices(&mut g);
            assert!(g.object_view(source).tapped);
            assert!(g.object_view(witness).tapped);
            if depart {
                g.move_object(source, Zone::Graveyard);
            }
            settle(&mut g);
            assert!(g.object_view(target).tapped);
        }
    }
}
#[test]
fn wanderbrine_trapper_tapped_payment_is_illegal_and_invalid_target_does_not_refund_costs() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Wanderbrine Trapper");
    g.state.objects[source as usize].summoning_sick = false;
    let witness = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let target = spawn(&mut g, Player::P1, Card::HillGiant, Zone::Battlefield);
    g.tap_objects(&[witness]);
    let activation = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject_atomic(&mut g, activation.clone());
    g.untap_objects(&[witness], untap_rules::UntapCause::Effect);
    checked(&mut g, activation);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let object = g.object_ref(witness);
    checked(&mut g, Action::ChooseAdditionalCost { object });
    finish_cast_choices(&mut g);
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert!(!g.object_view(target).tapped);
    assert!(g.object_view(source).tapped);
    assert!(g.object_view(witness).tapped);
}

fn kithkin_tokens(g: &Game, p: Player) -> Vec<CardId> {
    let token = Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == token)
                .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn kithkeeper_vivid_counts_current_owned_permanent_colors_once_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for colored in [false, true] {
            let mut g = setup(p);
            spawn(
                &mut g,
                p.other(),
                Card::from_name("Vampire Nighthawk").unwrap(),
                Zone::Battlefield,
            );
            if colored {
                spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                spawn(
                    &mut g,
                    p,
                    Card::from_name("Omniscience").unwrap(),
                    Zone::Battlefield,
                );
            }
            cast_named(&mut g, p, "Kithkeeper");
            assert_eq!(kithkin_tokens(&g, p).len(), if colored { 3 } else { 1 });
            for token in kithkin_tokens(&g, p) {
                assert_eq!(
                    g.stats(token),
                    Stats {
                        power: 1,
                        toughness: 1
                    }
                );
                assert!(g.has_subtype(token, "Kithkin"));
                assert_eq!(g.object_view(token).colors.count_ones(), 2);
            }
        }
    }
    let mut g = setup(Player::P0);
    let id = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Kithkeeper").unwrap(),
        Zone::Hand,
    );
    cast(&mut g, id, None);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    g.move_object(id, Zone::Graveyard);
    settle(&mut g);
    assert!(kithkin_tokens(&g, Player::P0).is_empty());
}
#[test]
fn kithkeeper_can_tap_itself_while_sick_or_three_other_creatures_and_binding_is_exact() {
    for include_source in [false, true] {
        for depart in [false, true] {
            let mut g = setup(Player::P0);
            let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
            let source = cast_named(&mut g, Player::P0, "Kithkeeper");
            let tokens = kithkin_tokens(&g, Player::P0);
            assert_eq!(tokens.len(), 2);
            assert!(g.object_view(source).summoning_sick);
            g.state.active = Player::P1;
            g.state.priority = Player::P0;
            g.state.step = Step::Upkeep;
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let first = if include_source { source } else { bear };
            let object = g.object_ref(first);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            reject_atomic(&mut g, Action::ChooseAdditionalCost { object });
            for &id in &tokens {
                let object = g.object_ref(id);
                checked(&mut g, Action::ChooseAdditionalCost { object });
            }
            finish_cast_choices(&mut g);
            assert!(g.object_view(first).tapped);
            assert!(tokens.iter().all(|&id| g.object_view(id).tapped));
            assert_eq!(g.object_view(source).tapped, include_source);
            if depart {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.stats(source).power, if depart { 3 } else { 6 });
            assert_eq!(g.keywords(source).contains(Keywords::FLYING), !depart);
            g.finish_cleanup();
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert!(!g.keywords(source).contains(Keywords::FLYING));
        }
    }
}
#[test]
fn kithkeeper_requires_three_distinct_untapped_creatures_and_can_repeat_after_untapping() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Kithkeeper");
    let activation = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject_atomic(&mut g, activation.clone());
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let token = kithkin_tokens(&g, Player::P0)[0];
    for round in 1..=2 {
        checked(&mut g, activation.clone());
        for id in [source, bear, token] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 3 + round * 3);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        reject_atomic(&mut g, activation.clone());
        g.untap_objects(&[source, bear, token], untap_rules::UntapCause::Effect);
    }
}

#[test]
fn blossombind_paid_aura_preserves_existing_counters_blocks_untap_and_ordinary_counter_effects() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.set_counters(
            bear,
            Counters::of(CounterKind::PlusOne, 1).with(CounterKind::Stun, 1),
        );
        let aura = spell(&mut g, p, "Blossombind", &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.object_view(bear).tapped);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(bear, CounterKind::Stun), 1);
        assert!(!g.can_receive_counters(bear));
        spell(&mut g, p, "Snakeskin Veil", &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
        spell(&mut g, p, "Shore Up", &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.object_view(bear).tapped);
        assert_eq!(g.counter_count(bear, CounterKind::Stun), 1);
        g.untap_objects(&[bear], untap_rules::UntapCause::ControllerStep(p));
        assert!(g.object_view(bear).tapped);
        assert_eq!(g.counter_count(bear, CounterKind::Stun), 1);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.can_receive_counters(bear));
        g.untap_objects(&[bear], untap_rules::UntapCause::Effect);
        assert!(g.object_view(bear).tapped);
        assert_eq!(g.counter_count(bear, CounterKind::Stun), 0);
        g.untap_objects(&[bear], untap_rules::UntapCause::Effect);
        assert!(!g.object_view(bear).tapped);
    }
}
#[test]
fn blossombind_enter_trigger_uses_departure_attachment_and_exact_recipient_incarnation() {
    for reenter in [false, true] {
        let mut g = setup(Player::P0);
        let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        let aura = spell(&mut g, Player::P0, "Blossombind", &[Target::Card(bear)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(!g.object_view(bear).tapped);
        g.move_object(aura, Zone::Graveyard);
        if reenter {
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(g.object_view(bear).tapped, !reenter);
        assert!(g.can_receive_counters(bear));
    }
}
#[test]
fn blossombind_prevents_optional_and_mandatory_blight_costs_without_removing_counter_costs() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(bear)]);
    settle(&mut g);
    let enemy = spawn(&mut g, Player::P1, Card::HillGiant, Zone::Battlefield);
    let id = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Cinder Strike").unwrap(),
        Zone::Hand,
    );
    checked(&mut g, Action::AnnounceCast { card: id });
    let object = g.object_ref(bear);
    reject_atomic(
        &mut g,
        Action::ChooseOptionalAdditionalCost {
            object: Some(object),
        },
    );
    checked(
        &mut g,
        Action::ChooseOptionalAdditionalCost { object: None },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    finish_cast_choices(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(enemy).damage, 2);
    let source = mare(&mut g, Player::P0);
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(source)]);
    settle(&mut g);
    activate(&mut g, source, 0, None);
    settle(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::MinusOne), 2);
    assert!(!g.can_receive_counters(source));
}
#[test]
fn blossombind_blink_ends_old_prohibition_and_entry_counters_are_placed_normally() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    let aura = spell(&mut g, Player::P0, "Blossombind", &[Target::Card(source)]);
    settle(&mut g);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(source)]);
    settle(&mut g);
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(source, CounterKind::MinusOne), 3);
    assert!(g.can_receive_counters(source));
    assert!(!g.object_view(source).tapped);
}
#[test]
fn blossombind_wither_deals_damage_and_gains_life_without_marks_or_counters() {
    let mut g = setup(Player::P0);
    let source = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Healer's Hawk").unwrap(),
        Zone::Battlefield,
    );
    let bear = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(bear)]);
    settle(&mut g);
    let equipment = cast_named(&mut g, Player::P0, "Barbed Bloodletter");
    assert_eq!(
        g.object_view(equipment).attached_to,
        Some(g.object_ref(source))
    );
    assert!(
        g.keywords(source)
            .contains(Keywords::WITHER.union(Keywords::LIFELINK))
    );
    spell(
        &mut g,
        Player::P0,
        "Rabid Bite",
        &[Target::Card(source), Target::Card(bear)],
    );
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(g.object_view(bear).damage, 0);
    assert_eq!(g.counter_count(bear, CounterKind::MinusOne), 0);
    assert!(g.object_view(bear).damaged_this_turn);
}

fn animate_counter_rule_fixture(g: &mut Game, id: CardId) {
    let p = g.object_view(id).controller;
    let item = g.activation_item(p, id, 0);
    g.animate_source(
        &item,
        crate::cards::AnimationSpec {
            artifact: false,
            creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            duration: crate::cards::AnimationDuration::Permanent,
        },
    );
    g.state.objects[id as usize].summoning_sick = false;
}
#[test]
fn blossombind_counter_cost_is_forbidden_even_before_its_tap_trigger_resolves() {
    let mut g = setup(Player::P0);
    let tome = cast_named(&mut g, Player::P0, "Mazemind Tome");
    animate_counter_rule_fixture(&mut g, tome);
    let activation = Action::AnnounceActivate {
        card: tome,
        ability: 0,
        counter_payment: Counters::default(),
    };
    assert!(g.legal_actions().contains(&activation));
    let aura = spell(&mut g, Player::P0, "Blossombind", &[Target::Card(tome)]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(!g.object_view(tome).tapped);
    reject_atomic(&mut g, activation.clone());
    assert_eq!(g.counter_count(tome, CounterKind::Page), 0);
    spell(&mut g, Player::P0, "Disenchant", &[Target::Card(aura)]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(g.can_receive_counters(tome));
    assert!(!g.object_view(tome).tapped);
    checked(&mut g, activation);
    finish_cast_choices(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(tome, CounterKind::Page), 1);
    assert!(g.object_view(tome).tapped);
}
#[test]
fn blossombind_positive_loyalty_cannot_be_paid_but_negative_loyalty_can() {
    let mut g = setup(Player::P0);
    let walker = cast_named(&mut g, Player::P0, "Vivien Reid");
    animate_counter_rule_fixture(&mut g, walker);
    let aura = spell(&mut g, Player::P0, "Blossombind", &[Target::Card(walker)]);
    settle(&mut g);
    reject_atomic(
        &mut g,
        Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: walker,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    finish_cast_choices(&mut g);
    assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 2);
    settle(&mut g);
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    assert!(g.can_receive_counters(walker));
}
#[test]
fn blossombind_blocks_blight_payment_routes_and_optional_resolution_payment() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(bear)]);
    settle(&mut g);
    cast_named(&mut g, Player::P0, "Boggart Mischief");
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.controller == Player::P0
                && o.card.is_creature())
            .count(),
        1
    );
    assert_eq!(g.counter_count(bear, CounterKind::MinusOne), 0);
    g.state.priority = Player::P1;
    let bolt = spell(
        &mut g,
        Player::P1,
        "Lightning Bolt",
        &[Target::Player(Player::P0)],
    );
    checked(&mut g, Action::Pass);
    let id = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Wild Unraveling").unwrap(),
        Zone::Hand,
    );
    g.state.players[0].mana = [0, 2, 0, 0, 0, 1];
    checked(&mut g, Action::AnnounceCast { card: id });
    reject_atomic(&mut g, Action::ChooseSpellCost { cost: 0 });
    checked(&mut g, Action::ChooseSpellCost { cost: 1 });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(bolt),
        },
    );
    finish_cast_choices(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.counter_count(bear, CounterKind::MinusOne), 0);
}
#[test]
fn blossombind_selected_land_untap_effect_honors_prohibition_before_stun() {
    let mut g = setup(Player::P0);
    let land = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Soulstone Sanctuary").unwrap(),
        Zone::Battlefield,
    );
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: land,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    finish_cast_choices(&mut g);
    settle(&mut g);
    g.set_counters(land, Counters::of(CounterKind::Stun, 1));
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(land)]);
    settle(&mut g);
    let other = spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
    g.tap_objects(&[other]);
    g.set_counters(other, Counters::of(CounterKind::Stun, 1));
    for _ in 0..12 {
        spawn(&mut g, Player::P0, Card::Forest, Zone::Library);
    }
    let id = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Finale of Revelation").unwrap(),
        Zone::Hand,
    );
    g.state.players[Player::P0.index()].mana = [0, 12, 0, 0, 0, 0];
    checked(&mut g, Action::AnnounceCast { card: id });
    checked(&mut g, Action::ChooseX { value: 10 });
    finish_cast_choices(&mut g);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
    for id in [land, other] {
        let object = g.object_ref(id);
        checked(&mut g, Action::SelectLand { object });
    }
    checked(&mut g, Action::FinishLandSelection);
    settle(&mut g);
    assert!(g.object_view(land).tapped);
    assert_eq!(g.counter_count(land, CounterKind::Stun), 1);
    assert!(g.object_view(other).tapped);
    assert_eq!(g.counter_count(other, CounterKind::Stun), 0);
}

#[test]
fn blossombind_keeps_applicable_counter_replacement_choices_but_commits_no_counters() {
    let mut g = setup(Player::P0);
    let bear = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Battlefield,
    );
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(bear)]);
    settle(&mut g);
    let doubling = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Doubling Season").unwrap(),
        Zone::Battlefield,
    );
    spawn(
        &mut g,
        Player::P0,
        Card::from_name("Michelangelo, Weirdness to 11").unwrap(),
        Zone::Battlefield,
    );
    g.effect_counter(
        bear,
        CounterKind::PlusOne,
        1,
        replacements::ResumeMode::Pending,
    );
    assert!(g.advance_replacements(false));
    let source = g.object_ref(doubling);
    checked(&mut g, Action::ChooseReplacement { source, ability: 1 });
    settle(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
}

fn crescendo_type_choice(g: &mut Game) {
    finish_cast_choices(g);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(
        g.state.pending,
        Pending::ResolutionCreatureType { .. }
    ));
}

#[test]
fn harmonized_crescendo_chooses_on_resolution_and_counts_current_permanents_including_kindred_and_changeling()
 {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            spawn(
                &mut g,
                p,
                Card::from_name("Llanowar Elves").unwrap(),
                Zone::Battlefield,
            );
            let changeling = spawn(
                &mut g,
                p,
                Card::from_name("Prideful Feastling").unwrap(),
                Zone::Battlefield,
            );
            let kindred = spawn(
                &mut g,
                p,
                Card::from_name("Morcant's Eyes").unwrap(),
                Zone::Battlefield,
            );
            assert!(!g.is_creature(kindred));
            spawn(
                &mut g,
                p.other(),
                Card::from_name("Llanowar Elves").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                p,
                Card::from_name("Llanowar Elves").unwrap(),
                Zone::Hand,
            );
            for _ in 0..6 {
                spawn(&mut g, p, Card::Island, Zone::Library);
            }
            let spell = spawn(
                &mut g,
                p,
                Card::from_name("Harmonized Crescendo").unwrap(),
                Zone::Hand,
            );
            checked(&mut g, Action::AnnounceCast { card: spell });
            let creature_type = crate::cards::CreatureType::from_name("Elf").unwrap();
            reject_atomic(&mut g, Action::ChooseCreatureType { creature_type });
            assert!(g.observe(p).resolution_creature_type.is_none());
            finish_cast_choices(&mut g);
            if departed {
                g.move_object(changeling, Zone::Hand);
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert!(matches!(
                g.state.pending,
                Pending::ResolutionCreatureType { .. }
            ));
            assert_eq!(
                g.legal_actions()
                    .iter()
                    .filter(|action| matches!(action, Action::ChooseCreatureType { .. }))
                    .count(),
                crate::CREATURE_TYPES.len()
            );
            let hand = g
                .state
                .objects
                .iter()
                .filter(|object| object.zone == Zone::Hand && object.owner == p)
                .count();
            checked(&mut g, Action::ChooseCreatureType { creature_type });
            settle(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|object| object.zone == Zone::Hand && object.owner == p)
                    .count(),
                hand + if departed { 2 } else { 3 }
            );
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
            assert!(g.resolution_chosen_creature_type().is_none());
        }
    }
}

#[test]
fn harmonized_crescendo_convoke_pays_colors_and_generic_with_distinct_sick_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let blue = spawn(
            &mut g,
            p,
            Card::from_name("Brineborn Cutthroat").unwrap(),
            Zone::Battlefield,
        );
        let second = spawn(
            &mut g,
            p,
            Card::from_name("Brineborn Cutthroat").unwrap(),
            Zone::Battlefield,
        );
        let bear = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        for _ in 0..4 {
            spawn(&mut g, p, Card::Island, Zone::Library);
        }
        let spell = spawn(
            &mut g,
            p,
            Card::from_name("Harmonized Crescendo").unwrap(),
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::BeginConvoke);
        let wrong = g.object_ref(bear);
        reject_atomic(
            &mut g,
            Action::ChooseConvoker {
                object: wrong,
                contribution: ConvokeContribution::Blue,
            },
        );
        for (id, contribution) in [
            (blue, ConvokeContribution::Blue),
            (second, ConvokeContribution::Blue),
            (bear, ConvokeContribution::Generic),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
            if id == blue {
                reject_atomic(
                    &mut g,
                    Action::ChooseConvoker {
                        object,
                        contribution,
                    },
                );
            }
        }
        crescendo_type_choice(&mut g);
        assert!(
            g.object_view(blue).tapped
                && g.object_view(second).tapped
                && g.object_view(bear).tapped
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let creature_type = crate::cards::CreatureType::from_name("Merfolk").unwrap();
        checked(&mut g, Action::ChooseCreatureType { creature_type });
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|object| object.zone == Zone::Hand && object.owner == p)
                .count(),
            2
        );
    }
}

#[test]
fn harmonized_crescendo_zero_matching_permanents_draws_zero_without_library_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = spawn(
            &mut g,
            p,
            Card::from_name("Harmonized Crescendo").unwrap(),
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceCast { card: spell });
        crescendo_type_choice(&mut g);
        let creature_type = crate::cards::CreatureType::from_name("Time Lord").unwrap();
        let hand = g
            .state
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Hand && object.owner == p)
            .count();
        checked(&mut g, Action::ChooseCreatureType { creature_type });
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|object| object.zone == Zone::Hand && object.owner == p)
                .count(),
            hand
        );
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert!(g.outcome().is_none());
    }
}

fn thoughtweft_attack(g: &mut Game, attackers: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in attackers {
        g.state.objects[card as usize].summoning_sick = false;
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}

#[test]
fn thoughtweft_imbuer_paid_body_can_buff_itself_and_only_single_declarations_trigger() {
    for p in [Player::P0, Player::P1] {
        for alone in [false, true] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Thoughtweft Imbuer").unwrap(),
                Zone::Hand,
            );
            cast(&mut g, source, None);
            settle(&mut g);
            let bear = spawn(
                &mut g,
                p,
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                p.other(),
                Card::from_name("Reluctant Dounguard").unwrap(),
                Zone::Battlefield,
            );
            let attackers = if alone {
                vec![source]
            } else {
                vec![source, bear]
            };
            thoughtweft_attack(&mut g, &attackers);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|action| matches!(action, Action::ChooseTarget { .. }))
            );
            settle(&mut g);
            assert_eq!(
                g.stats(source),
                Stats {
                    power: if alone { 1 } else { 0 },
                    toughness: if alone { 6 } else { 5 }
                }
            );
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            g.finish_cleanup();
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 0,
                    toughness: 5
                }
            );
        }
    }
}

#[test]
fn thoughtweft_imbuer_resolution_count_includes_kindred_changeling_and_survives_source_or_recipient_control_changes()
 {
    for p in [Player::P0, Player::P1] {
        for changed in [0, 1, 2] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Thoughtweft Imbuer").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                p,
                Card::from_name("Clachan Festival").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                p,
                Card::from_name("Prideful Feastling").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                p.other(),
                Card::from_name("Reluctant Dounguard").unwrap(),
                Zone::Battlefield,
            );
            let bear = spawn(
                &mut g,
                p,
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            thoughtweft_attack(&mut g, &[bear]);
            match changed {
                0 => {
                    g.move_object(source, Zone::Graveyard);
                }
                1 => {
                    let object = g.object_ref(source);
                    g.change_controller(object, p.other());
                }
                _ => {
                    let object = g.object_ref(bear);
                    g.change_controller(object, p.other());
                }
            }
            settle(&mut g);
            let amount = if changed == 2 { 3 } else { 2 };
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 2 + amount,
                    toughness: 2 + amount
                }
            );
            spawn(
                &mut g,
                p,
                Card::from_name("Reluctant Dounguard").unwrap(),
                Zone::Battlefield,
            );
            assert_eq!(g.stats(bear).power, 2 + amount);
        }
    }
}

#[test]
fn thoughtweft_imbuer_exact_attacker_binding_ignores_departed_incarnations_and_later_attackers() {
    for reentered in [false, true] {
        let mut g = setup(Player::P0);
        spawn(
            &mut g,
            Player::P0,
            Card::from_name("Thoughtweft Imbuer").unwrap(),
            Zone::Battlefield,
        );
        let bear = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        thoughtweft_attack(&mut g, &[bear]);
        if reentered {
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
        } else {
            g.state.attackers.clear();
        }
        let later = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        settle(&mut g);
        assert_eq!(g.stats(bear).power, if reentered { 2 } else { 3 });
        assert_eq!(g.stats(later).power, 2);
    }
}

#[test]
fn thoughtweft_imbuer_multiple_observers_stack_and_opposing_declarations_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = spawn(
            &mut g,
            p,
            Card::from_name("Thoughtweft Imbuer").unwrap(),
            Zone::Battlefield,
        );
        spawn(
            &mut g,
            p,
            Card::from_name("Thoughtweft Imbuer").unwrap(),
            Zone::Battlefield,
        );
        thoughtweft_attack(&mut g, &[first]);
        settle(&mut g);
        assert_eq!(
            g.stats(first),
            Stats {
                power: 4,
                toughness: 9
            }
        );
        let mut g = setup(p.other());
        spawn(
            &mut g,
            p,
            Card::from_name("Thoughtweft Imbuer").unwrap(),
            Zone::Battlefield,
        );
        let enemy = spawn(
            &mut g,
            p.other(),
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        thoughtweft_attack(&mut g, &[enemy]);
        settle(&mut g);
        assert_eq!(g.stats(enemy).power, 2);
    }
}

#[test]
fn kinbinding_counts_entries_before_it_arrives_and_keeps_departure_history_until_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p, Card::from_name("Bear Cub").unwrap(), Zone::Hand);
        cast(&mut g, bear, None);
        settle(&mut g);
        let kin = spawn(
            &mut g,
            p,
            Card::from_name("Kinbinding").unwrap(),
            Zone::Hand,
        );
        cast(&mut g, kin, None);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        let second = spawn(&mut g, p, Card::from_name("Bear Cub").unwrap(), Zone::Hand);
        cast(&mut g, second, None);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 4);
        assert_eq!(g.stats(second).power, 4);
        g.move_object(bear, Zone::Hand);
        assert_eq!(g.stats(second).power, 4);
        let object = g.object_ref(second);
        g.change_controller(object, p.other());
        assert_eq!(g.stats(second).power, 2);
        let object = g.object_ref(kin);
        g.change_controller(object, p.other());
        assert_eq!(g.stats(second).power, 2);
        let object = g.object_ref(kin);
        g.change_controller(object, p);
        let object = g.object_ref(second);
        g.change_controller(object, p);
        assert_eq!(g.stats(second).power, 4);
        g.begin_turn();
        assert_eq!(g.stats(second).power, 2);
    }
}

#[test]
fn kinbinding_begin_combat_token_is_green_white_kithkin_and_its_entry_immediately_increases_bonus()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        let kin = spawn(
            &mut g,
            p,
            Card::from_name("Kinbinding").unwrap(),
            Zone::Hand,
        );
        cast(&mut g, kin, None);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 2);
        g.begin_step(Step::BeginCombat);
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.zone == Zone::Battlefield && object.controller == p && object.is_token()
            })
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(
            g.object_view(token).card.identity(),
            "@token/c48e9e904360e4cee1deeed8"
        );
        assert_eq!(g.object_view(token).card.colors(), 17);
        assert!(g.has_subtype(token, "Kithkin"));
        assert_eq!(
            g.stats(token),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(g.stats(bear).power, 3);
        g.move_object(token, Zone::Graveyard);
        assert_eq!(g.stats(bear).power, 3);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.begin_step(Step::BeginCombat);
        settle(&mut g);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|object| object.zone == Zone::Battlefield && object.is_token())
        );
    }
}

#[test]
fn kinbinding_multiple_sources_stack_and_source_ability_loss_stops_its_bonus() {
    let mut g = setup(Player::P0);
    let bear = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Hand,
    );
    cast(&mut g, bear, None);
    settle(&mut g);
    let first = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Kinbinding").unwrap(),
        Zone::Hand,
    );
    cast(&mut g, first, None);
    settle(&mut g);
    let second = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Kinbinding").unwrap(),
        Zone::Hand,
    );
    cast(&mut g, second, None);
    settle(&mut g);
    assert_eq!(g.stats(bear).power, 4);
    g.move_object(first, Zone::Hand);
    assert_eq!(g.stats(bear).power, 3);
    animate_counter_rule_fixture(&mut g, second);
    spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(second)],
    );
    settle(&mut g);
    assert_eq!(g.stats(bear).power, 2);
}

fn flitterwing(g: &mut Game, p: Player) -> CardId {
    let source = spawn(
        g,
        p,
        Card::from_name("Flitterwing Nuisance").unwrap(),
        Zone::Hand,
    );
    cast(g, source, None);
    settle(g);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert!(g.keywords(source).contains(Keywords::FLYING));
    source
}
fn hand_count(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|object| object.zone == Zone::Hand && object.owner == p)
        .count()
}

#[test]
fn flitterwing_nuisance_paid_counter_removal_registers_only_on_resolution_and_each_creature_draws()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = flitterwing(&mut g, p);
        let bear = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        for _ in 0..5 {
            spawn(&mut g, p, Card::Island, Zone::Library);
        }
        activate(&mut g, source, 0, None);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        assert!(g.observe(p).delayed_combat_draws.is_empty());
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_combat_draws.len(), 1);
        assert_eq!(
            g.observe(p).delayed_combat_draws,
            g.observe(p.other()).delayed_combat_draws
        );
        assert!(!g.legal_actions().iter().any(|action| matches!(action, Action::AnnounceActivate { card, .. } | Action::Activate { card, .. } | Action::BeginActivate { card, .. } if *card == source)));
        let hand = hand_count(&g, p);
        thoughtweft_attack(&mut g, &[source, bear]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        settle(&mut branch);
        assert_eq!(hand_count(&branch, p), hand + 2);
        assert_eq!(branch.observe(p).delayed_combat_draws.len(), 1);
        branch.finish_cleanup();
        assert!(branch.observe(p).delayed_combat_draws.is_empty());
    }
}

#[test]
fn flitterwing_nuisance_delayed_observer_survives_source_departure_and_draws_for_lethal_planeswalker_damage()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = flitterwing(&mut g, p);
        activate(&mut g, source, 0, None);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        let bear = spawn(
            &mut branch,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        let walker = spawn(
            &mut branch,
            p.other(),
            Card::from_name("Vivien Reid").unwrap(),
            Zone::Battlefield,
        );
        branch.set_counters(walker, Counters::of(CounterKind::Loyalty, 1));
        for _ in 0..3 {
            spawn(&mut branch, p, Card::Island, Zone::Library);
        }
        let hand = hand_count(&branch, p);
        branch.state.objects[bear as usize].summoning_sick = false;
        branch.state.step = Step::Attackers;
        branch.state.pending = Pending::Attackers;
        let defender = branch.object_ref(walker);
        checked(
            &mut branch,
            Action::AttackPlaneswalker {
                card: bear,
                defender,
            },
        );
        checked(&mut branch, Action::FinishAttackers);
        checked(&mut branch, Action::Pass);
        checked(&mut branch, Action::Pass);
        checked(&mut branch, Action::FinishBlockers);
        checked(&mut branch, Action::Pass);
        checked(&mut branch, Action::Pass);
        assert_eq!(branch.object_view(walker).zone, Zone::Graveyard);
        settle(&mut branch);
        assert_eq!(hand_count(&branch, p), hand + 1);
    }
}

#[test]
fn flitterwing_nuisance_multiple_registrations_any_named_counter_and_current_damage_controller_are_independent()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = flitterwing(&mut g, p);
        g.add_counter(source, CounterKind::Stun, 1);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::of(CounterKind::Stun, 1),
            },
        );
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 1);
        assert_eq!(g.counter_count(source, CounterKind::Stun), 0);
        activate(&mut g, source, 0, None);
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_combat_draws.len(), 2);
        let bear = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        let enemy = spawn(
            &mut g,
            p.other(),
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        for _ in 0..8 {
            spawn(&mut g, p, Card::Island, Zone::Library);
        }
        let hand = hand_count(&g, p);
        let source_ref = g.object_ref(source);
        g.change_controller(source_ref, p.other());
        g.state.active = p.other();
        g.state.priority = p.other();
        let events = [
            DamageEvent {
                preventable: true,
                source: g.damage_source(bear),
                target: Target::Player(p.other()),
                amount: 1,
                combat: true,
            },
            DamageEvent {
                preventable: true,
                source: g.damage_source(enemy),
                target: Target::Player(p),
                amount: 1,
                combat: true,
            },
            DamageEvent {
                preventable: true,
                source: g.damage_source(bear),
                target: Target::Card(enemy),
                amount: 1,
                combat: true,
            },
            DamageEvent {
                preventable: true,
                source: g.damage_source(bear),
                target: Target::Player(p.other()),
                amount: 1,
                combat: false,
            },
            DamageEvent {
                preventable: true,
                source: g.damage_source(bear),
                target: Target::Player(p.other()),
                amount: 0,
                combat: true,
            },
        ];
        g.apply_damage_batch(&events);
        settle(&mut g);
        assert_eq!(hand_count(&g, p), hand + 2);
        let bear_ref = g.object_ref(bear);
        g.change_controller(bear_ref, p.other());
        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(bear),
            target: Target::Player(p),
            amount: 1,
            combat: true,
        };
        g.apply_damage_batch(&[event]);
        settle(&mut g);
        assert_eq!(hand_count(&g, p), hand + 2);
        g.finish_cleanup();
        let bear_ref = g.object_ref(bear);
        g.change_controller(bear_ref, p);
        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(bear),
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
        };
        g.apply_damage_batch(&[event]);
        settle(&mut g);
        assert_eq!(hand_count(&g, p), hand + 2);
    }
}

#[test]
fn flitterwing_nuisance_countered_activation_registers_nothing_and_repeated_batches_and_recipients_trigger_separately()
 {
    for countered in [false, true] {
        let mut g = setup(Player::P0);
        let source = flitterwing(&mut g, Player::P0);
        let bear = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        let walker = spawn(
            &mut g,
            Player::P1,
            Card::from_name("Vivien Reid").unwrap(),
            Zone::Battlefield,
        );
        g.set_counters(walker, Counters::of(CounterKind::Loyalty, 5));
        for _ in 0..8 {
            spawn(&mut g, Player::P0, Card::Island, Zone::Library);
        }
        let hand = hand_count(&g, Player::P0);
        activate(&mut g, source, 0, None);
        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(bear),
            target: Target::Player(Player::P1),
            amount: 1,
            combat: true,
        };
        // Activation is still on the stack, so this earlier damage is unobserved.
        g.apply_damage_batch(&[event]);
        assert!(g.state.pending_triggers.is_empty());
        if countered {
            let ability = g.state.stack.last().unwrap().id;
            assert!(g.counter_stack_item(ability, false));
        }
        settle(&mut g);
        assert_eq!(
            g.observe(Player::P0).delayed_combat_draws.len(),
            usize::from(!countered)
        );
        let hits = [
            event,
            DamageEvent {
                target: Target::Card(walker),
                ..event
            },
        ];
        g.apply_damage_batch(&hits);
        settle(&mut g);
        assert_eq!(
            hand_count(&g, Player::P0),
            hand + if countered { 0 } else { 2 }
        );
        g.apply_damage_batch(&[event]);
        settle(&mut g);
        assert_eq!(
            hand_count(&g, Player::P0),
            hand + if countered { 0 } else { 3 }
        );
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
    }
}

fn rimefire(g: &mut Game, p: Player, name: &str) -> CardId {
    let source = spawn(
        g,
        p,
        Card::from_name("Rimefire Torque").unwrap(),
        Zone::Hand,
    );
    checked(g, Action::AnnounceCast { card: source });
    finish_cast_choices(g);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    let kind = crate::cards::CreatureType::from_name(name).unwrap();
    checked(
        g,
        Action::ChooseEntryValue {
            value: crate::EntryChoiceValue::CreatureType(kind),
        },
    );
    settle(g);
    source
}

#[test]
fn rimefire_torque_paid_entry_choice_counts_each_owned_matching_permanent_including_kindred_and_changeling()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = rimefire(&mut g, p, "Elf");
        assert_eq!(g.counter_count(source, CounterKind::Charge), 0);
        for name in ["Llanowar Elves", "Morcant's Eyes", "Prideful Feastling"] {
            let id = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
            cast(&mut g, id, None);
            settle(&mut g);
        }
        assert_eq!(g.counter_count(source, CounterKind::Charge), 3);
        let other = spawn(
            &mut g,
            p.other(),
            Card::from_name("Llanowar Elves").unwrap(),
            Zone::Hand,
        );
        g.move_object(other, Zone::Battlefield);
        settle(&mut g);
        let bear = spawn(&mut g, p, Card::from_name("Bear Cub").unwrap(), Zone::Hand);
        cast(&mut g, bear, None);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Charge), 3);
        assert_eq!(
            g.observe(p)
                .objects
                .iter()
                .find(|object| object.reference.card == source)
                .unwrap()
                .counters
                .get(CounterKind::Charge),
            3
        );
    }
}

#[test]
fn rimefire_torque_charge_cost_taps_exact_source_and_delayed_copy_survives_departure_and_retargets_once()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = rimefire(&mut g, p, "Elf");
        g.set_counters(source, Counters::of(CounterKind::Page, 3));
        reject_atomic(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::of(CounterKind::Page, 3),
            },
        );
        g.set_counters(source, Counters::of(CounterKind::Charge, 3));
        activate(&mut g, source, 0, None);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.counter_count(source, CounterKind::Charge), 0);
        assert!(g.observe(p).delayed_spell_copies.is_empty());
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_spell_copies.len(), 1);
        g.move_object(source, Zone::Graveyard);
        let bear = spawn(&mut g, p, Card::from_name("Bear Cub").unwrap(), Zone::Hand);
        cast(&mut g, bear, None);
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_spell_copies.len(), 1);
        let before = g.state.players[p.index()].spells_cast_this_turn;
        spell(&mut g, p, "Shock", &[Target::Player(p.other())]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(p),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        settle(&mut branch);
        assert_eq!(branch.state.players[p.index()].life, 18);
        assert_eq!(branch.state.players[p.other().index()].life, 18);
        assert_eq!(
            branch.state.players[p.index()].spells_cast_this_turn,
            before + 1
        );
        assert!(branch.observe(p).delayed_spell_copies.is_empty());
        spell(&mut branch, p, "Shock", &[Target::Player(p.other())]);
        settle(&mut branch);
        assert_eq!(branch.state.players[p.other().index()].life, 16);
    }
}

#[test]
fn rimefire_torque_multiple_delayed_activations_stack_and_unused_observers_expire() {
    let mut g = setup(Player::P0);
    let source = rimefire(&mut g, Player::P0, "Time Lord");
    for _ in 0..2 {
        g.set_counters(source, Counters::of(CounterKind::Charge, 3));
        g.state.objects[source as usize].tapped = false;
        activate(&mut g, source, 0, None);
        settle(&mut g);
    }
    assert_eq!(g.observe(Player::P0).delayed_spell_copies.len(), 2);
    for _ in 0..8 {
        spawn(&mut g, Player::P0, Card::Island, Zone::Library);
    }
    let before = hand_count(&g, Player::P0);
    spell(&mut g, Player::P0, "Divination", &[]);
    settle(&mut g);
    assert_eq!(hand_count(&g, Player::P0), before + 6);
    assert!(g.observe(Player::P0).delayed_spell_copies.is_empty());
    g.set_counters(source, Counters::of(CounterKind::Charge, 3));
    g.state.objects[source as usize].tapped = false;
    activate(&mut g, source, 0, None);
    settle(&mut g);
    assert_eq!(g.observe(Player::P0).delayed_spell_copies.len(), 1);
    g.finish_cleanup();
    assert!(g.observe(Player::P0).delayed_spell_copies.is_empty());
}

fn wait_library_owner(g: &mut Game) {
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::TargetLibrary { .. }));
}
fn library_destination(g: &mut Game, victim: CardId, destination: LibraryDestination) {
    let object = g.object_ref(victim);
    checked(
        g,
        Action::ChooseLibraryDestination {
            object,
            destination,
        },
    );
    settle(g);
}

#[test]
fn temporal_cleansing_foreign_controller_owner_choice_second_or_bottom_and_short_libraries_are_exact()
 {
    for p in [Player::P0, Player::P1] {
        for size in [0, 1, 3] {
            for destination in [
                LibraryDestination::SecondFromTop,
                LibraryDestination::Bottom,
            ] {
                let mut g = setup(p);
                let owner = p.other();
                let old = g.state.players[owner.index()].library.clone();
                for id in old {
                    g.move_object(id, Zone::Graveyard);
                }
                for _ in 0..size {
                    spawn(&mut g, owner, Card::Island, Zone::Library);
                }
                let before = g.state.players[owner.index()].library.clone();
                let old_top = before.last().map(|&id| g.object_ref(id));
                if let Some(top) = old_top {
                    g.remember_library_top(owner, top);
                }
                let victim = spawn(
                    &mut g,
                    owner,
                    Card::from_name("Bear Cub").unwrap(),
                    Zone::Battlefield,
                );
                let object = g.object_ref(victim);
                g.change_controller(object, p);
                spell(&mut g, p, "Temporal Cleansing", &[Target::Card(victim)]);
                wait_library_owner(&mut g);
                assert_eq!(g.actor(), Some(owner));
                assert!(g.observe(owner).target_library.is_some());
                assert!(g.observe(p).target_library.is_none());
                reject_atomic(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination: LibraryDestination::Top,
                    },
                );
                let mut branch = {
                    let parent = g.clone();
                    parent.clone()
                };
                library_destination(&mut branch, victim, destination);
                let mut expected = before;
                let index = if destination == LibraryDestination::Bottom {
                    0
                } else {
                    expected.len().saturating_sub(1)
                };
                expected.insert(index, victim);
                assert_eq!(branch.state.players[owner.index()].library, expected);
                assert_eq!(branch.object_view(victim).zone, Zone::Library);
                if destination == LibraryDestination::SecondFromTop {
                    assert_eq!(
                        branch.public_library_top(owner),
                        old_top.or_else(|| Some(branch.object_ref(victim)))
                    );
                }
                assert_eq!(
                    branch
                        .observe(owner)
                        .last_target_library_move
                        .unwrap()
                        .destination,
                    destination
                );
            }
        }
    }
}

#[test]
fn temporal_cleansing_convoke_pays_exact_cost_rejects_lands_and_invalid_incarnation_fizzles() {
    let mut g = setup(Player::P0);
    let blue = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Brineborn Cutthroat").unwrap(),
        Zone::Battlefield,
    );
    let bear = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Battlefield,
    );
    let land = spawn(&mut g, Player::P1, Card::Island, Zone::Battlefield);
    let victim = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Mazemind Tome").unwrap(),
        Zone::Battlefield,
    );
    let source = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Temporal Cleansing").unwrap(),
        Zone::Hand,
    );
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    checked(&mut g, Action::AnnounceCast { card: source });
    reject_atomic(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    checked(&mut g, Action::BeginConvoke);
    for (id, contribution) in [
        (blue, ConvokeContribution::Blue),
        (bear, ConvokeContribution::Generic),
    ] {
        let object = g.object_ref(id);
        checked(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution,
            },
        );
    }
    finish_cast_choices(&mut g);
    assert!(g.object_view(blue).tapped && g.object_view(bear).tapped);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    g.move_object(victim, Zone::Hand);
    g.move_object(victim, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    assert!(g.state.players[1].library.iter().all(|&id| id != victim));
    assert!(g.state.resolving.is_none());
}

#[test]
fn temporal_cleansing_target_tokens_cease_without_inserting_library_cards() {
    for destination in [
        LibraryDestination::SecondFromTop,
        LibraryDestination::Bottom,
    ] {
        let mut g = setup(Player::P0);
        let token = spawn(
            &mut g,
            Player::P1,
            Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap(),
            Zone::Battlefield,
        );
        let before = g.state.players[1].library.clone();
        spell(
            &mut g,
            Player::P0,
            "Temporal Cleansing",
            &[Target::Card(token)],
        );
        wait_library_owner(&mut g);
        library_destination(&mut g, token, destination);
        assert_eq!(g.state.players[1].library, before);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}

#[test]
fn swat_away_moves_uncounterable_spells_by_owner_choice_without_countering() {
    for p in [Player::P0, Player::P1] {
        for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
            let mut g = setup(p);
            let victim = spell(&mut g, p, "Sphinx of the Final Word", &[]);
            let stack = g.state.stack.last().unwrap().id;
            assert!(!g.counter_stack_item(stack, false));
            checked(&mut g, Action::Pass);
            spell(&mut g, p.other(), "Swat Away", &[Target::Spell(victim)]);
            wait_library_owner(&mut g);
            assert_eq!(g.actor(), Some(p));
            assert!(g.observe(p.other()).target_library.is_none());
            library_destination(&mut g, victim, destination);
            let library = &g.state.players[p.index()].library;
            assert_eq!(
                if destination == LibraryDestination::Top {
                    library.last()
                } else {
                    library.first()
                },
                Some(&victim)
            );
            assert_eq!(g.object_view(victim).zone, Zone::Library);
            assert!(!g.state.stack.iter().any(|item| item.card == victim));
        }
    }
}

#[test]
fn swat_away_discount_requires_creature_attacking_player_and_stops_after_it_leaves() {
    for p in [Player::P0, Player::P1] {
        for walker_attack in [false, true] {
            let mut g = setup(p.other());
            let attacker = spawn(
                &mut g,
                p.other(),
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            let walker = spawn(
                &mut g,
                p,
                Card::from_name("Vivien Reid").unwrap(),
                Zone::Battlefield,
            );
            g.set_counters(walker, Counters::of(CounterKind::Loyalty, 5));
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            g.state.objects[attacker as usize].summoning_sick = false;
            let defender = g.object_ref(walker);
            checked(
                &mut g,
                if walker_attack {
                    Action::AttackPlaneswalker {
                        card: attacker,
                        defender,
                    }
                } else {
                    Action::Attack { card: attacker }
                },
            );
            checked(&mut g, Action::FinishAttackers);
            checked(&mut g, Action::Pass);
            let source = spawn(&mut g, p, Card::from_name("Swat Away").unwrap(), Zone::Hand);
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
            let available = g
                .legal_actions()
                .contains(&Action::AnnounceCast { card: source });
            assert_eq!(available, !walker_attack);
            if !walker_attack {
                checked(&mut g, Action::AnnounceCast { card: source });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(attacker),
                    },
                );
                finish_cast_choices(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                wait_library_owner(&mut g);
                library_destination(&mut g, attacker, LibraryDestination::Bottom);
            } else {
                g.move_object(attacker, Zone::Hand);
                assert!(
                    !g.legal_actions()
                        .contains(&Action::AnnounceCast { card: source })
                );
            }
        }
    }
}

#[test]
fn swat_away_spell_copies_cease_without_library_insertion_and_original_still_resolves() {
    let mut g = setup(Player::P0);
    let teach = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Teach by Example").unwrap(),
        Zone::Hand,
    );
    checked(&mut g, Action::AnnounceCast { card: teach });
    let choice = g
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::ChooseSpellCost { .. }))
        .unwrap();
    checked(&mut g, choice);
    finish_cast_choices(&mut g);
    settle(&mut g);
    spell(&mut g, Player::P0, "Shock", &[Target::Player(Player::P1)]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::Retarget { .. }));
    checked(&mut g, Action::FinishStackTargets);
    let copy_item = g
        .state
        .stack
        .iter()
        .find(|item| item.is_copy)
        .unwrap()
        .clone();
    let copy = copy_item.card;
    let before = g.state.players[0].library.clone();
    spell(
        &mut g,
        Player::P0,
        "Swat Away",
        &[Target::Stack(copy_item.id)],
    );
    wait_library_owner(&mut g);
    library_destination(&mut g, copy, LibraryDestination::Top);
    assert_eq!(g.state.players[0].library, before);
    assert_eq!(g.object_view(copy).zone, Zone::Outside);
    assert_eq!(g.state.players[1].life, 18);
}

fn behold_announce(g: &mut Game, source: CardId, witness: Option<CardId>, route: Option<u16>) {
    checked(g, Action::AnnounceCast { card: source });
    if let Some(cost) = route {
        checked(g, Action::ChooseSpellCost { cost });
    }
    if let Some(witness) = witness {
        let object = g.object_ref(witness);
        checked(g, Action::ChooseAdditionalCost { object });
    }
    finish_cast_choices(g);
}

#[test]
fn behold_white_blue_reveal_or_controlled_permanent_routes_pay_and_keep_the_witness() {
    for p in [Player::P0, Player::P1] {
        for name in ["Kinsbaile Aspirant", "Silvergill Mentor"] {
            for hand in [false, true] {
                let mut g = setup(p);
                let witness = spawn(
                    &mut g,
                    p,
                    Card::from_name(if hand {
                        "Crib Swap"
                    } else if name == "Kinsbaile Aspirant" {
                        "Clachan Festival"
                    } else {
                        "@token/2cff3b0b2f34a41fce4cd46b"
                    })
                    .unwrap(),
                    if hand { Zone::Hand } else { Zone::Battlefield },
                );
                let object = g.object_ref(witness);
                if hand {
                    assert!(
                        !g.observe(p.other())
                            .objects
                            .iter()
                            .any(|view| view.reference == object)
                    );
                }
                let source = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
                g.state.players[p.index()].mana = if name == "Kinsbaile Aspirant" {
                    [1, 0, 0, 0, 0, 0]
                } else {
                    [0, 1, 0, 0, 0, 1]
                };
                checked(&mut g, Action::AnnounceCast { card: source });
                checked(&mut g, Action::ChooseSpellCost { cost: 0 });
                checked(&mut g, Action::ChooseAdditionalCost { object });
                if hand {
                    assert!(
                        !g.observe(p.other())
                            .objects
                            .iter()
                            .any(|view| view.reference == object)
                    );
                }
                finish_cast_choices(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(
                    g.object_view(witness).zone,
                    if hand { Zone::Hand } else { Zone::Battlefield }
                );
                if hand {
                    assert!(
                        g.observe(p.other())
                            .objects
                            .iter()
                            .any(|view| view.reference == object)
                    );
                }
                settle(&mut g);
                assert_eq!(
                    g.stats(source),
                    Stats {
                        power: 2,
                        toughness: 1
                    }
                );
                if name == "Silvergill Mentor" {
                    assert!(
                        g.state
                            .objects
                            .iter()
                            .enumerate()
                            .any(|(id, object)| object.zone == Zone::Battlefield
                                && object.controller == p
                                && object.card.identity() == "@token/2cff3b0b2f34a41fce4cd46b"
                                && id as CardId != witness)
                    );
                }
            }
        }
    }
}

#[test]
fn behold_white_blue_extra_two_route_and_free_casts_cannot_bypass_the_additional_cost() {
    for p in [Player::P0, Player::P1] {
        for name in ["Kinsbaile Aspirant", "Silvergill Mentor"] {
            for free in [false, true] {
                let mut g = setup(p);
                if free {
                    spawn(
                        &mut g,
                        p,
                        Card::from_name("Omniscience").unwrap(),
                        Zone::Battlefield,
                    );
                }
                let source = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
                let action = if free {
                    Action::AnnounceFree {
                        card: source,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    }
                } else {
                    Action::AnnounceCast { card: source }
                };
                g.state.players[p.index()].mana = if free {
                    [0; 6]
                } else if name == "Kinsbaile Aspirant" {
                    [1, 0, 0, 0, 0, 0]
                } else {
                    [0, 1, 0, 0, 0, 1]
                };
                reject_atomic(&mut g, action.clone());
                g.state.players[p.index()].mana[5] += 2;
                checked(&mut g, action);
                reject_atomic(&mut g, Action::ChooseSpellCost { cost: 0 });
                checked(&mut g, Action::ChooseSpellCost { cost: 1 });
                finish_cast_choices(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                settle(&mut g);
                assert_eq!(
                    g.stats(source),
                    Stats {
                        power: 2,
                        toughness: 1
                    }
                );
            }
        }
    }
}

#[test]
fn kinsbaile_aspirant_each_other_owned_creature_grows_once_and_old_source_does_not_follow_reentry()
{
    let mut g = setup(Player::P0);
    let witness = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Crib Swap").unwrap(),
        Zone::Hand,
    );
    let source = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Kinsbaile Aspirant").unwrap(),
        Zone::Hand,
    );
    behold_announce(&mut g, source, Some(witness), Some(0));
    settle(&mut g);
    assert_eq!(g.stats(source).power, 2);
    let bear = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Hand,
    );
    cast(&mut g, bear, None);
    settle(&mut g);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 3,
            toughness: 2
        }
    );
    let enemy = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Hand,
    );
    g.move_object(enemy, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.stats(source).power, 3);
    g.finish_cleanup();
    assert_eq!(g.stats(source).power, 2);
    let other = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Hand,
    );
    g.move_object(other, Zone::Battlefield);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.stats(source).power, 2);
}

#[test]
fn silvergill_mentor_creates_real_white_blue_merfolk_token_without_behold_when_not_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = spawn(
            &mut g,
            p,
            Card::from_name("Silvergill Mentor").unwrap(),
            Zone::Graveyard,
        );
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == p
                    && object.card.identity() == "@token/2cff3b0b2f34a41fce4cd46b"
            })
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(
            g.stats(token),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.object_view(token).card.colors(), 3);
        assert!(g.has_subtype(token, "Merfolk"));
    }
}

#[test]
fn champions_white_blue_exile_hand_witness_and_return_its_owner_on_every_departure_despite_control_change()
 {
    for p in [Player::P0, Player::P1] {
        for name in ["Champion of the Clachan", "Champions of the Shoal"] {
            for destination in [Zone::Graveyard, Zone::Hand, Zone::Exile, Zone::Library] {
                let mut g = setup(p);
                let witness = spawn(&mut g, p, Card::from_name("Crib Swap").unwrap(), Zone::Hand);
                let source = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
                behold_announce(&mut g, source, Some(witness), None);
                assert_eq!(g.object_view(witness).zone, Zone::Exile);
                settle(&mut g);
                let object = g.object_ref(source);
                g.change_controller(object, p.other());
                g.move_object(source, destination);
                settle(&mut g);
                assert_eq!(g.object_view(witness).zone, Zone::Hand);
                assert_eq!(g.object_view(witness).owner, p);
                assert_eq!(g.object_view(source).zone, destination);
            }
        }
    }
}

#[test]
fn champions_white_blue_countered_cast_does_not_return_cost_or_link_a_later_reanimated_incarnation()
{
    for name in ["Champion of the Clachan", "Champions of the Shoal"] {
        let mut g = setup(Player::P0);
        let source = spawn(
            &mut g,
            Player::P0,
            Card::from_name(name).unwrap(),
            Zone::Hand,
        );
        reject_atomic(&mut g, Action::AnnounceCast { card: source });
        let witness = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Crib Swap").unwrap(),
            Zone::Hand,
        );
        behold_announce(&mut g, source, Some(witness), None);
        let id = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Exile);
    }
}

#[test]
fn champion_of_the_clachan_flash_and_lord_apply_to_other_owned_kithkin_and_stop_with_source_ability_loss()
 {
    let p = Player::P0;
    let mut g = setup(Player::P1);
    let witness = spawn(&mut g, p, Card::from_name("Crib Swap").unwrap(), Zone::Hand);
    let kithkin = spawn(
        &mut g,
        p,
        Card::from_name("Reluctant Dounguard").unwrap(),
        Zone::Battlefield,
    );
    let changeling = spawn(
        &mut g,
        p,
        Card::from_name("Prideful Feastling").unwrap(),
        Zone::Battlefield,
    );
    let enemy = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Reluctant Dounguard").unwrap(),
        Zone::Battlefield,
    );
    let before = g.stats(kithkin);
    let changeling_before = g.stats(changeling);
    let enemy_before = g.stats(enemy);
    checked(&mut g, Action::Pass);
    let source = spawn(
        &mut g,
        p,
        Card::from_name("Champion of the Clachan").unwrap(),
        Zone::Hand,
    );
    behold_announce(&mut g, source, Some(witness), None);
    settle(&mut g);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 4,
            toughness: 5
        }
    );
    assert_eq!(
        g.stats(kithkin),
        Stats {
            power: before.power + 1,
            toughness: before.toughness + 1
        }
    );
    assert_eq!(
        g.stats(changeling),
        Stats {
            power: changeling_before.power + 1,
            toughness: changeling_before.toughness + 1
        }
    );
    assert_eq!(g.stats(enemy), enemy_before);
    g.state.priority = p;
    spell(&mut g, p, "Noggle the Mind", &[Target::Card(source)]);
    settle(&mut g);
    assert_eq!(g.stats(kithkin), before);
    g.move_object(source, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.object_view(witness).zone, Zone::Exile);
}

#[test]
fn champions_of_the_shoal_entry_and_actual_tap_each_offer_one_target_and_stun_even_already_tapped()
{
    let mut g = setup(Player::P0);
    let witness = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Crib Swap").unwrap(),
        Zone::Hand,
    );
    let victim = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Bear Cub").unwrap(),
        Zone::Battlefield,
    );
    let source = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Champions of the Shoal").unwrap(),
        Zone::Hand,
    );
    behold_announce(&mut g, source, Some(witness), None);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    settle(&mut g);
    assert!(g.object_view(victim).tapped);
    assert_eq!(g.counter_count(victim, CounterKind::Stun), 1);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 4,
            toughness: 6
        }
    );
    g.tap_objects(&[source]);
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    settle(&mut g);
    assert_eq!(g.counter_count(victim, CounterKind::Stun), 2);
    g.tap_objects(&[source]);
    assert!(g.state.pending_triggers.is_empty());
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    g.tap_objects(&[source]);
    g.prepare_triggers();
    checked(&mut g, Action::FinishTargets);
    settle(&mut g);
    assert_eq!(g.counter_count(victim, CounterKind::Stun), 2);
}

fn solitude_activate(g: &mut Game, aura: CardId, witness: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: aura,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(witness);
    checked(g, Action::ChooseAdditionalCost { object });
    finish_cast_choices(g);
}

#[test]
fn spiral_into_solitude_restricts_combat_and_paid_nontarget_exile_uses_sacrificed_aura_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let aura = spell(&mut g, p, "Spiral into Solitude", &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.combat_restricted(bear, true));
        assert!(g.combat_restricted(bear, false));
        // The restriction belongs to the Aura, so losing the creature's own
        // abilities does not release it. Hexproof and indestructible cannot
        // stop this nontargeted exile ability.
        spell(&mut g, p, "Noggle the Mind", &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.combat_restricted(bear, true));
        assert!(g.combat_restricted(bear, false));
        let source = g.object_ref(aura);
        g.grant_keyword_effects(
            &[bear],
            source,
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
        );
        let mana_before: u64 = g.state.players[p.index()]
            .mana
            .iter()
            .map(|&n| u64::from(n))
            .sum();
        solitude_activate(&mut g, aura, witness);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 1);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(g.state.stack.last().unwrap().targets.is_empty());
        let mana_after: u64 = g.state.players[p.index()]
            .mana
            .iter()
            .map(|&n| u64::from(n))
            .sum();
        assert_eq!(mana_before - mana_after, 2);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
    }
}

#[test]
fn spiral_into_solitude_old_attachment_does_not_follow_reentry_or_a_new_aura_attachment() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let aura = spell(&mut g, p, "Spiral into Solitude", &[Target::Card(bear)]);
            settle(&mut g);
            solitude_activate(&mut g, aura, witness);
            g.move_object(bear, Zone::Hand);
            if reenter {
                g.move_object(bear, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(bear).zone,
                if reenter {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
        }
        let mut g = setup(p);
        let old = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let new = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let aura = spell(&mut g, p, "Spiral into Solitude", &[Target::Card(old)]);
        settle(&mut g);
        solitude_activate(&mut g, aura, witness);
        // A later incarnation of the Aura cannot change the old ability's
        // captured attachment.
        let mut movement = g.entry_move(aura, Zone::Battlefield, false);
        movement.attachment = Some(g.object_ref(new));
        g.stage_entries(vec![movement], None);
        assert!(!g.advance_entries());
        settle(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Exile);
        assert_eq!(g.object_view(new).zone, Zone::Battlefield);
        assert_eq!(
            g.state.objects[aura as usize].attached_to,
            Some(g.object_ref(new))
        );
    }
}

#[test]
fn spiral_into_solitude_exiles_tokens_and_can_blight_the_enchanted_creature_without_targeting_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let aura = spell(&mut g, p, "Spiral into Solitude", &[Target::Card(bear)]);
        settle(&mut g);
        solitude_activate(&mut g, aura, bear);
        assert_eq!(g.counter_count(bear, CounterKind::MinusOne), 1);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        let token = spawn(
            &mut g,
            p.other(),
            Card::catalogue()
                .find(|card| card.identity() == "@token/2cff3b0b2f34a41fce4cd46b")
                .unwrap(),
            Zone::Battlefield,
        );
        let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let aura = spell(&mut g, p, "Spiral into Solitude", &[Target::Card(token)]);
        settle(&mut g);
        solitude_activate(&mut g, aura, witness);
        settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}

#[test]
fn spiral_into_solitude_blight_cost_cannot_use_enemy_or_counter_prohibited_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let aura = spell(&mut g, p, "Spiral into Solitude", &[Target::Card(bear)]);
        settle(&mut g);
        let action = Action::AnnounceActivate {
            card: aura,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject_atomic(&mut g, action.clone());
        let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let prohibition = spell(&mut g, p, "Blossombind", &[Target::Card(witness)]);
        settle(&mut g);
        reject_atomic(&mut g, action.clone());
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
        assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
        g.move_object(prohibition, Zone::Graveyard);
        checked(&mut g, action);
        let enemy = g.object_ref(bear);
        reject_atomic(&mut g, Action::ChooseAdditionalCost { object: enemy });
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
    }
}

fn attack_tap_payment(g: &mut Game, source: CardId) -> bool {
    thoughtweft_attack(g, &[source]);
    advance_to_tap_payment(g)
}
fn advance_to_tap_payment(g: &mut Game) -> bool {
    g.prepare_triggers();
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return true;
        }
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return false;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::PayCosts { .. } | Action::FinishTargets
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("attack trigger did not reach payment");
}
fn guide_target(g: &mut Game, target: CardId) {
    g.prepare_triggers();
    let target = Target::Card(target);
    for _ in 0..20 {
        if g.legal_actions().contains(&Action::ChooseTarget { target }) {
            checked(g, Action::ChooseTarget { target });
            finish_cast_choices(g);
            settle(g);
            return;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::PayCosts { .. } | Action::FinishTargets
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("reflexive target not offered");
}

#[test]
fn gravelgill_scoundrel_vigilance_optional_tap_is_exact_nontarget_and_unblockable_expires() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let source = cast_named(&mut g, p, "Gravelgill Scoundrel");
            let witness = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[witness as usize].summoning_sick = true;
            let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            let tapped = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            g.tap_objects(&[tapped]);
            let source_ref = g.object_ref(source);
            g.grant_keyword_effects(&[witness], source_ref, Keywords::HEXPROOF);
            assert!(attack_tap_payment(&mut g, source));
            assert!(!g.object_view(source).tapped);
            for invalid in [source, enemy, tapped] {
                let object = g.object_ref(invalid);
                reject_atomic(&mut g, Action::TapForEffect { object });
            }
            if pay {
                let object = g.object_ref(witness);
                checked(&mut g, Action::TapForEffect { object });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.object_view(witness).tapped, pay);
            assert_eq!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED), pay);
            // A nontarget tap cost may use a summoning-sick, hexproof witness.
            assert!(g.object_view(witness).summoning_sick);
            if pay {
                let object = g.object_ref(source);
                g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::EndOfTurn);
                assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
                g.state.pending = Pending::Blockers;
                g.state.priority = p.other();
                reject_atomic(
                    &mut g,
                    Action::Block {
                        blocker: enemy,
                        attacker: source,
                    },
                );
            }
            g.finish_cleanup();
            assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}

#[test]
fn gravelgill_scoundrel_no_witness_skips_and_old_trigger_never_grants_to_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Gravelgill Scoundrel");
        assert!(!attack_tap_payment(&mut g, source));
        assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Gravelgill Scoundrel");
        thoughtweft_attack(&mut g, &[source]);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert!(advance_to_tap_payment(&mut g));
        // "Another" compares exact objects. This new incarnation is another
        // permanent relative to the old trigger, but cannot receive its grant.
        let object = g.object_ref(source);
        checked(&mut g, Action::TapForEffect { object });
        settle(&mut g);
        assert!(g.object_view(source).tapped);
        assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
    }
}

#[test]
fn meanders_guide_merfolk_payment_includes_noncreature_changeling_then_reflexive_targets_own_small_creature()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Meanders Guide");
        let witness = spawn(
            &mut g,
            p,
            Card::from_name("Firdoch Core").unwrap(),
            Zone::Battlefield,
        );
        let wrong_type = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(
            &mut g,
            p.other(),
            Card::from_name("Brineborn Cutthroat").unwrap(),
            Zone::Battlefield,
        );
        let target = spawn(&mut g, p, Card::GrizzlyBears, Zone::Graveyard);
        let too_large = spawn(&mut g, p, Card::HillGiant, Zone::Graveyard);
        let enemy_grave = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Graveyard);
        let land = spawn(&mut g, p, Card::Forest, Zone::Graveyard);
        assert!(!g.is_creature(witness));
        assert!(g.has_subtype(witness, "Merfolk"));
        assert!(attack_tap_payment(&mut g, source));
        assert!(g.object_view(source).tapped);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
        );
        for invalid in [wrong_type, enemy, source] {
            let object = g.object_ref(invalid);
            reject_atomic(&mut g, Action::TapForEffect { object });
        }
        let object = g.object_ref(witness);
        checked(&mut g, Action::TapForEffect { object });
        assert!(g.object_view(witness).tapped);
        g.prepare_triggers();
        for invalid in [too_large, enemy_grave, land] {
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        guide_target(&mut g, target);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert!(!g.object_view(target).tapped);
        assert!(g.object_view(target).summoning_sick);
    }
}

#[test]
fn meanders_guide_may_decline_or_pay_without_graveyard_targets_and_never_taps_already_tapped_merfolk()
 {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let source = cast_named(&mut g, p, "Meanders Guide");
            let witness = spawn(
                &mut g,
                p,
                Card::from_name("Brineborn Cutthroat").unwrap(),
                Zone::Battlefield,
            );
            let tapped = spawn(
                &mut g,
                p,
                Card::from_name("Brineborn Cutthroat").unwrap(),
                Zone::Battlefield,
            );
            g.tap_objects(&[tapped]);
            assert!(attack_tap_payment(&mut g, source));
            let object = g.object_ref(tapped);
            reject_atomic(&mut g, Action::TapForEffect { object });
            if pay {
                let object = g.object_ref(witness);
                checked(&mut g, Action::TapForEffect { object });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.object_view(witness).tapped, pay);
            assert!(g.state.stack.is_empty());
            assert!(g.state.announcing.is_none());
        }
    }
}

#[test]
fn meanders_guide_captured_controller_can_pay_after_source_changes_controller_or_leaves() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let source = cast_named(&mut g, p, "Meanders Guide");
            let witness = spawn(
                &mut g,
                p,
                Card::from_name("Brineborn Cutthroat").unwrap(),
                Zone::Battlefield,
            );
            let target = spawn(&mut g, p, Card::GrizzlyBears, Zone::Graveyard);
            thoughtweft_attack(&mut g, &[source]);
            if leaves {
                g.move_object(source, Zone::Graveyard);
            } else {
                let object = g.object_ref(source);
                g.change_controller(object, p.other());
            }
            assert!(advance_to_tap_payment(&mut g));
            assert_eq!(g.actor(), Some(p));
            let object = g.object_ref(witness);
            checked(&mut g, Action::TapForEffect { object });
            guide_target(&mut g, if leaves { source } else { target });
            let returned = if leaves { source } else { target };
            assert_eq!(g.object_view(returned).zone, Zone::Battlefield);
            assert_eq!(g.object_view(returned).controller, p);
        }
    }
}

#[test]
fn meanders_guide_reflexive_target_is_bound_before_response_and_does_not_follow_graveyard_reentry()
{
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Meanders Guide");
    let witness = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Brineborn Cutthroat").unwrap(),
        Zone::Battlefield,
    );
    let target = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Graveyard);
    assert!(attack_tap_payment(&mut g, source));
    let object = g.object_ref(witness);
    checked(&mut g, Action::TapForEffect { object });
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish_cast_choices(&mut g);
    let old = g.object_ref(target);
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Graveyard);
    assert_ne!(old, g.object_ref(target));
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    assert!(g.object_view(witness).tapped);
}

fn sentry_choice(g: &mut Game, attackers: &[CardId]) {
    thoughtweft_attack(g, attackers);
    assert!(advance_to_tap_payment(g));
    assert_eq!(
        g.state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap()
            .kind,
        EffectChoiceKind::HandCreature
    );
}

#[test]
fn kinscaer_sentry_bounds_hand_entry_by_current_attackers_and_entering_attacking_does_not_trigger_attack()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Kinscaer Sentry");
        let a = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let b = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let entry = spawn(
            &mut g,
            p,
            Card::from_name("Meanders Guide").unwrap(),
            Zone::Hand,
        );
        let too_large = spawn(&mut g, p, Card::HillGiant, Zone::Hand);
        let land = spawn(&mut g, p, Card::Forest, Zone::Hand);
        let enemy = spawn(&mut g, p.other(), Card::LlanowarElves, Zone::Hand);
        sentry_choice(&mut g, &[source, a, b]);
        for invalid in [too_large, land, enemy] {
            let object = g.object_ref(invalid);
            reject_atomic(&mut g, Action::PutHandCreature { object });
        }
        let object = g.object_ref(entry);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        assert_eq!(g.object_view(entry).zone, Zone::Battlefield);
        assert!(g.object_view(entry).tapped);
        assert!(g.object_view(entry).summoning_sick);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == entry && a.defender == Some(CombatDefender::Player(p.other())))
        );
        assert!(g.state.stack.is_empty());
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.resolving.is_none());
    }
}

#[test]
fn kinscaer_sentry_attacker_departure_lowers_resolution_limit_and_source_departure_does_not_stop_it()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Kinscaer Sentry");
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let two = spawn(&mut g, p, Card::GrizzlyBears, Zone::Hand);
        let one = spawn(&mut g, p, Card::LlanowarElves, Zone::Hand);
        thoughtweft_attack(&mut g, &[source, bear]);
        g.move_object(source, Zone::Graveyard);
        assert!(advance_to_tap_payment(&mut g));
        let object = g.object_ref(two);
        reject_atomic(&mut g, Action::PutHandCreature { object });
        let object = g.object_ref(one);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        assert_eq!(g.object_view(one).zone, Zone::Battlefield);
        assert!(g.object_view(one).tapped);
        assert!(g.state.attackers.iter().any(|a| a.card == one));
        assert_eq!(g.object_view(two).zone, Zone::Hand);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn kinscaer_sentry_hand_phase_hides_eligibility_and_decline_preserves_hand() {
    for p in [Player::P0, Player::P1] {
        let make = |eligible| {
            let mut g = setup(p);
            let source = cast_named(&mut g, p, "Kinscaer Sentry");
            let card = spawn(
                &mut g,
                p,
                if eligible {
                    Card::LlanowarElves
                } else {
                    Card::HillGiant
                },
                Zone::Hand,
            );
            sentry_choice(&mut g, &[source]);
            (g, card)
        };
        let (mut yes, y) = make(true);
        let (mut no, n) = make(false);
        assert_eq!(
            serde_json::to_value(yes.observe(p.other())).unwrap(),
            serde_json::to_value(no.observe(p.other())).unwrap()
        );
        assert!(
            yes.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PutHandCreature { .. }))
        );
        assert!(
            !no.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PutHandCreature { .. }))
        );
        for (g, card) in [(&mut yes, y), (&mut no, n)] {
            checked(g, Action::DeclineHandEntry);
            settle(g);
            assert_eq!(g.object_view(card).zone, Zone::Hand);
        }
    }
}

#[test]
fn kinscaer_sentry_attacking_entry_can_choose_a_different_defender_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Kinscaer Sentry");
        let walker = spawn(
            &mut g,
            p.other(),
            Card::from_name("Ajani, Caller of the Pride").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 4;
        let entry = spawn(&mut g, p, Card::LlanowarElves, Zone::Hand);
        let attacker = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        thoughtweft_attack(&mut g, &[source, attacker]);
        let source_ref = g.object_ref(source);
        g.change_controller(source_ref, p.other());
        // The other declared attacker still supplies X=1 for the captured controller.
        assert!(advance_to_tap_payment(&mut g));
        assert_eq!(g.actor(), Some(p));
        let object = g.object_ref(entry);
        checked(&mut g, Action::PutHandCreature { object });
        assert!(matches!(g.state.pending, Pending::HandDefender { .. }));
        let defender = CombatDefender::Planeswalker(g.object_ref(walker));
        checked(&mut g, Action::ChooseHandDefender { defender });
        settle(&mut g);
        assert_eq!(g.object_view(entry).controller, p);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == entry && a.defender == Some(defender))
        );
        assert!(g.object_view(entry).tapped);
    }
}

#[test]
fn kinscaer_sentry_first_strike_and_lifelink_win_combat_before_regular_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Kinscaer Sentry");
        let blocker = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        sentry_choice(&mut g, &[source]);
        checked(&mut g, Action::DeclineHandEntry);
        settle(&mut g);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).damage, 0);
        assert_eq!(g.state.players[p.index()].life, 22);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}

fn rhys_grant(g: &mut Game, p: Player, target: CardId) -> CardId {
    let source = spawn(
        g,
        p,
        Card::from_name("Rhys, the Evermore").unwrap(),
        Zone::Hand,
    );
    checked(g, Action::AnnounceCast { card: source });
    finish_cast_choices(g);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish_cast_choices(g);
    settle(g);
    source
}
fn rhys_removal(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish_cast_choices(g);
    assert!(advance_to_tap_payment(g));
    assert_eq!(
        g.counter_removal_domain().unwrap().recipient,
        g.object_ref(target)
    );
}
fn checked_counter_raw(g: &mut Game, counters: Counters) {
    let actor = g.actor().unwrap();
    let action = Action::ChooseCounterRemoval { counters };
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut child = {
        let parent = g.clone();
        let owned = parent.clone();
        drop(parent);
        owned
    };
    child.apply(actor, action.clone()).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&before).unwrap();
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), done);
}

#[test]
fn rhys_flash_grants_persist_that_uses_current_death_controller_owner_and_exact_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p;
        let source = rhys_grant(&mut g, p, bear);
        let original = g.object_ref(bear);
        g.move_object(source, Zone::Graveyard);
        g.change_controller(original, p.other());
        g.move_object(bear, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(bear), original);
        assert_eq!(g.object_view(bear).controller, p);
        assert!(!g.object_view(bear).tapped);
        assert_eq!(g.counter_count(bear, CounterKind::MinusOne), 1);
        g.move_object(bear, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}

#[test]
fn rhys_persist_minus_counter_ability_loss_expiry_and_old_graveyard_incarnation_prevent_return() {
    for mode in 0..4 {
        let mut g = setup(Player::P0);
        let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        rhys_grant(&mut g, Player::P0, bear);
        match mode {
            0 => {
                g.add_counter(bear, CounterKind::MinusOne, 1);
            }
            1 => {
                let object = g.object_ref(bear);
                g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::EndOfTurn);
            }
            2 => g.finish_cleanup(),
            _ => {}
        }
        g.move_object(bear, Zone::Graveyard);
        if mode == 3 {
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard, "mode {mode}");
    }
}

#[test]
fn rhys_counter_removal_accepts_exact_nonrepresentative_mixed_named_allocations_and_rejects_overpay()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = spawn(
            &mut g,
            p,
            Card::from_name("Rhys, the Evermore").unwrap(),
            Zone::Battlefield,
        );
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.set_counters(
            bear,
            Counters::of(CounterKind::PlusOne, 5)
                .with(CounterKind::Stun, 8)
                .with(CounterKind::Soul, 13)
                .with(CounterKind::Charge, 21),
        );
        rhys_removal(&mut g, source, bear);
        assert!(g.object_view(source).tapped);
        reject_atomic(
            &mut g,
            Action::ChooseCounterRemoval {
                counters: Counters::of(CounterKind::Soul, 14),
            },
        );
        let counters = Counters::of(CounterKind::PlusOne, 2)
            .with(CounterKind::Stun, 3)
            .with(CounterKind::Soul, 5)
            .with(CounterKind::Charge, 8);
        assert!(
            g.legal_action_space()
                .counter_removal
                .unwrap()
                .contains(&counters)
        );
        assert!(!g.legal_actions().contains(&Action::ChooseCounterRemoval {
            counters: counters.clone()
        }));
        checked_counter_raw(&mut g, counters);
        settle(&mut g);
        for (kind, amount) in [
            (CounterKind::PlusOne, 3),
            (CounterKind::Stun, 5),
            (CounterKind::Soul, 8),
            (CounterKind::Charge, 13),
        ] {
            assert_eq!(g.counter_count(bear, kind), amount);
        }
    }
}

#[test]
fn rhys_counter_removal_handles_zero_and_u64_named_counts_without_enumerating_quantities() {
    for empty in [false, true] {
        let mut g = setup(Player::P0);
        let source = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Rhys, the Evermore").unwrap(),
            Zone::Battlefield,
        );
        let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        if !empty {
            g.set_counters(
                bear,
                Counters::of(CounterKind::Soul, u64::MAX).with(CounterKind::Charge, u64::MAX),
            );
        }
        rhys_removal(&mut g, source, bear);
        assert!(g.legal_actions().len() < 40);
        let mut terminal = g.clone();
        terminal.apply(Player::P0, Action::Concede).unwrap();
        assert!(terminal.legal_action_space().counter_removal.is_none());
        let counters = if empty {
            Counters::default()
        } else {
            Counters::of(CounterKind::Soul, u64::MAX - 7).with(CounterKind::Charge, u64::MAX - 11)
        };
        checked_counter_raw(&mut g, counters);
        settle(&mut g);
        assert_eq!(
            g.counter_count(bear, CounterKind::Soul),
            if empty { 0 } else { 7 }
        );
        assert_eq!(
            g.counter_count(bear, CounterKind::Charge),
            if empty { 0 } else { 11 }
        );
    }
}

#[test]
fn rhys_counter_activation_is_sorcery_speed_requires_tap_and_uses_exact_target_even_after_source_departure()
 {
    let mut g = setup(Player::P0);
    let source = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Rhys, the Evermore").unwrap(),
        Zone::Battlefield,
    );
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let action = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    g.state.objects[source as usize].summoning_sick = true;
    reject_atomic(&mut g, action.clone());
    g.state.objects[source as usize].summoning_sick = false;
    g.state.active = Player::P1;
    reject_atomic(&mut g, action.clone());
    g.state.active = Player::P0;
    let mana = g.state.players[0].mana;
    g.state.players[0].mana[0] = 0;
    reject_atomic(&mut g, action.clone());
    g.state.players[0].mana = mana;
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    checked(&mut g, action);
    reject_atomic(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish_cast_choices(&mut g);
    g.move_object(source, Zone::Graveyard);
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    g.set_counters(bear, Counters::of(CounterKind::Stun, 2));
    settle(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::Stun), 2);
}

fn walker_end_choice(g: &mut Game) -> bool {
    g.begin_step(Step::End);
    advance_to_tap_payment(g)
}
#[test]
fn slumbering_walker_entry_counters_and_optional_named_removal_precede_reflexive_small_power_target()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Slumbering Walker");
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 2);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 2,
                toughness: 5
            }
        );
        g.add_counter(source, CounterKind::Soul, 1);
        let target = spawn(
            &mut g,
            p,
            Card::from_name("Thoughtweft Imbuer").unwrap(),
            Zone::Graveyard,
        );
        let large = spawn(&mut g, p, Card::HillGiant, Zone::Graveyard);
        let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Graveyard);
        assert!(walker_end_choice(&mut g));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
        );
        reject_atomic(
            &mut g,
            Action::ChooseCounterRemoval {
                counters: Counters::default(),
            },
        );
        reject_atomic(
            &mut g,
            Action::ChooseCounterRemoval {
                counters: Counters::of(CounterKind::MinusOne, 2),
            },
        );
        checked(
            &mut g,
            Action::ChooseCounterRemoval {
                counters: Counters::of(CounterKind::Soul, 1),
            },
        );
        g.prepare_triggers();
        reject_atomic(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(large),
            },
        );
        reject_atomic(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        guide_target(&mut g, target);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert!(!g.object_view(target).tapped);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 2);
        assert_eq!(g.counter_count(source, CounterKind::Soul), 0);
    }
}

#[test]
fn slumbering_walker_decline_no_counters_and_old_source_incarnation_never_create_reflexive_return()
{
    for mode in 0..4 {
        let mut g = setup(Player::P0);
        let source = cast_named(&mut g, Player::P0, "Slumbering Walker");
        let target = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Graveyard);
        if mode == 1 {
            g.set_counters(source, Counters::default());
        }
        if mode == 3 {
            g.state.active = Player::P1;
        }
        g.begin_step(Step::End);
        if mode == 2 {
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
        }
        let choice = advance_to_tap_payment(&mut g);
        assert_eq!(choice, mode == 0);
        if choice {
            checked(&mut g, Action::DeclineEffectPayment);
        }
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(
            g.counter_count(source, CounterKind::MinusOne),
            if mode == 1 { 0 } else { 2 }
        );
    }
}

#[test]
fn slumbering_walker_may_pay_without_a_graveyard_target_and_blossombind_does_not_prevent_removal() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Slumbering Walker");
    spell(&mut g, Player::P0, "Blossombind", &[Target::Card(source)]);
    settle(&mut g);
    assert!(!g.can_receive_counters(source));
    assert!(walker_end_choice(&mut g));
    checked(
        &mut g,
        Action::ChooseCounterRemoval {
            counters: Counters::of(CounterKind::MinusOne, 1),
        },
    );
    settle(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::MinusOne), 1);
    assert_eq!(
        g.stats(source),
        Stats {
            power: 3,
            toughness: 6
        }
    );
}

#[test]
fn rhys_counter_removal_can_target_itself_or_resolve_after_its_source_has_left() {
    for p in [Player::P0, Player::P1] {
        for self_target in [false, true] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Rhys, the Evermore").unwrap(),
                Zone::Battlefield,
            );
            let target = if self_target {
                source
            } else {
                spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield)
            };
            g.set_counters(target, Counters::of(CounterKind::Soul, 2));
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish_cast_choices(&mut g);
            if !self_target {
                g.move_object(source, Zone::Graveyard);
            }
            assert!(advance_to_tap_payment(&mut g));
            checked(
                &mut g,
                Action::ChooseCounterRemoval {
                    counters: Counters::of(CounterKind::Soul, 1),
                },
            );
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::Soul), 1);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn rhys_entry_persist_requires_another_owned_creature_and_cannot_follow_reentry_before_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let source = spawn(
            &mut g,
            p,
            Card::from_name("Rhys, the Evermore").unwrap(),
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceCast { card: source });
        finish_cast_choices(&mut g);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        for invalid in [source, enemy] {
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish_cast_choices(&mut g);
        let old = g.object_ref(target);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(old, g.object_ref(target));
        settle(&mut g);
        g.move_object(target, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}

#[test]
fn rhys_granted_persist_cannot_return_a_token_even_though_it_triggers_on_its_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token_card = Card::catalogue()
            .find(|card| {
                card.token_info().is_some() && matches!(card.effect(), Effect::Creature(3, 3))
            })
            .unwrap();
        let token = spawn(&mut g, p, token_card, Zone::Battlefield);
        rhys_grant(&mut g, p, token);
        assert!(g.object_view(token).is_token());
        g.move_object(token, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}

fn colossus_trigger(g: &mut Game, p: Player) -> CardId {
    let source = spawn(
        g,
        p,
        Card::from_name("Curious Colossus").unwrap(),
        Zone::Hand,
    );
    checked(g, Action::AnnounceCast { card: source });
    finish_cast_choices(g);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    reject_atomic(
        g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_cast_choices(g);
    source
}

#[test]
fn curious_colossus_paid_body_freezes_resolution_creatures_and_keeps_modifiers_and_other_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(
            &mut g,
            p.other(),
            Card::from_name("Juggernaut").unwrap(),
            Zone::Battlefield,
        );
        let late = spawn(
            &mut g,
            p.other(),
            Card::from_name("Serra Angel").unwrap(),
            Zone::Hand,
        );
        let land = spawn(&mut g, p.other(), Card::Island, Zone::Battlefield);
        let source = colossus_trigger(&mut g, p);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 7,
                toughness: 7
            }
        );
        let original = g.object_ref(enemy);
        g.set_counters(
            enemy,
            Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Flying, 1),
        );
        g.state.objects[enemy as usize].growth = Stats {
            power: 3,
            toughness: 4,
        };
        g.grant_keyword_effects(
            &[enemy],
            original,
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
        );
        // This entrant is included even though it was absent when the ability triggered.
        g.move_object(late, Zone::Battlefield);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 6,
                toughness: 7
            }
        );
        assert_eq!(
            g.stats(late),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(
            g.stats(own),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.printed_abilities_available(enemy));
        assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(enemy).contains(Keywords::FLYING));
        assert_eq!(g.counter_count(enemy, CounterKind::Flying), 1);
        assert!(!g.keywords(late).contains(Keywords::FLYING));
        assert!(g.current_kinds(enemy) & 16 != 0);
        assert!(g.has_subtype(enemy, "Juggernaut"));
        assert!(g.has_subtype(enemy, "Coward"));
        assert_eq!(
            g.object_view(late).colors,
            Card::from_name("Serra Angel").unwrap().colors()
        );
        assert!(!g.has_subtype(land, "Coward"));
        let later = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        assert!(!g.has_subtype(later, "Coward"));
        assert_eq!(g.stats(later).power, 2);
        g.change_controller(original, p);
        g.finish_cleanup();
        assert!(g.has_subtype(enemy, "Coward"));
        assert!(!g.printed_abilities_available(enemy));
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        g.move_object(enemy, Zone::Exile);
        g.move_object(enemy, Zone::Battlefield);
        settle(&mut g);
        assert_ne!(g.object_ref(enemy), original);
        assert!(!g.has_subtype(enemy, "Coward"));
        assert!(g.printed_abilities_available(enemy));
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 5,
                toughness: 3
            }
        );
    }
}

#[test]
fn curious_colossus_changeling_and_later_keyword_and_base_effects_follow_layers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = spawn(
            &mut g,
            p.other(),
            Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap(),
            Zone::Battlefield,
        );
        let source = colossus_trigger(&mut g, p);
        settle(&mut g);
        assert!(!g.keywords(target).contains(Keywords::CHANGELING));
        for subtype in ["Coward", "Faerie", "Goblin", "Shapeshifter"] {
            assert!(g.has_subtype(target, subtype));
        }
        g.grant_keyword_effects(&[target], g.object_ref(source), Keywords::FLYING);
        assert!(g.keywords(target).contains(Keywords::FLYING));
        // A later layer-7b effect temporarily supersedes the permanent 1/1 setter.
        let item = spell(&mut g, p, "Glamer Gifter", &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(target),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        g.move_object(item, Zone::Graveyard);
        g.finish_cleanup();
        assert_eq!(
            g.stats(target),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(!g.keywords(target).contains(Keywords::FLYING));
        assert!(!g.printed_abilities_available(target));
        assert!(g.has_subtype(target, "Goblin"));
    }
}

#[test]
fn curious_colossus_resolution_group_uses_current_control_and_trigger_survives_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let leaving = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let joining = spawn(
            &mut g,
            p,
            Card::from_name("Serra Angel").unwrap(),
            Zone::Battlefield,
        );
        let source = colossus_trigger(&mut g, p);
        let object = g.object_ref(source);
        g.change_controller(object, p.other());
        let object = g.object_ref(leaving);
        g.change_controller(object, p);
        let object = g.object_ref(joining);
        g.change_controller(object, p.other());
        settle(&mut g);
        assert_eq!(g.stats(leaving).power, 2);
        assert_eq!(g.stats(joining).power, 1);
        assert!(!g.keywords(joining).contains(Keywords::FLYING));
        // The Colossus is now itself in the chosen opponent's resolution-time group.
        assert_eq!(
            g.stats(source),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.has_subtype(source, "Giant"));
        assert!(g.has_subtype(source, "Warrior"));
        assert!(g.has_subtype(source, "Coward"));
        assert!(!g.printed_abilities_available(source));
    }
}

#[test]
fn rhys_exact_counter_removal_updates_keyword_counter_grants_on_partial_and_zero_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = spawn(
            &mut g,
            p,
            Card::from_name("Rhys, the Evermore").unwrap(),
            Zone::Battlefield,
        );
        let target = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let object = g.object_ref(target);
        g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::Permanent);
        g.set_counters(
            target,
            Counters::of(CounterKind::Flying, 2)
                .with(CounterKind::FirstStrike, 1)
                .with(CounterKind::Lifelink, 1),
        );
        rhys_removal(&mut g, source, target);
        checked_counter_raw(
            &mut g,
            Counters::of(CounterKind::Flying, 1).with(CounterKind::FirstStrike, 1),
        );
        settle(&mut g);
        assert!(
            g.keywords(target)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        assert!(!g.keywords(target).contains(Keywords::FIRST_STRIKE));
        g.state.objects[source as usize].tapped = false;
        rhys_removal(&mut g, source, target);
        checked_counter_raw(
            &mut g,
            Counters::of(CounterKind::Flying, 1).with(CounterKind::Lifelink, 1),
        );
        settle(&mut g);
        assert!(!g.keywords(target).contains(Keywords::FLYING));
        assert!(!g.keywords(target).contains(Keywords::LIFELINK));
        assert_eq!(g.object_view(target).counters.total(), 0);
        g.move_object(source, Zone::Graveyard);
        rhys_grant(&mut g, p, target);
        g.move_object(target, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 1);
    }
}

fn farewell_tokens(g: &Game, p: Player) -> Vec<CardId> {
    let token = Card::from_name("@token/2cff3b0b2f34a41fce4cd46b").unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.controller == p && object.card == token)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn wanderwine_farewell_returns_one_or_two_exact_nonland_permanents_and_creates_real_tokens() {
    for p in [Player::P0, Player::P1] {
        for two in [false, true] {
            let mut g = setup(p);
            let core = spawn(
                &mut g,
                p,
                Card::from_name("Firdoch Core").unwrap(),
                Zone::Battlefield,
            );
            assert!(!g.is_creature(core));
            let first = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            let second = spawn(
                &mut g,
                p.other(),
                Card::from_name("Juggernaut").unwrap(),
                Zone::Battlefield,
            );
            let original = g.object_ref(first);
            g.change_controller(original, p);
            let land = spawn(&mut g, p.other(), Card::Island, Zone::Battlefield);
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Wanderwine Farewell").unwrap(),
                Zone::Hand,
            );
            checked(&mut g, Action::AnnounceCast { card: source });
            reject_atomic(&mut g, Action::FinishTargets);
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            if two {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(second),
                    },
                );
                reject_atomic(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(core),
                    },
                );
            }
            finish_cast_choices(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(first).zone, Zone::Hand);
            assert_eq!(g.object_view(first).owner, p.other());
            assert_eq!(
                g.object_view(second).zone,
                if two { Zone::Hand } else { Zone::Battlefield }
            );
            let tokens = farewell_tokens(&g, p);
            assert_eq!(tokens.len(), if two { 2 } else { 1 });
            for token in tokens {
                assert_eq!(
                    g.stats(token),
                    Stats {
                        power: 1,
                        toughness: 1
                    }
                );
                assert_eq!(g.object_view(token).colors, 3);
                assert!(g.has_subtype(token, "Merfolk"));
                assert!(!g.object_view(token).tapped);
            }
        }
    }
}

#[test]
fn wanderwine_farewell_merfolk_condition_is_checked_after_bounce_and_token_departures_count() {
    for p in [Player::P0, Player::P1] {
        for return_last_merfolk in [false, true] {
            let mut g = setup(p);
            let core = spawn(
                &mut g,
                p,
                Card::from_name("Firdoch Core").unwrap(),
                Zone::Battlefield,
            );
            let token = spawn(
                &mut g,
                p.other(),
                Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap(),
                Zone::Battlefield,
            );
            let targets = if return_last_merfolk {
                vec![Target::Card(core), Target::Card(token)]
            } else {
                vec![Target::Card(token)]
            };
            spell(&mut g, p, "Wanderwine Farewell", &targets);
            settle(&mut g);
            assert_eq!(
                farewell_tokens(&g, p).len(),
                usize::from(!return_last_merfolk)
            );
            // The ceased token's allocator slot may already hold the new Merfolk.
            if return_last_merfolk {
                assert_eq!(g.object_view(token).zone, Zone::Outside);
                assert_eq!(g.object_view(core).zone, Zone::Hand);
            }
        }
        let mut g = setup(p);
        let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        spell(&mut g, p, "Wanderwine Farewell", &[Target::Card(bear)]);
        settle(&mut g);
        assert!(farewell_tokens(&g, p).is_empty());
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
    }
}

#[test]
fn wanderwine_farewell_partial_target_failure_counts_only_actual_return_and_all_invalid_fizzles() {
    for all_invalid in [false, true] {
        let mut g = setup(Player::P0);
        spawn(
            &mut g,
            Player::P0,
            Card::from_name("Firdoch Core").unwrap(),
            Zone::Battlefield,
        );
        let a = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
        let b = spawn(&mut g, Player::P1, Card::HillGiant, Zone::Battlefield);
        spell(
            &mut g,
            Player::P0,
            "Wanderwine Farewell",
            &[Target::Card(a), Target::Card(b)],
        );
        g.move_object(a, Zone::Exile);
        g.move_object(a, Zone::Battlefield);
        if all_invalid {
            g.move_object(b, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(
            farewell_tokens(&g, Player::P0).len(),
            usize::from(!all_invalid)
        );
        assert_eq!(
            g.object_view(b).zone,
            if all_invalid {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
    }
}

#[test]
fn wanderwine_farewell_convoke_uses_distinct_sick_creatures_and_pays_colored_and_generic_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = spawn(
            &mut g,
            p,
            Card::from_name("Brineborn Cutthroat").unwrap(),
            Zone::Battlefield,
        );
        let b = spawn(
            &mut g,
            p,
            Card::from_name("Brineborn Cutthroat").unwrap(),
            Zone::Battlefield,
        );
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        for id in [a, b, bear] {
            g.state.objects[id as usize].summoning_sick = true;
        }
        let target = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        let source = spawn(
            &mut g,
            p,
            Card::from_name("Wanderwine Farewell").unwrap(),
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: source });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        checked(&mut g, Action::FinishTargets);
        checked(&mut g, Action::BeginConvoke);
        for (id, contribution) in [
            (a, ConvokeContribution::Blue),
            (b, ConvokeContribution::Blue),
            (bear, ConvokeContribution::Generic),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
            reject_atomic(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!([a, b, bear].iter().all(|&id| g.object_view(id).tapped));
        assert_eq!(farewell_tokens(&g, p).len(), 1);
    }
}

#[test]
fn sunderflock_greatest_elemental_mana_value_reduces_only_generic_and_includes_kindred_permanents()
{
    for p in [Player::P0, Player::P1] {
        for name in [None, Some("Firdoch Core"), Some("Sunderflock")] {
            let mut g = setup(p);
            let reduction = name.map_or(0, |name| {
                let card = Card::from_name(name).unwrap();
                let id = spawn(&mut g, p, card, Zone::Battlefield);
                assert!(g.has_subtype(id, "Elemental"));
                // Keep the provider available for reduction, but not as extra mana.
                g.state.objects[id as usize].tapped = true;
                card.fixed_mana_value()
            });
            // An opposing nine-mana Elemental is not part of your reduction.
            spawn(
                &mut g,
                p.other(),
                Card::from_name("Sunderflock").unwrap(),
                Zone::Battlefield,
            );
            let id = spawn(
                &mut g,
                p,
                Card::from_name("Sunderflock").unwrap(),
                Zone::Hand,
            );
            let generic = 7u32.saturating_sub(u32::try_from(reduction).unwrap());
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, generic];
            reject_atomic(&mut g, Action::AnnounceCast { card: id });
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, generic];
            checked(&mut g, Action::AnnounceCast { card: id });
            finish_cast_choices(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 5,
                    toughness: 5
                }
            );
            assert!(g.keywords(id).contains(Keywords::FLYING));
        }
    }
}

#[test]
fn sunderflock_cast_trigger_returns_current_non_elemental_creatures_simultaneously_to_owners() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        let object = g.object_ref(enemy);
        g.change_controller(object, p);
        g.grant_keyword_effects(
            &[enemy],
            object,
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
        );
        let changeling = spawn(
            &mut g,
            p.other(),
            Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap(),
            Zone::Battlefield,
        );
        let core = spawn(
            &mut g,
            p,
            Card::from_name("Firdoch Core").unwrap(),
            Zone::Battlefield,
        );
        let source = spell(&mut g, p, "Sunderflock", &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let late = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        for id in [bear, enemy, late] {
            assert_eq!(g.object_view(id).zone, Zone::Hand);
        }
        assert_eq!(g.object_view(enemy).owner, p.other());
        for id in [changeling, core] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn sunderflock_free_cast_is_still_cast_but_direct_entry_blink_and_spell_copy_are_not() {
    for p in [Player::P0, Player::P1] {
        for route in 0..4 {
            let mut g = setup(p);
            let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            let source = if route == 0 {
                spawn(
                    &mut g,
                    p,
                    Card::from_name("Omniscience").unwrap(),
                    Zone::Battlefield,
                );
                let source = spawn(
                    &mut g,
                    p,
                    Card::from_name("Sunderflock").unwrap(),
                    Zone::Hand,
                );
                g.state.players[p.index()].mana = [0; 6];
                checked(
                    &mut g,
                    Action::AnnounceFree {
                        card: source,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    },
                );
                finish_cast_choices(&mut g);
                source
            } else if route == 3 {
                let source = spell(&mut g, p, "Sunderflock", &[]);
                // Exact permanent-copy qualification, without inventing a new copy-spell consumer.
                g.state.stack.last_mut().unwrap().is_copy = true;
                source
            } else {
                let source = spawn(
                    &mut g,
                    p,
                    Card::from_name("Sunderflock").unwrap(),
                    Zone::Graveyard,
                );
                g.move_object(source, Zone::Battlefield);
                if route == 2 {
                    g.move_object(source, Zone::Exile);
                    g.move_object(source, Zone::Battlefield);
                }
                source
            };
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(bear).zone,
                if route == 0 {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn sunderflock_cost_funding_observes_current_elemental_types_and_restores_after_aura_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = spawn(
            &mut g,
            p,
            Card::from_name("Sunderflock").unwrap(),
            Zone::Battlefield,
        );
        let aura = spell(&mut g, p, "Noggle the Mind", &[Target::Card(provider)]);
        settle(&mut g);
        assert!(!g.has_subtype(provider, "Elemental"));
        let source = spawn(
            &mut g,
            p,
            Card::from_name("Sunderflock").unwrap(),
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 6];
        reject_atomic(&mut g, Action::AnnounceCast { card: source });
        g.move_object(aura, Zone::Graveyard);
        assert!(g.has_subtype(provider, "Elemental"));
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: source });
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(provider).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn wanderwine_farewell_waits_for_linked_return_entry_choice_before_checking_merfolk_and_uses_saved_count()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let automaton = spell(&mut g, p, "Adaptive Automaton", &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let elf = crate::cards::CreatureType::from_name("Elf").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: crate::EntryChoiceValue::CreatureType(elf),
            },
        );
        settle(&mut g);
        assert!(!g.has_subtype(automaton, "Merfolk"));
        g.state.active = p.other();
        g.state.priority = p.other();
        let light = cast_named(&mut g, p.other(), "Banishing Light");
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        g.state.active = p;
        g.state.priority = p;
        spell(&mut g, p, "Wanderwine Farewell", &[Target::Card(light)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        assert_eq!(g.state.resolving.as_ref().unwrap().moved_count, Some(1));
        assert!(farewell_tokens(&g, p).is_empty());
        let merfolk = crate::cards::CreatureType::from_name("Merfolk").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: crate::EntryChoiceValue::CreatureType(merfolk),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(light).zone, Zone::Hand);
        assert_eq!(g.object_view(automaton).zone, Zone::Battlefield);
        assert!(g.has_subtype(automaton, "Merfolk"));
        assert_eq!(farewell_tokens(&g, p).len(), 1);
    }
}

fn answer_tokens(g: &Game, p: Player) -> Vec<CardId> {
    let token = Card::from_name("@token/91416e259800bff2445a15f5").unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.controller == p && object.card == token)
                .then_some(id as CardId)
        })
        .collect()
}
fn take_test_priority(g: &mut Game, p: Player) {
    assert_eq!(g.state.pending, Pending::Priority);
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}

#[test]
fn glen_elendras_answer_counters_opposing_spell_trigger_and_source_independent_activation_but_keeps_own_spell()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let tome = spawn(
            &mut g,
            p.other(),
            Card::from_name("Mazemind Tome").unwrap(),
            Zone::Battlefield,
        );
        let own = spell(&mut g, p, "Shock", &[Target::Player(p.other())]);
        take_test_priority(&mut g, p.other());
        let glamer = spell(&mut g, p.other(), "Glamer Gifter", &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish_cast_choices(&mut g);
        take_test_priority(&mut g, p.other());
        let hostile = spell(&mut g, p.other(), "Shock", &[Target::Player(p)]);
        activate(&mut g, tome, 0, None);
        finish_cast_choices(&mut g);
        g.move_object(tome, Zone::Graveyard);
        let count = g
            .state
            .stack
            .iter()
            .filter(|item| item.controller == p.other())
            .count();
        assert_eq!(count, 3);
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Glen Elendra's Answer", &[]);
        settle(&mut g);
        assert_eq!(answer_tokens(&g, p).len(), 3);
        assert_eq!(g.object_view(hostile).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(glamer).zone, Zone::Battlefield);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        for token in answer_tokens(&g, p) {
            assert_eq!(
                g.stats(token),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert_eq!(g.object_view(token).colors, 6);
            assert!(g.has_subtype(token, "Faerie"));
            assert!(g.keywords(token).contains(Keywords::FLYING));
        }
    }
}

#[test]
fn glen_elendras_answer_counters_all_permanent_spells_in_one_graveyard_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        // Illusion Spinners has conditional flash while you control a Faerie.
        spawn(
            &mut g,
            p.other(),
            Card::from_name("@token/91416e259800bff2445a15f5").unwrap(),
            Zone::Battlefield,
        );
        let moon = spawn(
            &mut g,
            p.other(),
            Card::from_name("Moonshadow").unwrap(),
            Zone::Battlefield,
        );
        g.set_counters(moon, Counters::of(CounterKind::MinusOne, 6));
        let a = spell(&mut g, p.other(), "Glamer Gifter", &[]);
        let b = spell(&mut g, p.other(), "Illusion Spinners", &[]);
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Glen Elendra's Answer", &[]);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(moon, CounterKind::MinusOne), 5);
        assert_eq!(answer_tokens(&g, p).len(), 2);
    }
}

#[test]
fn glen_elendras_answer_cannot_be_countered_and_does_not_count_uncounterable_victims_or_already_resolved_counters()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let sphinx = spell(&mut g, p.other(), "Sphinx of the Final Word", &[]);
        let shock = spell(&mut g, p.other(), "Shock", &[Target::Player(p)]);
        take_test_priority(&mut g, p);
        let answer = spell(&mut g, p, "Glen Elendra's Answer", &[]);
        take_test_priority(&mut g, p.other());
        let negate = spell(&mut g, p.other(), "Negate", &[Target::Spell(answer)]);
        settle(&mut g);
        assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(g.object_view(negate).zone, Zone::Graveyard);
        assert_eq!(g.object_view(answer).zone, Zone::Graveyard);
        assert_eq!(answer_tokens(&g, p).len(), 1);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}

#[test]
fn glen_elendras_answer_shared_uncounterable_spell_prohibition_does_not_protect_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        spawn(
            &mut g,
            p.other(),
            Card::from_name("Sphinx of the Final Word").unwrap(),
            Zone::Battlefield,
        );
        let tome = spawn(
            &mut g,
            p.other(),
            Card::from_name("Mazemind Tome").unwrap(),
            Zone::Battlefield,
        );
        let shock = spell(&mut g, p.other(), "Shock", &[Target::Player(p)]);
        activate(&mut g, tome, 0, None);
        finish_cast_choices(&mut g);
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Glen Elendra's Answer", &[]);
        settle(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(answer_tokens(&g, p).len(), 1);
    }
}

#[test]
fn glen_elendras_answer_counts_countered_spell_copies_that_cease_and_redirected_counter_destinations()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let teach = spawn(
            &mut g,
            p.other(),
            Card::from_name("Teach by Example").unwrap(),
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceCast { card: teach });
        let route = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .unwrap();
        checked(&mut g, route);
        finish_cast_choices(&mut g);
        settle(&mut g);
        let original = spell(&mut g, p.other(), "Shock", &[Target::Player(p)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::FinishStackTargets);
        let copy = g
            .state
            .stack
            .iter()
            .find(|item| item.is_copy)
            .unwrap()
            .clone();
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Glen Elendra's Answer", &[]);
        settle(&mut g);
        assert_eq!(g.object_view(original).zone, Zone::Graveyard);
        assert!(!g.state.stack.iter().any(|item| item.id == copy.id));
        assert_ne!(g.object_ref(copy.card), copy.object);
        assert_eq!(answer_tokens(&g, p).len(), 2);
        assert_eq!(g.state.players[p.index()].life, 20);

        let mut g = setup(p.other());
        let progenitus = spell(&mut g, p.other(), "Progenitus", &[]);
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Glen Elendra's Answer", &[]);
        settle(&mut g);
        assert_eq!(g.object_view(progenitus).zone, Zone::Library);
        assert!(
            g.state.players[p.other().index()]
                .library
                .contains(&progenitus)
        );
        assert_eq!(answer_tokens(&g, p).len(), 1);
    }
}

#[test]
fn glen_elendras_answer_with_no_opposing_stack_objects_has_no_token_and_exact_two_generic_two_blue_cost()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        spell(&mut g, p, "Glen Elendra's Answer", &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(answer_tokens(&g, p).is_empty());
    }
}

fn ajani(g: &mut Game, p: Player) -> CardId {
    let id = spell(g, p, "Ajani, Outland Chaperone", &[]);
    settle(g);
    assert_eq!(g.counter_count(id, CounterKind::Loyalty), 3);
    id
}
fn announce_loyalty(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn ajani_outland_chaperone_paid_body_plus_one_real_kithkin_and_loyalty_timing_limits() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 1];
        let source = ajani(&mut g, p);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        g.tap_objects(&[bear]);
        assert!(g.object_view(source).summoning_sick);
        announce_loyalty(&mut g, source, 0);
        finish_cast_choices(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Loyalty), 4);
        settle(&mut g);
        let tokens = kithkin_tokens(&g, p);
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            g.stats(tokens[0]),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.object_view(tokens[0]).colors, 17);
        assert!(g.has_subtype(tokens[0], "Kithkin"));
        for ability in [0, 1] {
            reject_atomic(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability,
                    counter_payment: Counters::default(),
                },
            );
        }
        super::ability_timeline_tests::next_turn(&mut g);
        take_test_priority(&mut g, p);
        reject_atomic(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        super::ability_timeline_tests::next_turn(&mut g);
        for _ in 0..8 {
            if g.state.step == Step::Main1 {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert_eq!(g.state.step, Step::Main1);
        announce_loyalty(&mut g, source, 0);
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Loyalty), 5);
        assert_eq!(kithkin_tokens(&g, p).len(), 2);
    }
}

#[test]
fn ajani_outland_chaperone_minus_two_restricts_tapped_creatures_and_keeps_source_damage_after_departure()
 {
    for p in [Player::P0, Player::P1] {
        for invalidate in 0..3 {
            let mut g = setup(p);
            let source = ajani(&mut g, p);
            let target = spawn(
                &mut g,
                p.other(),
                Card::from_name("Sunderflock").unwrap(),
                Zone::Battlefield,
            );
            let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let land = spawn(&mut g, p.other(), Card::Island, Zone::Battlefield);
            g.tap_objects(&[target, land]);
            announce_loyalty(&mut g, source, 1);
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish_cast_choices(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::Loyalty), 1);
            g.move_object(source, Zone::Graveyard);
            if invalidate == 1 {
                g.state.objects[target as usize].tapped = false;
            }
            if invalidate == 2 {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
                g.tap_objects(&[target]);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).damage,
                if invalidate == 0 { 4 } else { 0 }
            );
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        }
        let mut g = setup(p);
        let source = ajani(&mut g, p);
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.tap_objects(&[own]);
        announce_loyalty(&mut g, source, 1);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
    }
}

fn ajani_ultimate_choice(g: &mut Game, p: Player, source: CardId) {
    g.set_counters(source, Counters::of(CounterKind::Loyalty, 8));
    announce_loyalty(g, source, 2);
    finish_cast_choices(g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(
        g.state.pending,
        Pending::EffectLibrarySelection { .. }
    ));
    assert_eq!(g.actor(), Some(p));
}

#[test]
fn ajani_outland_chaperone_ultimate_looks_at_current_life_privately_and_enters_any_eligible_nonland_permanents_then_shuffles()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = ajani(&mut g, p);
        let enchanted = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let picks: Vec<_> = [
            "Bear Cub",
            "Mazemind Tome",
            "Pacifism",
            "Ajani, Caller of the Pride",
        ]
        .iter()
        .map(|name| spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Library))
        .collect();
        let big = spawn(&mut g, p, Card::HillGiant, Zone::Library);
        let land = spawn(&mut g, p, Card::Island, Zone::Library);
        g.state.players[p.index()].life = 1;
        g.set_counters(source, Counters::of(CounterKind::Loyalty, 7));
        reject_atomic(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        g.set_counters(source, Counters::of(CounterKind::Loyalty, 8));
        announce_loyalty(&mut g, source, 2);
        finish_cast_choices(&mut g);
        g.state.players[p.index()].life = 6;
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let view = g.observe(p).library_selection.unwrap();
        assert_eq!(view.remaining.len(), 6);
        assert_eq!(view.maximum, 4);
        assert_eq!(view.minimum, 0);
        assert!(!view.public);
        assert!(g.observe(p.other()).library_selection.is_none());
        reject_atomic(&mut g, Action::SelectLibrary { card: big });
        reject_atomic(&mut g, Action::SelectLibrary { card: land });
        for &id in &picks {
            checked(&mut g, Action::SelectLibrary { card: id });
            reject_atomic(&mut g, Action::SelectLibrary { card: id });
        }
        let rng = g.state.rng;
        checked(&mut g, Action::FinishLibrarySelection);
        assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
        let object = g.object_ref(enchanted);
        checked(&mut g, Action::ChooseAuraAttachment { object });
        settle(&mut g);
        assert_ne!(g.state.rng, rng);
        for &id in &picks {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).controller, p);
            assert!(!g.object_view(id).tapped);
        }
        assert_eq!(g.object_view(picks[2]).attached_to, Some(object));
        assert_eq!(g.counter_count(picks[3], CounterKind::Loyalty), 4);
        for id in [big, land] {
            assert_eq!(g.object_view(id).zone, Zone::Library);
        }
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(g.public_library_top(p).is_none());
    }
}

#[test]
fn ajani_outland_chaperone_ultimate_may_choose_nothing_even_without_eligible_cards_and_always_shuffles()
 {
    for p in [Player::P0, Player::P1] {
        for eligible in [false, true] {
            let mut g = setup(p);
            let source = ajani(&mut g, p);
            let id = spawn(
                &mut g,
                p,
                if eligible {
                    Card::GrizzlyBears
                } else {
                    Card::Island
                },
                Zone::Library,
            );
            g.state.players[p.index()].life = 100;
            ajani_ultimate_choice(&mut g, p, source);
            assert!(g.observe(p.other()).library_selection.is_none());
            let before = g.state.players[p.index()].library.clone();
            let rng = g.state.rng;
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            let mut actual = g.state.players[p.index()].library.clone();
            let mut expected = before;
            actual.sort();
            expected.sort();
            assert_eq!(actual, expected);
            assert_eq!(g.object_view(id).zone, Zone::Library);
            assert_ne!(g.state.rng, rng);
        }
    }
}

#[test]
fn ajani_outland_chaperone_zero_life_look_still_shuffles_and_countered_ultimate_preserves_library()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = ajani(&mut g, p);
        spawn(
            &mut g,
            p,
            Card::from_name("Herald of Eternal Dawn").unwrap(),
            Zone::Battlefield,
        );
        g.state.players[p.index()].life = 0;
        g.set_counters(source, Counters::of(CounterKind::Loyalty, 8));
        let before = g.state.players[p.index()].library.clone();
        let rng = g.state.rng;
        let top = g.object_ref(*before.last().unwrap());
        g.remember_library_top(p, top);
        announce_loyalty(&mut g, source, 2);
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_ne!(g.state.rng, rng);
        assert_eq!(g.state.players[p.index()].library.len(), before.len());
        assert!(g.public_library_top(p).is_none());
        assert!(g.observe(p).library_selection.is_none());

        let mut g = setup(p);
        let source = ajani(&mut g, p);
        g.set_counters(source, Counters::of(CounterKind::Loyalty, 8));
        announce_loyalty(&mut g, source, 2);
        finish_cast_choices(&mut g);
        let id = g.state.stack.last().unwrap().id;
        let before = g.state.players[p.index()].library.clone();
        let rng = g.state.rng;
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library, before);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

fn winnowing_choice(g: &mut Game, p: Player) {
    spell(g, p, "Winnowing", &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
    assert_eq!(g.actor(), Some(p));
    assert_eq!(
        g.observe(p).effect_choice.as_ref().unwrap().kind,
        crate::EffectChoiceKind::KeepSharedCreatureTypes
    );
}
fn winnowing_keep(g: &mut Game, p: Player, id: CardId) {
    assert_eq!(g.actor(), Some(p));
    let object = g.object_ref(id);
    checked(g, Action::ChooseCreature { object });
}

#[test]
fn winnowing_controller_chooses_for_both_players_without_targeting_and_uses_current_controllers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let anchor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let same = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        let sacrifice = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        let object = g.object_ref(sacrifice);
        g.change_controller(object, p);
        g.grant_keyword_effects(&[sacrifice], object, Keywords::INDESTRUCTIBLE);
        let enemy = spawn(
            &mut g,
            p,
            Card::from_name("Tolarian Terror").unwrap(),
            Zone::Battlefield,
        );
        let object = g.object_ref(enemy);
        g.change_controller(object, p.other());
        g.grant_keyword_effects(&[enemy], object, Keywords::HEXPROOF);
        let other = spawn(
            &mut g,
            p.other(),
            Card::from_name("Llanowar Elves").unwrap(),
            Zone::Battlefield,
        );
        let core = spawn(
            &mut g,
            p,
            Card::from_name("Firdoch Core").unwrap(),
            Zone::Battlefield,
        );
        let land = spawn(&mut g, p, Card::Plains, Zone::Battlefield);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 4];
        winnowing_choice(&mut g, p);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.observe(p.other()).effect_choice.is_some());
        for id in [core, land, enemy] {
            let object = g.object_ref(id);
            reject_atomic(&mut g, Action::ChooseCreature { object });
        }
        reject_atomic(&mut g, Action::DeclineEffectPayment);
        winnowing_keep(&mut g, p, anchor);
        let object = g.object_ref(anchor);
        reject_atomic(&mut g, Action::ChooseCreature { object });
        assert_eq!(g.actor(), Some(p));
        winnowing_keep(&mut g, p, enemy);
        settle(&mut g);
        for id in [anchor, same, enemy, core, land] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
        for id in [sacrifice, other] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.object_view(sacrifice).owner, p.other());
        assert_eq!(g.object_view(enemy).controller, p.other());
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn winnowing_skips_empty_players_and_commits_all_sacrifices_in_one_graveyard_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let moon = spawn(
            &mut g,
            p.other(),
            Card::from_name("Moonshadow").unwrap(),
            Zone::Battlefield,
        );
        g.set_counters(moon, Counters::of(CounterKind::MinusOne, 6));
        let a = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let b = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        winnowing_choice(&mut g, p);
        assert_eq!(
            g.observe(p).effect_choice.as_ref().unwrap().required[p.index()],
            0
        );
        winnowing_keep(&mut g, p, moon);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(moon, CounterKind::MinusOne), 5);

        let mut g = setup(p);
        let source = spell(&mut g, p, "Winnowing", &[]);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(g.observe(p).effect_choice.is_none());
    }
}

fn crew_winnowing_vehicle(g: &mut Game, p: Player, vehicle: CardId, contributor: CardId) {
    let ability = g.state.objects[vehicle as usize]
        .card
        .activations()
        .iter()
        .position(|a| a.crew_minimum().is_some())
        .unwrap() as u16;
    checked(
        g,
        Action::AnnounceActivate {
            card: vehicle,
            ability,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(contributor);
    checked(g, Action::ChooseCrewer { object });
    checked(g, Action::FinishCrew);
    settle(g);
    assert!(g.is_creature(vehicle));
    assert!(g.has_subtype(vehicle, "Vehicle"));
    assert!(
        crate::cards::CREATURE_TYPES
            .iter()
            .all(|kind| !g.has_subtype(vehicle, kind))
    );
    assert_eq!(g.object_view(vehicle).controller, p);
}

#[test]
fn winnowing_typeless_vehicle_anchor_keeps_itself_but_neither_other_vehicles_nor_changelings_share_its_type()
 {
    for p in [Player::P0, Player::P1] {
        for choose_changeling in [false, true] {
            let mut g = setup(p);
            let a = spawn(
                &mut g,
                p,
                Card::from_name("Cultivator's Caravan").unwrap(),
                Zone::Battlefield,
            );
            let b = spawn(
                &mut g,
                p,
                Card::from_name("Cultivator's Caravan").unwrap(),
                Zone::Battlefield,
            );
            let first = spawn(&mut g, p, Card::HillGiant, Zone::Battlefield);
            let second = spawn(&mut g, p, Card::HillGiant, Zone::Battlefield);
            crew_winnowing_vehicle(&mut g, p, a, first);
            crew_winnowing_vehicle(&mut g, p, b, second);
            let changeling = spawn(
                &mut g,
                p,
                Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap(),
                Zone::Battlefield,
            );
            winnowing_choice(&mut g, p);
            winnowing_keep(&mut g, p, if choose_changeling { changeling } else { a });
            settle(&mut g);
            assert_eq!(
                g.object_view(a).zone,
                if choose_changeling {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(b).zone, Zone::Graveyard);
            for id in [first, second] {
                assert_eq!(
                    g.object_view(id).zone,
                    if choose_changeling {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            if choose_changeling {
                assert_eq!(g.object_view(changeling).zone, Zone::Battlefield);
            } else {
                assert_ne!(g.object_view(changeling).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn winnowing_uses_effective_creature_types_and_exact_incarnations_in_public_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let anchor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let other = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        spell(&mut g, p, "Noggle the Mind", &[Target::Card(anchor)]);
        settle(&mut g);
        assert!(!g.has_subtype(anchor, "Bear"));
        assert!(g.has_subtype(anchor, "Noggle"));
        let stale = g.object_ref(other);
        g.move_object(other, Zone::Exile);
        g.move_object(other, Zone::Battlefield);
        winnowing_choice(&mut g, p);
        reject_atomic(&mut g, Action::ChooseCreature { object: stale });
        winnowing_keep(&mut g, p, anchor);
        settle(&mut g);
        assert_eq!(g.object_view(anchor).zone, Zone::Battlefield);
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    }
}

#[test]
fn winnowing_convoke_pays_white_and_generic_with_distinct_sick_creatures_and_then_chooses_tapped_anchor()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap();
        let a = spawn(&mut g, p, token, Zone::Battlefield);
        let b = spawn(&mut g, p, token, Zone::Battlefield);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        for id in [a, b, bear] {
            g.state.objects[id as usize].summoning_sick = true;
        }
        let source = spawn(&mut g, p, Card::from_name("Winnowing").unwrap(), Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(&mut g, Action::AnnounceCast { card: source });
        checked(&mut g, Action::BeginConvoke);
        for (id, contribution) in [
            (a, ConvokeContribution::White),
            (b, ConvokeContribution::White),
            (bear, ConvokeContribution::Generic),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
            reject_atomic(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        finish_cast_choices(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!([a, b, bear].iter().all(|&id| g.object_view(id).tapped));
        winnowing_keep(&mut g, p, a);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}

#[test]
fn winnowing_simultaneous_sacrifice_preserves_granted_persist_after_grantor_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let anchor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let victim = spawn(&mut g, p, Card::HillGiant, Zone::Battlefield);
        let rhys = rhys_grant(&mut g, p, victim);
        let before = g.object_ref(victim);
        g.change_controller(before, p.other());
        let enemy_anchor = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        winnowing_choice(&mut g, p);
        winnowing_keep(&mut g, p, anchor);
        winnowing_keep(&mut g, p, enemy_anchor);
        settle(&mut g);
        assert_eq!(g.object_view(rhys).zone, Zone::Graveyard);
        let returned = g.object_view(victim);
        assert_eq!(returned.zone, Zone::Battlefield);
        assert_eq!(returned.controller, p);
        assert_ne!(returned.reference, before);
        assert_eq!(returned.counters.get(CounterKind::MinusOne), 1);
        assert_eq!(g.stats(victim), Stats::from(2));
        assert!(!returned.tapped);
        assert_eq!(g.object_view(anchor).zone, Zone::Battlefield);
        assert_eq!(g.object_view(enemy_anchor).zone, Zone::Battlefield);
    }
}
