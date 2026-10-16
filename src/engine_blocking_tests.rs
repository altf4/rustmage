use super::tests::{act, scenario, spawn};
use super::*;
fn attacker(g: &mut Game, flags: Keywords) -> CardId {
    let id = spawn(g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.state.objects[id as usize].gained_keywords = flags;
    g.state.attackers.push(Attacker {
        card: id,
        blocked: false,
    });
    id
}
fn defender(g: &mut Game, flags: Keywords) -> CardId {
    let id = spawn(g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    g.state.objects[id as usize].gained_keywords = flags;
    id
}
fn setup() -> Game {
    let mut g = scenario();
    g.state.step = Step::Blockers;
    g.state.pending = Pending::Blockers;
    g
}
fn has(g: &Game, b: CardId, a: CardId) -> bool {
    g.legal_actions().contains(&Action::Block {
        blocker: b,
        attacker: a,
    })
}

#[test]
fn menace_needs_two_and_rejects_stranding_or_premature_finish_atomically() {
    let mut g = setup();
    let a = attacker(&mut g, Keywords::MENACE);
    let b = defender(&mut g, Keywords::NONE);
    assert!(!has(&g, b, a));
    assert!(g.legal_actions().contains(&Action::FinishBlockers));
    let c = defender(&mut g, Keywords::NONE);
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    assert!(!g.legal_actions().contains(&Action::FinishBlockers));
    assert!(g.observe(Player::P0).blocks.is_empty());
    assert_eq!(g.observe(Player::P1).blocks, vec![(b, a)]);
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(Player::P1, Action::FinishBlockers),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    act(
        &mut g,
        Action::Block {
            blocker: c,
            attacker: a,
        },
    );
    act(&mut g, Action::FinishBlockers);
    assert_eq!(g.state.blocks.len(), 2);
    assert!(g.state.attackers[0].blocked);
    let result = g.state.clone();
    g.restore(&saved).unwrap();
    act(
        &mut g,
        Action::Block {
            blocker: c,
            attacker: a,
        },
    );
    act(&mut g, Action::FinishBlockers);
    assert_eq!(g.state, result);
}

#[test]
fn minimum_three_and_maximum_one_combine_cumulatively_with_menace() {
    let mut g = setup();
    let a = attacker(&mut g, Keywords::MIN_THREE_BLOCKERS.union(Keywords::MENACE));
    let b = defender(&mut g, Keywords::NONE);
    let c = defender(&mut g, Keywords::NONE);
    assert!(!has(&g, b, a));
    let d = defender(&mut g, Keywords::NONE);
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    act(
        &mut g,
        Action::Block {
            blocker: c,
            attacker: a,
        },
    );
    assert!(!g.legal_actions().contains(&Action::FinishBlockers));
    act(
        &mut g,
        Action::Block {
            blocker: d,
            attacker: a,
        },
    );
    assert!(g.legal_actions().contains(&Action::FinishBlockers));
    let mut g = setup();
    let a = attacker(&mut g, Keywords::MAX_ONE_BLOCKER);
    let b = defender(&mut g, Keywords::NONE);
    let c = defender(&mut g, Keywords::NONE);
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    assert!(!has(&g, c, a));
    let mut g = setup();
    attacker(&mut g, Keywords::MAX_ONE_BLOCKER.union(Keywords::MENACE));
    for _ in 0..3 {
        defender(&mut g, Keywords::NONE);
    }
    assert_eq!(
        g.legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::Block { .. }))
            .count(),
        0
    );
}

