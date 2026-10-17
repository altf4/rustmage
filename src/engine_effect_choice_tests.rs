use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn permanent(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Battlefield)
}
fn enter(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.prepare_triggers();
    id
}
fn sacrifice(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::Sacrifice { object });
}
fn replay(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    assert_eq!(g.snapshot(), fork.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn rejected(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision;
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, rev);
}
fn factory(name: &str) -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == name))
        .unwrap()
}
fn fund(g: &mut Game, id: CardId) {
    let (mut mana, n) = g.state.objects[id as usize].card.cost();
    mana[5] += n;
    g.state.players[0].mana = mana;
}
const ANY: SacrificeFilter = SacrificeFilter {
    artifact_enchantment_or_token: false,
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    subtype: None,
};
// Synthetic programs exercise IR combinations that are not yet complete card admissions.
fn program(g: &mut Game, code: &'static [Instruction], targets: Vec<BoundTarget>) {
    let id = named(g, Player::P0, "Shock", Zone::Hand);
    let mut item = g.cast_item(Player::P0, id, [0; 6]);
    item.targets = targets;
    g.state.resolving = Some(Resolution {
        explore: None,
        evidence: None,
        connive: None,
        item,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        choice: None,
    });
    g.run_program();
}

#[test]
fn each_player_sacrifices_in_public_apnap_order_then_every_selected_object_leaves_together() {
    let mut g = scenario();
    g.state.active = Player::P1;
    g.state.priority = Player::P0;
    let a = permanent(&mut g, Player::P1, "Grizzly Bears");
    let b = permanent(&mut g, Player::P1, "Agents of HYDRA");
    let c = permanent(&mut g, Player::P0, "Grizzly Bears");
    let source = enter(&mut g, "Abyssal Gorestalker");
    pass_pair(&mut g);
    assert_eq!(g.actor(), Some(Player::P1));
    rejected(&mut g, Action::DeclineEffectPayment);
    rejected(&mut g, Action::Pass);
    let wrong = g.object_ref(c);
    rejected(&mut g, Action::Sacrifice { object: wrong });
    let old = g.object_ref(a);
    rejected(
        &mut g,
        Action::Sacrifice {
            object: ObjectRef {
                generation: old.generation + 1,
                ..old
            },
        },
    );
    rejected(
        &mut g,
        Action::Sacrifice {
            object: ObjectRef {
                card: u32::MAX,
                generation: 0,
            },
        },
    );
    replay(&mut g, Action::Sacrifice { object: old });
    rejected(&mut g, Action::Sacrifice { object: old });
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    sacrifice(&mut g, b);
    assert_eq!(g.actor(), Some(Player::P0));
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            g.observe(viewer).effect_choice.unwrap().selected[1].len(),
            2
        );
    }
    assert!(
        [a, b, c, source]
            .iter()
            .all(|&id| g.object_view(id).zone == Zone::Battlefield)
    );
    assert!(g.state.pending_triggers.is_empty());
    let before = g.snapshot();
    let bytes = g.state_bytes();
    let mut fork = g.clone();
    for branch in [&mut g, &mut fork] {
        sacrifice(branch, c);
        sacrifice(branch, source);
    }
    assert_eq!(g.snapshot(), fork.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    sacrifice(&mut g, c);
    sacrifice(&mut g, source);
    assert_eq!(g.snapshot(), after);
    assert!(bytes > std::mem::size_of::<State>());
    for id in [a, b, c, source] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert!(g.observe(Player::P0).effect_choice.is_none());
    assert_eq!(g.state.stack.len(), 1); // The sacrificed Agents' death trigger waits until resolution ends.
    pass_pair(&mut g);
    assert!(g.state.objects.iter().any(|o| {
        o.zone == Zone::Battlefield
            && o.card
                .token_info()
                .is_some_and(|i| i.name == "Villain Token")
    }));
}

