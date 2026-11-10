//! Private choice ownership, exact spell taxes and lasting exile permission.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CHOSEN_TYPE_TAX: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ChosenCardTypeSpellTax(1)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NoncreatureCardTypeAfterOpponentHand),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const LIBRARY_CHOICE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::SelectTargetLibraryFaceDownPlay {
            look: 9,
            count: 2,
            spend_any_mana: true,
        }],
    }],
    ..CardRules::EMPTY
};

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn type_choice(kind: NoncreatureCardType) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::CardType(kind),
    }
}

fn can_cast(game: &Game, id: CardId) -> bool {
    game.legal_actions().iter().any(|action| {
        matches!(action, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == id)
    })
}

#[test]
fn private_hand_peek_branches_into_public_exact_source_choices_and_taxes_both_seats() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CHOSEN_TYPE_TAX)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        let own_opt = put(&mut game, player, "Opt", Zone::Hand);
        let opposing_opt = put(&mut game, opponent, "Opt", Zone::Hand);
        let artifact_creature = put(&mut game, player, "Solemn Simulacrum", Zone::Hand);

        cast(&mut game, source, None);
        resolve_top(&mut game);
        assert_eq!(game.actor(), Some(player));
        assert_eq!(game.object_view(source).zone, Zone::Stack);
        let look = game
            .observe(player)
            .entry_choice
            .unwrap()
            .opponent_hand
            .unwrap();
        assert_eq!(
            look.iter().map(|card| card.id).collect::<Vec<_>>(),
            [opposing_opt]
        );
        assert!(
            game.observe(opponent)
                .entry_choice
                .unwrap()
                .opponent_hand
                .is_none()
        );
        assert_eq!(
            game.legal_actions(),
            NoncreatureCardType::ALL
                .into_iter()
                .map(type_choice)
                .collect::<Vec<_>>()
        );

        let choose_instant = type_choice(NoncreatureCardType::Instant);
        let saved = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &choose_instant)
            .unwrap();
        assert_eq!(
            game.apply(opponent, choose_instant.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(game.snapshot(), saved);
        reject(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Blue),
            },
        );
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        act(&mut raw, choose_instant);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for seat in [player, opponent] {
            let observation = raw.observe(seat);
            assert!(observation.entry_choice.is_none());
            assert_eq!(
                observation
                    .objects
                    .iter()
                    .find(|card| card.id == source)
                    .unwrap()
                    .entry_choice,
                Some(EntryChoiceValue::CardType(NoncreatureCardType::Instant))
            );
        }

        for seat in [player, opponent] {
            let own_spell = if seat == player {
                own_opt
            } else {
                opposing_opt
            };
            raw.state.players[seat.index()].mana = [0, 1, 0, 0, 0, 0];
            assert_eq!(raw.actor(), Some(seat));
            assert!(!can_cast(&raw, own_spell));
            raw.state.players[seat.index()].mana[5] = 1;
            assert!(can_cast(&raw, own_spell));
            if seat == player {
                raw.state.players[seat.index()].mana = [0, 0, 0, 0, 0, 4];
                assert!(can_cast(&raw, artifact_creature));
                act(&mut raw, Action::Pass);
            }
        }
        cast(&mut raw, opposing_opt, None);
        assert_eq!(raw.state.players[opponent.index()].mana, [0; 6]);

        raw.restore(&saved).unwrap();
        act(&mut raw, type_choice(NoncreatureCardType::Artifact));
        raw.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        assert!(!can_cast(&raw, artifact_creature));
        raw.state.players[player.index()].mana[5] = 5;
        assert!(can_cast(&raw, artifact_creature));
        assert_eq!(
            indexed.object_view(source).entry_choice,
            Some(EntryChoiceValue::CardType(NoncreatureCardType::Instant))
        );
    }
}

#[test]
fn spell_tax_uses_proposed_face_and_only_the_available_ability() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CHOSEN_TYPE_TAX)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let adventure_card = put(&mut game, player, "Picklock Prankster", Zone::Hand);
    let disable = put(&mut game, player, "Witness Protection", Zone::Hand);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);

    cast(&mut game, source, None);
    resolve_top(&mut game);
    act(&mut game, type_choice(NoncreatureCardType::Instant));
    let adventure = Action::AnnounceAdventure {
        card: adventure_card,
    };

    // Creature and Adventure both cost {1}{U}. Only the proposed Instant is taxed.
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    assert!(can_cast(&game, adventure_card));
    reject(&mut game, adventure.clone());
    game.state.players[player.index()].mana[5] = 2;
    assert!(game.legal_actions().contains(&adventure));

    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, disable, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(disable).zone, Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    assert!(game.legal_actions().contains(&adventure));

    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(&mut game, disenchant, Some(Target::Card(disable)));
    settle(&mut game);
    assert_eq!(game.object_view(disable).zone, Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    assert!(can_cast(&game, adventure_card));
    reject(&mut game, adventure);
}

