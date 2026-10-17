//! Actual paid whole-pair programs, exact mill incarnations and private owned choices.
use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 6] = [
    "Bramble Familiar // Fetch Quest",
    "Most Decrepit Old Bird // Speak Secrets",
    "Picklock Prankster // Free the Fae",
    "Thranduil, Sindarin Liege // Silvan Rally",
    "Beluna Grandsquall // Seek Thrills",
    "Glamdring, Foe-hammer // Gleam of Death",
];
fn act(g: &mut Game, a: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap_or_else(|| panic!("illegal {a:?}, {:?}", g.observe(actor)));
    assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(saved, g.snapshot());
    let mut child = {
        let parent = g.clone();
        let c = parent.clone();
        drop(parent);
        c
    };
    child.apply_index(actor, rev, index).unwrap();
    g.apply(actor, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
    g.apply(actor, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn choose(g: &mut Game) {
    let actions = g.legal_actions();
    let a = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone()
    };
    act(g, a);
}
fn settle(g: &mut Game) {
    for _ in 0..150 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn priority(g: &mut Game, p: Player) {
    for _ in 0..30 {
        if g.actor() == Some(p) && g.state.pending == Pending::Priority {
            return;
        }
        choose(g);
    }
    panic!("priority");
}

fn choice(g: &mut Game, id: CardId) {
    act(g, Action::AnnounceAdventure { card: id });
    for _ in 0..80 {
        if matches!(g.state.pending, Pending::MilledSelection { .. }) {
            return;
        }
        choose(g);
    }
    panic!("no milling choice");
}
#[test]
fn six_paid_complete_pairs_resolve_and_recast_same_normal_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let n = [7, 4, 4, 4, 7, 6][i];
            let recovery = match i {
                0 | 3 => "Forest",
                4 => "Picklock Prankster // Free the Fae",
                _ => "Shock",
            };
            let ids: Vec<_> = (0..n)
                .map(|_| named(&mut g, p, recovery, Zone::Library))
                .collect();
            let old: Vec<_> = ids.iter().map(|&id| g.object_ref(id)).collect();
            let id = named(&mut g, p, name, Zone::Hand);
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            if i < 4 {
                choice(&mut g, id);
                let view = g.observe(p);
                let private = view.milled_choice.unwrap();
                assert_eq!(private.candidates.len(), n);
                assert!(g.observe(p.other()).milled_choice.is_none());
                assert!(g.observe(p.other()).actions.is_empty());
                if i < 3 {
                    reject(&mut g, Action::FinishMilledSelection);
                }
                let count = if i == 3 { 2 } else { 1 };
                for &target in ids.iter().take(count) {
                    {
                        let object = g.object_ref(target);
                        act(&mut g, Action::SelectMilled { object });
                    }
                }
                settle(&mut g);
                for (&target, prior) in ids.iter().zip(&old).take(count) {
                    assert_eq!(
                        g.state.objects[target as usize].zone,
                        if i == 0 {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    );
                    assert_eq!(g.object_ref(target).generation, prior.generation + 2);
                }
            } else {
                act(&mut g, Action::AnnounceAdventure { card: id });
                settle(&mut g);
                for &target in &ids {
                    assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
                }
            }
            assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < mana);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
            assert!(g.can_play_adventure(p, id));
            priority(&mut g, p);
            act(&mut g, Action::AnnounceExile { card: id });
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            assert!(!g.can_play_adventure(p, id));
        }
    }
}
#[test]
fn mandatory_filter_uses_normal_characteristics_and_exact_batch_not_old_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = named(&mut g, p, "Shock", Zone::Graveyard);
        let nonfaerie = named(&mut g, p, NAMES[0], Zone::Library);
        let faerie = named(&mut g, p, NAMES[2], Zone::Library);
        let instant = named(&mut g, p, "Shock", Zone::Library);
        named(&mut g, p, "Forest", Zone::Library);
        let source = named(&mut g, p, NAMES[2], Zone::Hand);
        choice(&mut g, source);
        let candidates = g.observe(p).milled_choice.unwrap().candidates;
        assert!(
            candidates.contains(&g.object_ref(faerie))
                && candidates.contains(&g.object_ref(instant))
        );
        for id in [old, nonfaerie] {
            {
                let object = g.object_ref(id);
                reject(&mut g, Action::SelectMilled { object });
            }
        }
        let previous_generation = g.object_ref(instant).generation - 1;
        reject(
            &mut g,
            Action::SelectMilled {
                object: ObjectRef {
                    card: instant,
                    generation: previous_generation,
                },
            },
        );
        {
            let object = g.object_ref(faerie);
            act(&mut g, Action::SelectMilled { object });
        }
        settle(&mut g);
        assert_eq!(g.state.objects[faerie as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[nonfaerie as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn optional_up_to_two_can_take_zero_or_one_and_cannot_repeat_a_card() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            let ids: Vec<_> = (0..4)
                .map(|_| named(&mut g, p, "Forest", Zone::Library))
                .collect();
            let source = named(&mut g, p, NAMES[3], Zone::Hand);
            choice(&mut g, source);
            for &id in ids.iter().take(count) {
                {
                    let object = g.object_ref(id);
                    act(&mut g, Action::SelectMilled { object });
                }
                if count == 1 {
                    {
                        let object = g.object_ref(id);
                        reject(&mut g, Action::SelectMilled { object });
                    }
                }
            }
            if count < 2 {
                act(&mut g, Action::FinishMilledSelection);
            }
            settle(&mut g);
            assert_eq!(
                ids.iter()
                    .filter(|&&id| g.state.objects[id as usize].zone == Zone::Hand)
                    .count(),
                count
            );
        }
    }
}