#[test]
fn mandatory_sacrifice_does_as_much_as_possible_including_its_source() {
    let mut g = scenario();
    let source = enter(&mut g, "Abyssal Gorestalker");
    pass_pair(&mut g);
    let choice = g.observe(Player::P0).effect_choice.unwrap();
    assert_eq!(choice.required, [1, 0]);
    sacrifice(&mut g, source);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert!(g.state.resolving.is_none());
}

#[test]
fn modal_edict_is_not_targeting_and_uses_current_flying_including_gained_keywords() {
    let mut g = scenario();
    let protected = permanent(&mut g, Player::P1, "Grizzly Bears");
    g.state.objects[protected as usize].gained_keywords = Keywords::FLYING
        .union(Keywords::HEXPROOF)
        .union(Keywords::INDESTRUCTIBLE);
    let land = spawn(&mut g, Player::P1, Card::Forest, Zone::Battlefield);
    let own = permanent(&mut g, Player::P0, "Shivan Dragon");
    let spell = named(&mut g, Player::P0, "Pick Your Poison", Zone::Hand);
    fund(&mut g, spell);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseMode { mode: 2 });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(&mut g, pay);
    pass_pair(&mut g);
    assert_eq!(g.actor(), Some(Player::P1));
    let objects: Vec<_> = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| {
            if let Action::Sacrifice { object } = a {
                Some(object.card)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(objects, vec![protected]);
    for id in [land, own] {
        let object = g.object_ref(id);
        rejected(&mut g, Action::Sacrifice { object });
    }
    sacrifice(&mut g, protected);
    assert_eq!(g.object_view(protected).zone, Zone::Graveyard);
}

#[test]
fn an_empty_edict_mode_is_legal_and_never_stalls_resolution() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Pick Your Poison", Zone::Hand);
    fund(&mut g, spell);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseMode { mode: 0 });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(&mut g, pay);
    pass_pair(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert!(g.state.resolving.is_none());
}

#[test]
fn optional_artifact_payment_can_be_declined_and_paid_after_source_departure() {
    let mut g = scenario();
    let artifact = permanent(&mut g, Player::P0, "Soul-Guide Lantern");
    let source = enter(&mut g, "Benthic Criminologists");
    g.move_object(source, Zone::Hand);
    pass_pair(&mut g);
    assert_eq!(
        g.observe(Player::P0).effect_choice.unwrap().kind,
        EffectChoiceKind::Sacrifice
    );
    let before = g.snapshot();
    let count = g.hand_count(Player::P0);
    replay(&mut g, Action::DeclineEffectPayment);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(g.hand_count(Player::P0), count);
    g.restore(&before).unwrap();
    let object = g.object_ref(artifact);
    replay(&mut g, Action::Sacrifice { object });
    assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(Player::P0), count + 1);
}

#[test]
fn discard_payment_cannot_discard_a_stack_source_and_is_private_then_public_on_commit() {
    let mut g = scenario();
    let source = enter(&mut g, "Discerning Peddler");
    let hand = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let enemy = named(&mut g, Player::P1, "Mountain", Zone::Hand);
    pass_pair(&mut g);
    assert!(g.observe(Player::P1).effect_choice.is_none());
    assert!(g.observe(Player::P1).actions.is_empty());
    let choice = g.observe(Player::P0).effect_choice.unwrap();
    assert_eq!(choice.kind, EffectChoiceKind::Discard);
    assert_eq!(choice.candidates, vec![g.object_ref(hand)]);
    rejected(&mut g, Action::Discard { card: source });
    rejected(&mut g, Action::Discard { card: enemy });
    let before = g.snapshot();
    let library = g.state.players[0].library.len();
    replay(&mut g, Action::DeclineEffectPayment);
    assert_eq!(g.state.players[0].library.len(), library);
    assert_eq!(g.object_view(hand).zone, Zone::Hand);
    g.restore(&before).unwrap();
    replay(&mut g, Action::Discard { card: hand });
    assert_eq!(g.state.players[0].library.len(), library - 1);
    assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == hand && o.zone == Zone::Graveyard)
    );
}