#[test]
fn changing_evasion_after_declaration_does_not_undo_existing_blocks() {
    let mut g = setup();
    let a = attacker(&mut g, Keywords::NONE);
    let b = defender(&mut g, Keywords::NONE);
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    act(&mut g, Action::FinishBlockers);
    g.state.objects[a as usize].gained_keywords = Keywords::MENACE.union(Keywords::CANT_BE_BLOCKED);
    g.state.step = Step::CombatDamage;
    g.prepare_damage();
    g.state_based_actions();
    assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn exact_action_mask_matches_exhaustive_complete_declarations() {
    // Independent oracle enumerates every complete assignment. Production uses
    // augmenting paths over only outstanding obligations, never this enumeration.
    let cases = [
        [Keywords::MENACE, Keywords::MENACE, Keywords::NONE],
        [
            Keywords::MENACE.union(Keywords::FLYING),
            Keywords::MENACE.union(Keywords::FLYING),
            Keywords::NONE,
        ],
        [
            Keywords::MIN_THREE_BLOCKERS,
            Keywords::MENACE,
            Keywords::NONE,
        ],
        [
            Keywords::MENACE.union(Keywords::MAX_ONE_BLOCKER),
            Keywords::MENACE,
            Keywords::NONE,
        ],
        [
            Keywords::NO_TOKEN_BLOCKERS.union(Keywords::MENACE),
            Keywords::MENACE,
            Keywords::MAX_ONE_BLOCKER,
        ],
        [Keywords::CANT_BE_BLOCKED, Keywords::FLYING, Keywords::NONE],
    ];
    let mut inspected = 0;
    for flags in cases {
        let mut root = setup();
        let aa: Vec<_> = flags.iter().map(|&f| attacker(&mut root, f)).collect();
        let bb = vec![
            defender(&mut root, Keywords::REACH),
            defender(&mut root, Keywords::NONE),
            defender(&mut root, Keywords::FLYING),
        ];
        let token = Card::catalogue()
            .find(|c| {
                c.token_info()
                    .is_some_and(|info| info.subtypes == "Elf Warrior")
            })
            .unwrap();
        root.create_tokens(Player::P1, token, 1, false);
        let mut bb = bb;
        bb.push((root.state.objects.len() - 1) as CardId);
        root.state.objects[bb[3] as usize].gained_keywords = Keywords::FLYING;
        let mut complete = Vec::new();
        for encoded in 0usize..256 {
            let mut code = encoded;
            let mut assignment = vec![None; 4];
            let mut counts = [0; 3];
            let mut valid = true;
            for (b, selected) in assignment.iter_mut().enumerate() {
                let slot = code % 4;
                code /= 4;
                if slot == 0 {
                    continue;
                }
                let a = slot - 1;
                *selected = Some(a);
                counts[a] += 1;
                // Known board: blockers 0,2,3 can block flying; 3 is a token.
                if flags[a].contains(Keywords::CANT_BE_BLOCKED)
                    || (flags[a].contains(Keywords::FLYING) && b == 1)
                    || (flags[a].contains(Keywords::NO_TOKEN_BLOCKERS) && b == 3)
                {
                    valid = false;
                }
            }
            for a in 0..3 {
                let minimum = if flags[a].contains(Keywords::MIN_THREE_BLOCKERS) {
                    3
                } else if flags[a].contains(Keywords::MENACE) {
                    2
                } else {
                    1
                };
                if (counts[a] > 0 && counts[a] < minimum)
                    || (flags[a].contains(Keywords::MAX_ONE_BLOCKER) && counts[a] > 1)
                {
                    valid = false;
                }
            }
            if valid {
                complete.push(assignment);
            }
        }
        let mut queue = vec![root];
        let mut seen = std::collections::HashSet::new();
        while let Some(g) = queue.pop() {
            let partial: Vec<_> = bb
                .iter()
                .map(|b| {
                    g.state
                        .selected_blocks
                        .iter()
                        .find(|(id, _)| id == b)
                        .map(|(_, a)| aa.iter().position(|id| id == a).unwrap())
                })
                .collect();
            if !seen.insert(partial.clone()) {
                continue;
            }
            inspected += 1;
            let actions = g.legal_actions();
            assert_eq!(
                actions.contains(&Action::FinishBlockers),
                complete.contains(&partial)
            );
            for (b, &blocker) in bb.iter().enumerate() {
                for (a, &attacker) in aa.iter().enumerate() {
                    let expected = partial[b].is_none()
                        && complete.iter().any(|full| {
                            full[b] == Some(a)
                                && partial.iter().zip(full).all(|(p, f)| p.is_none() || p == f)
                        });
                    assert_eq!(
                        has(&g, blocker, attacker),
                        expected,
                        "partial={partial:?}, b={b}, a={a}, flags={flags:?}"
                    );
                    if expected {
                        let mut next = g.clone();
                        act(&mut next, Action::Block { blocker, attacker });
                        queue.push(next);
                    }
                }
            }
        }
    }
    assert!(inspected > 100);
}

#[test]
fn compiled_restrictions_and_created_menace_tokens_use_shared_legality() {
    let mut g = scenario();
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let spell = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Enter the Enigma").unwrap(),
        Zone::Hand,
    );
    g.state.players[0].mana[1] = 1;
    super::tests::cast(&mut g, spell, Some(Target::Card(bear)));
    super::tests::pass_pair(&mut g);
    assert!(
        g.object_view(bear)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    let vampire = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Vampire Interloper").unwrap(),
        Zone::Battlefield,
    );
    assert!(!g.can_block_pair(vampire, bear));
    g.finish_cleanup();
    assert!(
        !g.object_view(bear)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    assert!(!g.can_be_blocker(vampire));
    let mut g = setup();
    let hydra = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Agents of HYDRA").unwrap(),
        Zone::Battlefield,
    );
    g.state.pending = Pending::Priority;
    g.move_object(hydra, Zone::Graveyard);
    g.prepare_triggers();
    super::tests::pass_pair(&mut g);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.card.is_token() && o.zone == Zone::Battlefield)
        .unwrap()
        .0 as CardId;
    assert!(g.object_view(token).keywords.contains(Keywords::MENACE));
    g.state.attackers.push(Attacker {
        card: token,
        blocked: false,
    });
    g.state.pending = Pending::Blockers;
    let b = defender(&mut g, Keywords::NONE);
    assert!(!has(&g, b, token));
    let c = defender(&mut g, Keywords::NONE);
    assert!(has(&g, c, token));
    let ceratops = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Rampaging Ceratops").unwrap(),
        Zone::Battlefield,
    );
    g.state.attackers.push(Attacker {
        card: ceratops,
        blocked: false,
    });
    assert!(!has(&g, b, ceratops));
    defender(&mut g, Keywords::NONE);
    assert!(has(&g, b, ceratops));
}