#[test]
fn bramble_aura_entry_is_non_targeted_and_impossible_entry_retains_actual_public_destination() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            for recipient in [false, true] {
                let mut g = setup(p);
                let target = recipient
                    .then(|| named(&mut g, p.other(), "Hulking Raptor", Zone::Battlefield));
                let aura = named(&mut g, p, "Frantic Strength", Zone::Library);
                for _ in 0..6 {
                    named(&mut g, p, "Forest", Zone::Library);
                }
                let source = named(&mut g, p, NAMES[0], Zone::Hand);
                choice(&mut g, source);
                if exile {
                    // Model the exact capture boundary of a public mill replacement.
                    // The engine currently admits no blanket graveyard-zone replacement.
                    g.move_object(aura, Zone::Exile);
                    let captured = g.object_ref(aura);
                    let milled = &mut g.state.resolving.as_mut().unwrap().milled;
                    let entry = milled.iter_mut().find(|r| r.card == aura).unwrap();
                    *entry = captured;
                }
                let before = g.object_ref(aura);
                act(&mut g, Action::SelectMilled { object: before });
                if let Some(target) = target {
                    assert_eq!(g.observe(p).decision, "aura_entry");
                    let object = g.object_ref(target);
                    act(&mut g, Action::ChooseAuraAttachment { object });
                    settle(&mut g);
                    assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
                    assert_eq!(g.state.objects[aura as usize].attached_to, Some(object));
                    assert_eq!(g.stats(target).power, 7);
                    assert!(g.state.stack.is_empty()); // Ward is not triggered by attachment choice.
                } else {
                    settle(&mut g);
                    assert_eq!(
                        g.state.objects[aura as usize].zone,
                        if exile { Zone::Exile } else { Zone::Graveyard }
                    );
                    assert_eq!(g.object_ref(aura), before);
                }
                assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
            }
        }
    }
}
fn base_cost(g: &Game, id: CardId) -> ManaCost {
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    ManaCost { colored, generic }
}
fn equip(g: &mut Game, equipment: CardId, creature: CardId) {
    let ability = g.state.objects[equipment as usize]
        .card
        .activations()
        .iter()
        .position(|a| a.program.contains(&Instruction::AttachSource))
        .unwrap();
    act(
        g,
        Action::Activate {
            card: equipment,
            ability: ability as u16,
            target: Some(Target::Card(creature)),
            payment: [0, 0, 0, 0, 0, 2],
            counter_payment: Counters::default(),
        },
    );
    settle(g);
}
#[test]
fn beluna_uses_actual_spell_face_and_glamdring_locks_layered_power_before_mana_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, NAMES[4], Zone::Battlefield);
        let spell = named(&mut g, p, "Beanstalk Wurm // Plant Beans", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        assert_eq!(
            g.reduced_spell_cost(p, spell, base_cost(&g, spell)).generic,
            3
        );
        assert_eq!(
            g.reduced_spell_cost(p, bear, base_cost(&g, bear)).generic,
            1
        );
        let artifact = named(&mut g, p, NAMES[5], Zone::Hand);
        assert_eq!(
            g.reduced_spell_cost(p, artifact, base_cost(&g, artifact))
                .generic,
            1
        );
        let omen = named(&mut g, p, "Bloomvine Regent // Claim Territory", Zone::Hand);
        assert_eq!(
            g.reduced_spell_cost(p, omen, base_cost(&g, omen)).generic,
            3
        );
        let enemy = named(
            &mut g,
            p.other(),
            "Beanstalk Wurm // Plant Beans",
            Zone::Hand,
        );
        assert_eq!(
            g.reduced_spell_cost(p.other(), enemy, base_cost(&g, enemy))
                .generic,
            4
        );
        let definition = g.state.objects[spell as usize]
            .card
            .adventure_spell()
            .unwrap();
        assert_eq!(
            g.spell_reduction_for_definition(p, spell, definition.fixed_mana_value(), definition),
            0
        );
        act(&mut g, Action::AnnounceAdventure { card: spell });
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 1);
        settle(&mut g);
        priority(&mut g, p);
        act(&mut g, Action::AnnounceExile { card: spell });
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 3);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);

        let mut g = setup(p);
        let sword = named(&mut g, p, NAMES[5], Zone::Battlefield);
        let bunny = named(&mut g, p, "Regal Bunnicorn", Zone::Battlefield);
        let treasure = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
            .unwrap();
        let token = spawn(&mut g, p, treasure, Zone::Battlefield);
        g.state.objects[bunny as usize]
            .counters
            .add(CounterKind::MinusOne, 1);
        equip(&mut g, sword, bunny);
        priority(&mut g, p);
        assert_eq!(g.stats(bunny).power, 2);
        let spell = named(&mut g, p, "Quick Study", Zone::Hand);
        assert_eq!(
            g.reduced_spell_cost(p, spell, base_cost(&g, spell)).generic,
            0
        );
        g.state.objects[bunny as usize].growth.power = -5;
        assert_eq!(
            g.reduced_spell_cost(p, spell, base_cost(&g, spell)).generic,
            2
        );
        g.state.objects[bunny as usize].growth.power = 0;
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: spell });
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.generic, 0);
        assert_eq!(request.required[1], 1);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==token&&*production==[0,1,0,0,0,0])).unwrap();
        act(&mut g, action);
        assert_eq!(g.stats(bunny).power, 1);
        assert_eq!(
            g.reduced_spell_cost(p, spell, base_cost(&g, spell)).generic,
            1
        );
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 0);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn recovered_normals_keep_threshold_mana_discard_bounce_and_real_landfall_lord_bodies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bird = named(&mut g, p, NAMES[1], Zone::Battlefield);
        assert_eq!(g.stats(bird).power, 1);
        let dead: Vec<_> = (0..7)
            .map(|_| named(&mut g, p, "Forest", Zone::Graveyard))
            .collect();
        assert_eq!(g.stats(bird).power, 2);
        assert_eq!(g.stats(bird).toughness, 2);
        g.move_object(dead[0], Zone::Exile);
        assert_eq!(g.stats(bird).power, 1);

        let mut g = setup(p);
        let source = named(&mut g, p, NAMES[3], Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        settle(&mut g);
        priority(&mut g, p);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        let elves: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.owner == p
                    && o.card.is_token()
                    && o.card.has_subtype("Elf")
            })
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(elves.len(), 1);
        assert_eq!(g.stats(elves[0]).power, 2);
        assert_eq!(g.stats(elves[0]).toughness, 2);
        assert_eq!(g.stats(source).power, 2);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        assert_eq!(g.stats(enemy).power, 1);

        let mut g = setup(p);
        let source = named(&mut g, p, NAMES[0], Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        settle(&mut g);
        let turn = g.state.turn;
        for _ in 0..200 {
            if g.state.turn > turn
                && g.state.active == p
                && g.state.step == Step::Main1
                && g.actor() == Some(p)
            {
                break;
            }
            choose(&mut g);
        }
        assert!(g.state.turn > turn && g.state.active == p);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::TapMana {card,..}|Action::ActivateMana {card,..} if *card==source)).unwrap();
        let green = g.state.players[p.index()].mana[4];
        act(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana[4], green + 1);
        // It cannot pay a second tap cost while tapped.
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate {card,..} if *card==source))
        );
        let turn = g.state.turn;
        for _ in 0..200 {
            if g.state.turn > turn
                && g.state.active == p
                && g.state.step == Step::Main1
                && g.actor() == Some(p)
            {
                break;
            }
            choose(&mut g);
        }
        named(&mut g, p, "Forest", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand)
            .count();
        let grave = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
            .count();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::AnnounceActivate {card,..} if *card==source))
            .unwrap();
        act(&mut g, action);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                .count(),
            grave + 1
        );
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count(),
            hand
        );
    }
}

