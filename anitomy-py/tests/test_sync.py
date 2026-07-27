# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""The hand-written `ElementKind` enum and the `.pyi` stub mirror lists that
live in Rust. Without these tests a new core variant surfaces as a `ValueError`
from `parse()` in user code rather than a CI failure.
"""

import ast
import dataclasses
import inspect
import pathlib

import anitomy_ng
from anitomy_ng import _anitomy


def test_element_kind_matches_native():
    assert {k.value for k in anitomy_ng.ElementKind} == set(_anitomy.kind_names())


def test_element_kind_declaration_order_matches_native():
    assert [k.value for k in anitomy_ng.ElementKind] == _anitomy.kind_names()


def test_every_native_kind_is_constructible():
    for name in _anitomy.kind_names():
        assert anitomy_ng.ElementKind(name).value == name


def test_options_fields_match_native():
    fields = {n for n, _ in inspect.getmembers(anitomy_ng.Options) if n.startswith("parse_")}
    assert fields == set(_anitomy.option_fields())


def test_options_stub_matches_native():
    """The `.pyi` is hand-written and never imported at runtime, so pyright
    only ever checks callers against it — nothing checks it against the real
    pyclass. Parse the stub file itself."""
    stub = pathlib.Path(anitomy_ng.__file__).parent / "_anitomy.pyi"
    tree = ast.parse(stub.read_text(encoding="utf-8"))
    declared = {
        node.target.id
        for cls in tree.body
        if isinstance(cls, ast.ClassDef) and cls.name == "Options"
        for node in cls.body
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name)
    }
    assert declared == set(_anitomy.option_fields())


def test_element_dataclass_fields():
    names = [f.name for f in dataclasses.fields(anitomy_ng.Element)]
    assert names == ["kind", "value", "position"]
