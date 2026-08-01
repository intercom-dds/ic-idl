# Copyright 2026 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

import json
from pathlib import Path
from typing import Any

import pytest

from conftest import make_output_dir, run_codegen

jsonschema = pytest.importorskip("jsonschema")

SCHEMA_PATH = Path(__file__).parent.parent / "schemas" / "dds-json_types.schema.json"

CORBA_KINDS = {"interface", "valuetype", "exception"}

COLLECTION_DECLS = (
    "sequenceStructMemberDecl",
    "arrayStructMemberDecl",
    "mapStructMemberDecl",
    "sequenceUnionCaseDecl",
    "arrayUnionCaseDecl",
    "mapUnionCaseDecl",
    "sequenceTypeDecl",
    "arrayTypeDecl",
    "mapTypeDecl",
)

TYPE_REF_FIELDS = ("type", "key_type", "value_type")


def load_schema() -> dict[str, Any]:
    schema = json.loads(SCHEMA_PATH.read_text())
    definitions = schema["definitions"]

    definitions["complexTypeKind"]["enum"].extend(["typedef", *sorted(CORBA_KINDS)])
    definitions["structDecl"]["properties"]["members"].pop("minItems", None)

    for name in COLLECTION_DECLS:
        properties = definitions[name]["properties"]
        for field in TYPE_REF_FIELDS:
            if field in properties:
                properties[field] = {
                    "oneOf": [
                        {"type": "string"},
                        {"$ref": "#/definitions/typeDecl"},
                    ]
                }

    return schema


def prune(node: Any) -> Any:
    if not isinstance(node, dict):
        return node

    if node.get("kind") == "module":
        return {
            name: prune(child)
            for name, child in node.items()
            if not (isinstance(child, dict) and child.get("kind") in CORBA_KINDS)
        }

    return node


def prune_document(doc: dict[str, Any]) -> dict[str, Any]:
    return {
        name: prune(decl)
        for name, decl in doc.items()
        if not (isinstance(decl, dict) and decl.get("kind") in CORBA_KINDS)
    }


@pytest.fixture(scope="session")
def validator() -> Any:
    return jsonschema.Draft7Validator(load_schema())


@pytest.fixture
def json_spec_output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "json-spec")


def test_json_spec(
    idl_file: Path, idl_compiler: Path, json_spec_output_dir: Path, validator: Any
) -> None:
    json_files = run_codegen(idl_compiler, idl_file, json_spec_output_dir, "json-out")

    for json_file in json_files:
        doc = prune_document(json.loads(json_file.read_text()))
        errors = sorted(validator.iter_errors(doc), key=lambda e: list(e.path))
        if errors:
            detail = "\n".join(
                f"  {'/'.join(str(p) for p in e.path)}: {e.message}" for e in errors[:10]
            )
            raise AssertionError(
                f"{json_file.name} violates DDS-JSON schema:\n{detail}"
            )
