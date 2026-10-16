//! Native Standard nonmana Ward choices, privacy, and snapshot workload.
use rustmage::{Action, Card, Config, DeckRules, Game, Keywords, Player, StackKind, Target, Zone};
const WARDS: [&str; 4] = [
    "Spectral Snatcher",
    "Sunset Saboteur",
    "Vein Ripper",
    "Sire of Seven Deaths",
];
fn ward_index(card: Card) -> Option<usize> {
    WARDS.iter().position(|n| *n == card.name())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let mut deck = Vec::new();
    for (name, count) in [("Swamp", 10), ("Island", 4), ("Mountain", 6), ("Forest", 4)] {
        deck.extend(vec![Card::from_name(name).unwrap(); count]);
    }
    for name in WARDS.into_iter().chain([
        "Fanatical Firebrand",
        "Shock",
        "Unsummon",
        "Murder",
        "Llanowar Elves",
    ]) {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported());
        deck.extend([card; 4]);
    }
    assert_eq!(deck.len(), 60);
    let (
        mut decisions,
        mut choices,
        mut paid,
        mut declined,
        mut replays,
        mut checks,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0);
    let (mut entries, mut requests, mut body_resolutions) = ([0; 4], [0; 4], [0; 2]);
    let mut cycles = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck.clone(), deck.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for turn in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let mut actions = game.legal_actions();
            // Even seeds retain Snatcher to exercise its battlefield body too.
            if seed % 2 == 0 {
                actions.retain(|a|!matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}|Action::BeginActivate{card,..} if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Spectral Snatcher")));
            }
            let mut action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            // Prefer legal opposing Ward targets to exercise all payment kinds.
            if let Some(a)=actions.iter().find(|a|matches!(a,Action::Cast{target:Some(Target::Card(id)),..}|Action::Activate{target:Some(Target::Card(id)),..} if before.objects.iter().any(|o|o.id==*id&&o.owner!=actor&&ward_index(o.card).is_some()))) { action=a.clone(); }
            if seed % 2 == 1 && let Some(a) = actions.iter().find(|a| matches!(a, Action::Activate { card, .. } if before.objects.iter().any(|o|o.id==*card && o.card.name()=="Spectral Snatcher"))) { action=a.clone(); }
            let ward = before
                .resolving
                .as_ref()
                .filter(|s| s.related_stack.is_some())
                .and_then(|s| ward_index(s.source_card));
            if let Some(index) = ward
                && (before.effect_choice.is_some() || before.decision == "optional_effect")
            {
                choices += 1;
                requests[index] += 1;
                assert_eq!(before.resolving.as_ref().unwrap().controller.other(), actor);
                let other = game.observe(actor.other());
                assert!(other.actions.is_empty());
                if index < 2 {
                    assert!(other.effect_choice.is_none());
                }
                if seed % 3 == 0 {
                    action = actions
                        .iter()
                        .find(|a| {
                            matches!(
                                a,
                                Action::DeclineEffectPayment
                                    | Action::ChooseOptional { accept: false }
                            )
                        })
                        .unwrap()
                        .clone();
                    declined += 1;
                } else {
                    paid += 1;
                }
            }
            let saved = game.snapshot();
            let boundary = ward.is_some()
                || before.announcing.is_some()
                || before.search_choice.is_some()
                || turn % 17 == 0;
            let mut fork = boundary.then(|| game.clone());
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            game.apply(actor, action.clone())?;
            let after = game.observe(actor);
            if let Some(index) = ward {
                let link = before.resolving.as_ref().unwrap().related_stack.unwrap();
                if matches!(
                    action,
                    Action::DeclineEffectPayment | Action::ChooseOptional { accept: false }
                ) {
                    assert!(!after.stack.iter().any(|s| s.id == link.id));
                }
                if matches!(action, Action::ChooseOptional { accept: true }) {
                    assert_eq!(
                        after.players[actor.index()].life,
                        before.players[actor.index()].life - 7
                    );
                    assert_eq!(
                        after.players[actor.index()].life_lost_this_turn,
                        before.players[actor.index()].life_lost_this_turn + 7
                    );
                    assert_eq!(index, 3);
                }
                if matches!(action, Action::Discard { .. } | Action::Sacrifice { .. }) {
                    assert!(after.stack.iter().any(|s| s.id == link.id));
                }
            }
            if before.decision == "priority" && matches!(action, Action::Pass) {
                for item in &before.stack {
                    if after.stack.iter().any(|s| s.id == item.id)
                        || after.resolving.as_ref().is_some_and(|s| s.id == item.id)
                    {
                        continue;
                    }
                    if let StackKind::TriggeredAbility { ability } = item.kind {
                        if ability == 1 && item.source_card.name() == "Sunset Saboteur" {
                            body_resolutions[0] += 1;
                        }
                        if ability == 1 && item.source_card.name() == "Vein Ripper" {
                            body_resolutions[1] += 1;
                        }
                    }
                }
            }
            if matches!(action, Action::FinishSearch)
                && before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.source_card.name() == "Spectral Snatcher")
            {
                cycles += 1;
            }
            for object in &after.objects {
                if object.zone != Zone::Battlefield {
                    continue;
                }
                if let Some(index) = ward_index(object.card) {
                    assert_eq!(
                        object.keywords,
                        [
                            Keywords::NONE,
                            Keywords::MENACE,
                            Keywords::FLYING,
                            Keywords::REACH
                                .union(Keywords::FIRST_STRIKE)
                                .union(Keywords::VIGILANCE)
                                .union(Keywords::MENACE)
                                .union(Keywords::TRAMPLE)
                                .union(Keywords::LIFELINK)
                        ][index]
                    );
                    checks += 1;
                    if !before
                        .objects
                        .iter()
                        .any(|o| o.reference == object.reference && o.zone == Zone::Battlefield)
                    {
                        entries[index] += 1;
                    }
                }
            }
            if let Some(f) = fork {
                let expected = game.snapshot();
                assert_eq!(f.snapshot(), expected);
                game.restore(&saved)?;
                assert!(game.apply_index(actor, before.revision, 0).is_err());
                game.apply(actor, action)?;
                assert_eq!(game.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some(), "game {seed} did not finish");
    }
    if games >= 100 {
        assert!(entries.iter().all(|n| *n > 0));
        assert!(requests.iter().all(|n| *n > 0));
        assert!(body_resolutions.iter().all(|n| *n > 0));
        assert!(
            cycles > 0 && paid > 0 && declined > 0 && replays > 0,
            "cycles {cycles}, paid {paid}, declined {declined}, replays {replays}"
        );
    }
    println!(
        "Completed {games} Standard nonmana Ward games: {decisions} decisions, {choices} owned cost choices, {paid} payments, {declined} declines, {checks} body checks, entries {entries:?}, requests {requests:?}, attack/death resolutions {body_resolutions:?}, {cycles} Swampcyclings, {replays} clone/restore replays, {max_bytes} maximum initialized bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
