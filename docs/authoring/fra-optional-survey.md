# Optional FRA uncast survey

This opt-in corpus is separate from routine validation. It defines ten deterministic FRA-only Standard decks and five fixed pairings, each with two seed indices and swapped deck seats, for 20 cases. The verified runner samples owned/index/raw/restore and both-viewer checks and replays each completed game. The deck list prioritizes physical card fronts absent from the CI 40949901439 cast ledger: it includes 112 such nonland identities and all 20 lands absent from that CI's decks. Inclusion does not establish play or resolution.

The bounded local run completed 19 of 20 cases with terminal outcomes and full replay. Across those completed cases it recorded 13,246 decisions, 159 distinct physical spell-front names, 25 land names actually played, and five other spell-face names. These are aggregate observed names, not per-card, per-seat, or printed-obligation coverage. The remaining case, pair 4 / seed index 1 / unswapped, timed out under both five- and ten-second whole-case bounds. Decision tracing reached 552 and 612 respectively, demonstrating progress across the retries but not proving completion or identifying a particular slow function. These results are from a local run and do not constitute current-head CI or full-set acceptance.

The local optimized-dev example build is recorded in `~/.cache/fra-survey-optimized-build.log`; the bounded batch and case output are in `~/.cache/fra-survey-bounded.log`, `~/.cache/fra-survey-case-0.log`, and `~/.cache/fra-survey-case-*-*-*.log`. The five-second diagnostic trace is `~/.cache/fra-survey-slow-case.log`; the ten-second retry is `~/.cache/fra-survey-case-4-1-0-retry.log`. These logs and any generated traces can contain private action payloads and hidden choices; keep them private. Build and process resource reports remain in `~/.cache/rustmage-build-reports/`.

For a bounded single-case run after the optimized executable exists, use the guard and place the timeout **inside** it, for example:

```sh
python3 scripts/resource_guard.py -- timeout 5s target/debug/examples/fra_optional_survey --case 4 1 0
```

The recorded retry used the same command with `10s`. A separate guarded batch used `/tmp/fra-survey-bounded.sh`, which ran individual `timeout 5s` invocations against the optimized-dev binary and wrote each case's log under `~/.cache`. Build and run records must be retained independently; the example path under `target/debug` reflects the optimized dev profile used for these results, not an unoptimized runtime measurement.

Case arguments are pair index 0–4, seed index 0–1, and swap 0/1. Progress is printed to stderr at each game start, replay, and completion; `survey_elapsed` is cumulative. Set `RUSTMAGE_FRA_SURVEY_TRACE_DECISIONS=1` for detailed legal-action, application, and replay boundaries. Reports distinguish `physical_cards_cast`, `other_spell_faces_cast`, and `lands_played`; they do not prove resolution or both-seat coverage for every card. The guard itself has no per-game timeout. Avoid launching an unbounded full survey by default; select a case and bound it explicitly when further diagnostics are needed.
