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