#[test]
fn insufficient_optional_discard_never_partially_pays_or_draws_and_partial_selection_can_decline() {
    const COST: &[Instruction] = &[
        Instruction::ChooseObjects {
            kind: EffectChoiceKind::Discard,
            players: PlayerSet::Controller,
            filter: ANY,
            amount: 2,
            optional_skip: Some(1),
        },
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    let mut g = scenario();
    let first = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let library = g.state.players[0].library.len();
    program(&mut g, COST, vec![]);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(first).zone, Zone::Hand);
    assert_eq!(g.state.players[0].library.len(), library);
    let second = named(&mut g, Player::P0, "Mountain", Zone::Hand);
    program(&mut g, COST, vec![]);
    replay(&mut g, Action::Discard { card: first });
    assert_eq!(g.object_view(first).zone, Zone::Hand);
    assert!(g.observe(Player::P1).effect_choice.is_none());
    let saved = g.snapshot();
    replay(&mut g, Action::DeclineEffectPayment);
    assert_eq!(g.object_view(first).zone, Zone::Hand);
    assert_eq!(g.object_view(second).zone, Zone::Hand);
    g.restore(&saved).unwrap();
    replay(&mut g, Action::Discard { card: second });
    assert_eq!(g.object_view(first).zone, Zone::Graveyard);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(Player::P0), 1);
}

#[test]
fn another_payment_excludes_only_the_bound_source_and_accepts_owned_artifact_creatures() {
    let mut g = scenario();
    let source = permanent(&mut g, Player::P0, "Beetle-Headed Merchants");
    let enemy = permanent(&mut g, Player::P1, "Sundial, Dawn Tyrant");
    let artifact = permanent(&mut g, Player::P0, "Sundial, Dawn Tyrant");
    g.attack_triggers(&g.object_view(source));
    g.prepare_triggers();
    pass_pair(&mut g);
    let object = g.object_ref(source);
    rejected(&mut g, Action::Sacrifice { object });
    let object = g.object_ref(enemy);
    rejected(&mut g, Action::Sacrifice { object });
    let library = g.state.players[0].library.len();
    sacrifice(&mut g, artifact);
    assert_eq!(g.object_view(source).counters.plus_one, 1);
    assert_eq!(g.state.players[0].library.len(), library - 1);
}

#[test]
fn springbloom_land_payment_precedes_search_and_can_replay_both_continuations() {
    let mut g = scenario();
    let land = spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
    enter(&mut g, "Springbloom Druid");
    pass_pair(&mut g);
    let before = g.snapshot();
    sacrifice(&mut g, land);
    assert_eq!(g.object_view(land).zone, Zone::Graveyard);
    assert_eq!(g.observe(Player::P0).decision, "library_search");
    let search = g.snapshot();
    let card = g
        .legal_actions()
        .into_iter()
        .find_map(|a| {
            if let Action::ChooseSearch { card } = a {
                Some(card)
            } else {
                None
            }
        })
        .unwrap();
    replay(&mut g, Action::ChooseSearch { card });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    assert!(g.object_view(card).tapped);
    let expected = g.snapshot();
    g.restore(&search).unwrap();
    act(&mut g, Action::ChooseSearch { card });
    act(&mut g, Action::FinishSearch);
    assert_eq!(g.snapshot(), expected);
    g.restore(&before).unwrap();
    act(&mut g, Action::DeclineEffectPayment);
    assert_eq!(g.object_view(land).zone, Zone::Battlefield);
    assert!(g.state.resolving.is_none());
}

#[test]
fn whole_spell_fizzle_skips_optional_payment_and_success_preserves_target_focus() {
    let mut g = scenario();
    let enemy = permanent(&mut g, Player::P1, "Shivan Dragon");
    let spell = named(&mut g, Player::P0, "Incinerating Blast", Zone::Hand);
    fund(&mut g, spell);
    let hand = named(&mut g, Player::P0, "Forest", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(enemy)));
    let before = g.snapshot();
    g.move_object(enemy, Zone::Hand);
    pass_pair(&mut g);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(hand).zone, Zone::Hand);
    g.restore(&before).unwrap();
    pass_pair(&mut g);
    assert_eq!(g.object_view(enemy).zone, Zone::Battlefield); // SBAs wait for the pending discard cost.
    assert_eq!(g.object_view(enemy).damage, 6);
    replay(&mut g, Action::Discard { card: hand });
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
}