fn target_library(game: &mut Game, player: Player) -> CardId {
    let source = put(game, player, "Grizzly Bears", Zone::Hand);
    cast(game, source, None);
    resolve_top(game);
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    reject(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    resolve_top(game);
    source
}

fn select_library_card(card: CardId) -> Action {
    Action::SelectLibrary { card }
}

fn next_main(game: &mut Game, player: Player) {
    let initial_turn = game.state.turn;
    for _ in 0..64 {
        if game.state.turn > initial_turn
            && game.state.active == player
            && game.state.step == Step::Main1
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("scenario did not reach the player's next main phase");
}

#[test]
fn private_target_library_choice_branches_and_grants_only_the_chosen_exile_plays() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LIBRARY_CHOICE)]);
    for player in [Player::P0, Player::P1] {
        let owner = player.other();
        let mut game = scenario(player);
        let untouched = game.state.players[owner.index()].library.to_vec();
        let looked: Vec<_> = [
            "Opt",
            "Mountain",
            "Giant Growth",
            "Plains",
            "Island",
            "Essence Scatter",
            "Lightning Strike",
            "Murder",
            "Llanowar Elves",
        ]
        .iter()
        .map(|name| put(&mut game, owner, name, Zone::Library))
        .collect();
        let source = target_library(&mut game, player);

        let private = game.observe(player).library_selection.unwrap();
        assert_eq!(private.minimum, 2);
        assert_eq!(private.maximum, 2);
        assert_eq!(
            private
                .remaining
                .iter()
                .map(|card| card.id)
                .collect::<Vec<_>>(),
            looked.iter().rev().copied().collect::<Vec<_>>()
        );
        assert!(game.observe(owner).library_selection.is_none());
        let saved = game.snapshot();
        assert_eq!(
            game.apply(owner, select_library_card(looked[0])),
            Err(Error::WrongActor)
        );
        reject(&mut game, select_library_card(untouched[0]));
        reject(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.snapshot(), saved);

        let revision = game.revision();
        let spell = select_library_card(looked[0]);
        let index = game
            .legal_actions()
            .iter()
            .position(|action| action == &spell)
            .unwrap();
        let mut rng = game.state.rng;
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);

        act(&mut raw, spell);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for game in [&mut raw, &mut indexed] {
            reject(game, Action::FinishLibrarySelection);
            reject(game, select_library_card(looked[0]));
            act(game, select_library_card(looked[1]));
            reject(game, select_library_card(looked[2]));
            act(game, Action::FinishLibrarySelection);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());

        let mut bottom = looked[2..].iter().rev().copied().collect::<Vec<_>>();
        rng.shuffle(&mut bottom);
        let expected_library: Vec<_> = bottom.into_iter().rev().chain(untouched).collect();
        assert_eq!(
            raw.state.players[owner.index()].library.to_vec(),
            expected_library
        );
        assert_eq!(raw.state.rng, rng);

        let hidden = &looked[..2];
        let beneficiary = raw.observe(player);
        let opponent = raw.observe(owner);
        for &id in hidden {
            assert!(
                beneficiary
                    .objects
                    .iter()
                    .any(|card| card.id == id && card.zone == Zone::Exile)
            );
            assert!(!opponent.objects.iter().any(|card| card.id == id));
            assert!(beneficiary.exile_permissions.iter().any(|permission| {
                permission.object == raw.object_ref(id)
                    && permission.player == player
                    && permission.expiry == ExilePlayExpiry::WhileExiled
                    && permission.spend_any_mana
                    && !permission.cast_only
            }));
            assert!(
                !opponent
                    .exile_permissions
                    .iter()
                    .any(|permission| permission.object.card == id)
            );
        }

        indexed.restore(&saved).unwrap();
        for &id in &looked[2..4] {
            act(&mut indexed, select_library_card(id));
        }
        act(&mut indexed, Action::FinishLibrarySelection);
        assert_eq!(indexed.object_view(looked[0]).zone, Zone::Library);
        assert_eq!(indexed.object_view(looked[2]).zone, Zone::Exile);
        assert_eq!(raw.object_view(looked[0]).zone, Zone::Exile);
        assert_eq!(raw.object_view(looked[2]).zone, Zone::Library);

        let removal = put(&mut raw, player, "Murder", Zone::Hand);
        cast(&mut raw, removal, Some(Target::Card(source)));
        settle(&mut raw);
        assert_eq!(raw.object_view(source).zone, Zone::Graveyard);
        next_main(&mut raw, player);
        act(&mut raw, Action::PlayLand { card: looked[1] });
        assert_eq!(raw.object_view(looked[1]).controller, player);
        assert_eq!(raw.object_view(looked[1]).owner, owner);
        assert_eq!(raw.object_view(looked[1]).zone, Zone::Battlefield);
        raw.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut raw, Action::AnnounceExile { card: looked[0] });
        pay(&mut raw);
        assert_eq!(raw.state.players[player.index()].mana, [0; 6]);
        assert_eq!(raw.object_view(looked[0]).zone, Zone::Stack);
    }
}

#[test]
fn shorter_library_exiles_every_available_card() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LIBRARY_CHOICE)]);
    for count in 0..=2 {
        let player = Player::P1;
        let owner = player.other();
        let mut game = scenario(player);
        let library = game.state.players[owner.index()].library.to_vec();
        for id in library {
            game.move_object(id, Zone::Graveyard);
        }
        let candidates: Vec<_> = (0..count)
            .map(|_| put(&mut game, owner, "Opt", Zone::Library))
            .collect();
        target_library(&mut game, player);
        assert!(game.observe(player).library_selection.is_none());
        assert!(game.state.players[owner.index()].library.is_empty());
        assert_eq!(game.state.exile_permissions.len(), count);
        for card in candidates {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
    }
}
