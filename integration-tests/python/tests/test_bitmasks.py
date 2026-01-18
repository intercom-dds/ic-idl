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

import enum
from types import ModuleType


def test_bitmask_is_flag_type(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    assert issubclass(bitmask.Permissions, enum.Flag)


def test_bitmask_members_exist(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    assert hasattr(bitmask.Permissions, "READ")
    assert hasattr(bitmask.Permissions, "WRITE")
    assert hasattr(bitmask.Permissions, "EXECUTE")
    assert hasattr(bitmask.Permissions, "DELETE")


def test_bitmask_auto_values(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    assert bitmask.Permissions.READ.value == 1
    assert bitmask.Permissions.WRITE.value == 2
    assert bitmask.Permissions.EXECUTE.value == 4
    assert bitmask.Permissions.DELETE.value == 8


def test_bitmask_explicit_values(generated_modules: dict[str, ModuleType]) -> None:
    """IDL bitmask explicit values are bit positions, so 0x01 = position 1 = 2^1 = 2."""
    bitmask = generated_modules["bitmask_types"]
    assert bitmask.ExplicitFlags.FLAG_A.value == 2
    assert bitmask.ExplicitFlags.FLAG_B.value == 4
    assert bitmask.ExplicitFlags.FLAG_C.value == 16
    assert bitmask.ExplicitFlags.FLAG_D.value == 256


def test_bitmask_or_operation(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    combined = bitmask.Permissions.READ | bitmask.Permissions.WRITE
    assert bitmask.Permissions.READ in combined
    assert bitmask.Permissions.WRITE in combined
    assert bitmask.Permissions.EXECUTE not in combined


def test_bitmask_and_operation(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    combined = (
        bitmask.Permissions.READ
        | bitmask.Permissions.WRITE
        | bitmask.Permissions.EXECUTE
    )
    result = combined & bitmask.Permissions.READ
    assert result == bitmask.Permissions.READ


def test_bitmask_in_struct(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    info = bitmask.FileInfo(
        path="/tmp/test", perms=bitmask.Permissions.READ | bitmask.Permissions.WRITE
    )
    assert bitmask.Permissions.READ in info.perms
    assert bitmask.Permissions.WRITE in info.perms
    assert bitmask.Permissions.EXECUTE not in info.perms


def test_bitmask_none_value(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    empty = bitmask.Permissions(0)
    assert empty.value == 0


def test_bitmask_all_combined(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    all_perms = (
        bitmask.Permissions.READ
        | bitmask.Permissions.WRITE
        | bitmask.Permissions.EXECUTE
        | bitmask.Permissions.DELETE
    )
    for p in [
        bitmask.Permissions.READ,
        bitmask.Permissions.WRITE,
        bitmask.Permissions.EXECUTE,
        bitmask.Permissions.DELETE,
    ]:
        assert p in all_perms


def test_bitmask_gapped_positions(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    assert bitmask.GappedFlags.LOW.value == 1
    assert bitmask.GappedFlags.HIGH.value == 128


def test_bitmask_single_flag(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    assert bitmask.SingleFlag.ONLY.value == 1


def test_bitmask_mixed_explicit_auto(generated_modules: dict[str, ModuleType]) -> None:
    bitmask = generated_modules["bitmask_types"]
    assert bitmask.MixedFlags.AUTO_FIRST.value == 1
    assert bitmask.MixedFlags.EXPLICIT_FOUR.value == 16
    assert bitmask.MixedFlags.AUTO_FIVE.value == 32
    assert bitmask.MixedFlags.AUTO_SIX.value == 64