#[test]
fn targeted_player_sacrifice_choices_belong_to_that_player_not_the_caster() {
    const CODE: &[Instruction] = &[
        Instruction::ChooseObjects {
            kind: EffectChoiceKind::Sacrifice,
            players: PlayerSet::Targeted,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                kinds: 1,
                ..ANY
            },
            amount: 1,
            optional_skip: None,
        },
        Instruction::GainLife(PlayerSet::Controller, 2),
    ];
    let mut g = scenario();
    let own = permanent(&mut g, Player::P0, "Grizzly Bears");
    let other = permanent(&mut g, Player::P1, "Grizzly Bears");
    let target = g.bind_target(Target::Player(Player::P1));
    program(&mut g, CODE, vec![target]);
    assert_eq!(g.actor(), Some(Player::P1));
    let object = g.object_ref(own);
    rejected(&mut g, Action::Sacrifice { object });
    sacrifice(&mut g, other);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn sacrificed_tokens_cease_after_resolution_and_linked_returns_wait_for_the_whole_batch() {
    let mut g = scenario();
    let own = permanent(&mut g, Player::P0, "Grizzly Bears");
    let token = spawn(&mut g, Player::P1, factory("Dog Token"), Zone::Battlefield);
    let source = enter(&mut g, "Abyssal Gorestalker");
    let held = named(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(held),
        return_zone: Zone::Battlefield,
    });
    pass_pair(&mut g);
    sacrifice(&mut g, own);
    sacrifice(&mut g, source);
    assert_eq!(g.object_view(held).zone, Zone::Exile);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    sacrifice(&mut g, token);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    assert!(g.state.free_tokens.contains(&token));
    assert_eq!(g.object_view(held).zone, Zone::Battlefield);
    assert!(g.state.exile_links.is_empty());
}

#[test]
fn another_is_relative_to_the_original_source_incarnation_after_departure_and_return() {
    let mut g = scenario();
    let source = permanent(&mut g, Player::P0, "Beetle-Headed Merchants");
    let old = g.object_ref(source);
    g.attack_triggers(&g.object_view(source));
    g.prepare_triggers();
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), old);
    pass_pair(&mut g);
    let library = g.state.players[0].library.len();
    let object = g.object_ref(source);
    replay(&mut g, Action::Sacrifice { object });
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].library.len(), library - 1);
}

#[test]
fn self_payment_uses_exact_incarnation_and_damage_uses_departure_keywords() {
    const CODE: &[Instruction] = &[
        Instruction::ChooseObjects {
            kind: EffectChoiceKind::Sacrifice,
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                source_only: true,
                ..ANY
            },
            amount: 1,
            optional_skip: Some(1),
        },
        Instruction::DamagePlayers(PlayerSet::Opponent, 1),
    ];
    for returned in [false, true] {
        let mut g = scenario();
        let source = permanent(&mut g, Player::P0, "Benthic Criminologists");
        g.state.objects[source as usize].gained_keywords = Keywords::LIFELINK;
        g.queue_trigger(&g.object_view(source), 0);
        let item = g.state.pending_triggers.pop().unwrap();
        g.state.resolving = Some(Resolution {
            explore: None,
            evidence: None,
            connive: None,
            item,
            program: CODE,
            pc: 0,
            target_group: 0,
            moved_count: None,
            discards: [0; 2],
            mana_payment: None,
            forage: None,
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
            milled_selection: None,
            library: None,
            selection: None,
            target_library: None,
            search: None,
            choice: None,
        });
        if returned {
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
        }
        g.run_program();
        if returned {
            assert!(g.state.resolving.is_none());
            assert_eq!(g.state.players[1].life, 20);
        } else {
            let object = g.object_ref(source);
            replay(&mut g, Action::Sacrifice { object });
            assert_eq!(g.state.players[0].life, 21);
            assert_eq!(g.state.players[1].life, 19);
        }
    }
}
