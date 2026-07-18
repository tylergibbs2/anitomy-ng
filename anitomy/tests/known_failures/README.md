# known_failures

One file per conformance suite (`anitomy_develop`, `anitomy_master`, `anitopy`,
`anitomy_ng`, `together`). Each lists, one per line, the fixture cases that the
parser does not currently match for that suite (by input for the single-file
suites; by case *name* for `together`, whose cases are sets of related inputs).

For the external suites these are not bugs to fix blindly: they have imperfect,
mutually-contradictory ground truth — some cases no implementation passes,
including upstream's own compiled binary (see `scripts/build_fixtures.py` and
`third_party/README.md`). The manifest turns each suite into a regression guard
plus a ratchet: both `anitomy/tests/conformance.rs` and
`anitomy-py/tests/test_conformance.py` fail if the failing set *changes* —
a newly-failing case is a regression; a newly-passing case must be removed here.

`anitomy_ng` is this project's own hand-curated ground-truth suite; its
`anitomy_ng.txt` records the few curated cases whose *correct* expected output
the parser can't yet reach without overfitting, so the intended result is
tracked rather than dropped or forced through.

`together` covers `anitomy_ng::parse_together` (parsing a *set* of related
filenames together; see issue #3). Its fixtures in `tests/fixtures/together.json`
carry hand-authored ground-truth output — the ideal per-file parse each record
*should* produce — so `together.txt` records exactly where it still falls short
of that target (e.g. residual directory noise, an unrecovered episode title, or
a varying non-episode span mistaken for an episode).

Regenerate after a deliberate parser or fixture change (Rust is the source of
truth; the Python suite reads the same files):

```sh
UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test conformance
UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test together
```

Then review the `git diff` — it shows exactly which cases moved and in which
direction. See `scripts/refresh_conformance.py` for the full re-vendor loop.