#[test]
fn no_eligible_milled_card_never_forces_illegal_choice_and_all_recovery_excludes_omen_and_old_grave()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let source = named(&mut g, p, NAMES[2], Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: source });
        settle(&mut g);
        assert!(g.observe(p).milled_choice.is_none());
        assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);

        let mut g = setup(p);
        let old = named(&mut g, p, NAMES[0], Zone::Graveyard);
        let omen = named(
            &mut g,
            p,
            "Bloomvine Regent // Claim Territory",
            Zone::Library,
        );
        let ordinary = named(&mut g, p, "Shock", Zone::Library);
        let adventure = named(&mut g, p, NAMES[2], Zone::Library);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let source = named(&mut g, p, NAMES[4], Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: source });
        settle(&mut g);
        assert_eq!(g.state.objects[adventure as usize].zone, Zone::Hand);
        for id in [old, omen, ordinary] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(adventure))
        );

        let mut g = setup(p);
        let chosen = named(&mut g, p, "Shock", Zone::Library);
        for _ in 0..3 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let source = named(&mut g, p, NAMES[1], Zone::Hand);
        choice(&mut g, source);
        let capture = g.object_ref(chosen);
        g.move_object(chosen, Zone::Hand);
        // A hidden replacement destination cannot appear in the public batch
        // continuation, even when the card's new physical identity is known.
        let hidden = g.object_ref(chosen);
        *g.state
            .resolving
            .as_mut()
            .unwrap()
            .milled
            .iter_mut()
            .find(|r| r.card == chosen)
            .unwrap() = hidden;
        let v = g.observe(p).milled_choice.unwrap();
        assert!(!v.milled.contains(&hidden));
        assert!(v.candidates.is_empty());
        reject(&mut g, Action::SelectMilled { object: capture });
        reject(&mut g, Action::SelectMilled { object: hidden });
        act(&mut g, Action::FinishMilledSelection);
        settle(&mut g);
        assert_eq!(g.state.objects[chosen as usize].zone, Zone::Hand);
    }
}
