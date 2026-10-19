# Fresh-machine setup and source intake

Read this on a fresh computer, when a checkout/source pin is missing, or when
adding a set not represented in the frozen inventory. Run repository commands
from the Rustmage root. These instructions need Git, Python 3, Rust/rustup with
rustfmt and Clippy, and GitHub access through `gh` or an equivalent connector.
Java/Maven and a running XMage server are not required to read its source.

## Get the project and skill

Use the user's repository/fork when supplied. The upstream is
[altf4/rustmage](https://github.com/altf4/rustmage); XMage source is
[magefree/mage](https://github.com/magefree/mage). In a workspace of your choice:

```sh
git clone https://github.com/altf4/rustmage.git
cd rustmage
```

For a copy of just this skill, clone Rustmage separately and open that checkout
in Codex. Repository-relative references in the skill always refer to the
checkout being implemented, not where the skill was installed.

To install the bundled skill on a new machine, from the repository root:

```sh
mkdir -p "$HOME/.agents/skills"
cp -R skills/rustmage-implement-set "$HOME/.agents/skills/"
```

If that destination already exists, inspect it and update deliberately rather
than nesting a duplicate or discarding customizations. Codex supports user skills
in `~/.agents/skills`, repo skills in `.agents/skills`, and symlinked skill folders;
see the [official skill documentation](https://learn.chatgpt.com/docs/build-skills).
Restart Codex if discovery has not refreshed. Invoke `$rustmage-implement-set`
with a chosen set, or directly ask Codex to read the repository's `SKILL.md` and
follow it when installation is unavailable.

Inspect existing issues/PRs and determine the authorized write destination.
For `gh`, check `gh auth status`; use an existing fork when upstream writes are
unavailable. Read-only cloning does not establish push or merge permissions.
Keep a local ledger and prepared diffs if authentication is missing, continue
independent local work, and report the access needed for publishing/reviews.
Never claim locally prepared work is published or merged.

## Establish the bounded build environment

The current resource guard uses Linux cgroup v2 and a working **user systemd**
manager. On macOS, Windows, or a container lacking that support, use a Linux host
or VM that can enforce the existing guard. Do not disable `.cargo/config.toml`,
replace the wrapper with raw Cargo, weaken the 6 GiB limit or pretend validation
ran. Source inspection and editing can proceed while build access is resolved.

Read `.github/workflows/ci.yml` for the current Rust toolchain and the example
bounded-runner setup. Install that toolchain with rustfmt and Clippy, and use it
for this checkout. Do not hard-code the version from a past chat. Check:

```sh
python3 --version
rustup show
systemctl --user show-environment > /dev/null
test -f /sys/fs/cgroup/cgroup.controllers
python3 scripts/resource_guard.py -- python3 -c 'from scripts.resource_limits import bounded_group; assert bounded_group() is not None; print("bounded runner ready")'
```

This preflight checks enforcement without allocating a kill probe. The guard
sets one Cargo job/test thread and shares its build lock and 6 GiB parent cgroup
across worktrees. Do not launch unmanaged builds from another directory with
`--manifest-path`. It writes resource reports under the cache; retain failure
evidence. Host service setup requiring administration is an environment
prerequisite, not a reason to bypass the guard.

## Fetch and pin XMage source

The resolver uses `RUSTMAGE_XMAGE_SOURCE`, otherwise a sibling `mage` directory;
linked worktrees also find the primary checkout's sibling. For a **new** source
clone, from the Rustmage root:

```sh
git clone https://github.com/magefree/mage.git ../mage
export RUSTMAGE_XMAGE_SOURCE="$(cd ../mage && pwd)"
rustmage_xmage_revision=$(python3 -c 'import json; print(json.load(open("data/standard/xmage-inventory.json"))["xmage_source_commit"])')
git -C "$RUSTMAGE_XMAGE_SOURCE" checkout --detach "$rustmage_xmage_revision"
```

Reuse an existing source checkout without resetting dirty work. If it is at a
different revision, use a separate source checkout/worktree and set the override.
Do not change sources while an audit or validator is consuming them. Inventory
reconciliation expects the exact pinned HEAD and clean Java source directories.

Keep Git history available: provenance tests use `git show REVISION:PATH`, and
records can reference older revisions besides the inventory HEAD. A shallow clone
of only today's pin may be insufficient. Fetch missing pinned commits as needed;
never remove historical pins or skip source checks to make validation pass.

## Understand the data boundary

Read these current files instead of relying on a remembered card count:

| Path | Purpose |
| --- | --- |
| `data/targets/*.json`, `docs/standard-target.md` | Frozen format/date, sets and bans |
| `data/standard/cards.json` | Pinned Oracle identities and complete face facts |
| `data/standard/xmage-inventory.json` | XMage revision, set tables, implementations and gaps |
| `data/catalogue-identities.json` | Stable catalogue IDs, canonical paths, aliases and physical metadata |
| `data/sets/<code>.json` | Full-set membership and facts pins |
| `src/card_definitions/<set>/*.{rs,json}` | Direct behavior plus paired provenance |
| `data/standard/coverage.json` | Generated admission report, not rules certification |

An existing gated card already has a canonical paired definition. Resolve its
identity/path and edit that pair; creating another copy under a reprint's set
would duplicate the card. For example, Oracle facts may name both faces while
the coverage/catalogue key uses a front face. Resolve by Oracle identity and
existing aliases rather than assuming every display name is a file path.

For paired provenance, inspect a recent directly authored card, the
`xmage_dependencies` records, `scripts/link_catalogue.py` and
`scripts/test_direct_cards.py`. The linker exposes `canonical` and
`physical_digest` for the actual hashing conventions. Preserve `typed_lift`
history when present and add truthful new source evidence. Do not copy another
card's identity, revision or hashes. Run the provenance tests against real source:

```sh
python3 -m unittest scripts.test_direct_cards scripts.test_set_status scripts.test_source_paths
```

## A new set or a newer Standard snapshot

A set missing from the target/inventory needs source-intake work before
`set_status --freeze` can succeed. There is **no automatic behavior importer**.
A new manifest cannot manufacture Oracle facts, catalogue identities or Rust
implementations. Do the following on a dedicated branch before dependent batches:

1. Verify the requested date, full set membership, releases and bans using
   [Wizards Standard](https://magic.wizards.com/en/formats/standard), the
   [banned list](https://magic.wizards.com/en/banned-restricted-list), official
   card/rules information and Scryfall facts. Record source URLs and retrieval
   date. Keep an explicitly requested historical snapshot reproducible.
2. Prepare a versioned target JSON modeled on the existing target; choose a
   reviewed XMage revision with the required tables/classes. Keep the old target
   available. Inspect `scripts/fetch_standard.py --help` and
   `scripts/inventory_standard.py --help` for the actual interfaces. Fetch
   candidate facts/inventory into new cache paths before replacing checked-in
   inputs. Pass `--target` explicitly: the inventory tool's default target may
   be older than the requested set.
3. Reconcile every physical member, including supplementary/starter printings,
   reprints, all faces and bans. `fetch_standard.py` queries **currently legal**
   Standard cards; its `--as-of` filters release dates, not historical legality,
   and its result excludes banned cards. XMage tables can also be incomplete.
   Neither source alone proves full-set membership. Preserve explicit gaps and
   obtain the missing authoritative facts; extend membership tooling/tests as
   needed rather than dropping banned or missing identities.
4. Add or update input metadata and gated paired definitions using the current
   schema. Preserve all existing stable IDs, aliases and synthetic recipes;
   inspect category/index boundaries before adding identities. If current tools
   cannot represent additions safely, implement and test the required metadata
   tooling first. Do not hand-edit generated files, renumber existing cards or
   put behavior into the linker. Source/legality updates must not overwrite
   already working typed implementations or falsely admit stubs.
5. Regenerate links with `python3 scripts/link_catalogue.py`. Freeze the selected
   set once the source inputs are reconciled, then check membership, provenance
   and the release validator. Review source/schema changes and their impact on
   existing sets and legality before integrating. Pin the accepted inputs for
   the implementation batches; do not refetch live facts during those jobs.

Use isolated worktrees for simultaneous authors. Read focused scenario tests
near the affected engine code; the existing `tests/foundations_games.rs` is a
full-game test pattern. The selected set requires its own acceptance evidence.
