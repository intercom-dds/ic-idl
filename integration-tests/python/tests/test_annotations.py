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

from types import ModuleType


def test_keyed_struct_exists(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.KeyedStruct(id=1, name="test", value=3.14)
    assert s.id == 1
    assert s.name == "test"


def test_multi_key_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.MultiKeyStruct(namespace="ns", id=42, data="payload")
    assert s.namespace == "ns"
    assert s.id == 42


def test_optional_fields_default_none(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.OptionalStruct(required_field=1)
    assert s.required_field == 1
    assert s.optional_int is None
    assert s.optional_string is None
    assert s.optional_seq is None


def test_optional_fields_can_be_set(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.OptionalStruct(
        required_field=1,
        optional_int=42,
        optional_string="hello",
        optional_seq=[1, 2, 3],
    )
    assert s.optional_int == 42
    assert s.optional_string == "hello"
    assert s.optional_seq == [1, 2, 3]


def test_optional_type_annotations(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    annotations = at.OptionalStruct.__annotations__
    assert annotations["required_field"] == "int"
    assert annotations["optional_int"] == "int | None"
    assert annotations["optional_string"] == "str | None"
    assert annotations["optional_seq"] == "list[int] | None"


def test_nested_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.NestedStruct(x=10, y=20)
    assert s.x == 10
    assert s.y == 20


def test_shared_refs_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    nested = at.NestedStruct(x=5, y=10)
    s = at.SharedRefs(shared_string="test", shared_struct=nested)
    assert s.shared_string == "test"
    assert s.shared_struct.x == 5


def test_combined_annotations(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.CombinedAnnotations(id=1, maybe_shared_name=None)
    assert s.id == 1
    assert s.maybe_shared_name is None

    s2 = at.CombinedAnnotations(id=2, maybe_shared_name="named")
    assert s2.maybe_shared_name == "named"


def test_annotated_interface_exists(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    assert hasattr(at.AnnotatedInterface, "fire_and_forget")
    assert hasattr(at.AnnotatedInterface, "get_value")
    assert hasattr(at.AnnotatedInterface, "set_value")


def test_topic_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    msg = at.TopicMessage(message_id=1, payload="data", timestamp=12345)
    assert msg.message_id == 1
    assert msg.payload == "data"
    assert msg.timestamp == 12345


def test_mutable_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.MutableStruct(version=1, data="v1")
    s.version = 2
    s.data = "v2"
    assert s.version == 2
    assert s.data == "v2"


def test_final_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["annotation_types"]
    s = at.FinalStruct(fixed_field=100)
    assert s.fixed_field == 100
