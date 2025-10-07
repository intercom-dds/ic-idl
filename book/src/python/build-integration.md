# Build integration

The Python backend only emits source files, so you can integrate it into any
packaging workflow that runs arbitrary commands before packaging.

## Setuptools example (`setup.py`)

```python
from pathlib import Path
from setuptools import setup
from setuptools.command.build_py import build_py
import subprocess

class BuildIDL(build_py):
    def run(self):
        schema = Path("schema.idl")
        output = Path("src/my_package/generated")
        output.mkdir(parents=True, exist_ok=True)

        subprocess.check_call([
            "ic-idl",
            "--python-out",
            str(output),
            str(schema),
        ])

        super().run()

setup(
    name="my-package",
    packages=["my_package"],
    cmdclass={"build_py": BuildIDL},
)
```

Generated files live under `src/my_package/generated` and are included in the
wheel.

## `pyproject.toml` (PEP 517)

If you build with `scikit-build-core`, `hatchling`, or other PEP 517 backends,
run `ic-idl` from a hook script or a small helper invoked before the build. For
example with `hatchling` you can add a custom build target:

```toml
[tool.hatch.build.targets.wheel.hooks.custom]
path = "tools/build_idl.py"
```

Where `tools/build_idl.py` executes the command shown above.

## Caching

Avoid regenerating files unnecessarily by comparing timestamps or hashing the
IDL files. Python’s `pathlib` and `hashlib` modules make this straightforward.

## Distribution

Ship the generated sources with your package so end users do not need the IDL
compiler. Include `intercom-dds` as a runtime dependency in `install_requires`.

```toml
[project]
name = "my-package"
version = "0.1.0"

[project.dependencies]
intercom-dds = ">=0.1"
```

Adjust the version constraint to match the runtime release you rely on.
