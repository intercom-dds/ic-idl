// Copyright 2026 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

import { spawnSync } from "bun";
import { existsSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";

const thisDir = import.meta.dir;
const integrationTestsDir = dirname(thisDir);
const rootDir = dirname(integrationTestsDir);
const corpusDir = join(integrationTestsDir, "corpus");
const generatedDir = join(
  rootDir,
  "target",
  "integration-tests",
  "typescript",
  "generated",
);

const exeName = process.platform === "win32" ? "ic-idl.exe" : "ic-idl";
const compilerPath =
  process.env.IDL_COMPILER ?? join(rootDir, "target", "debug", exeName);

if (!existsSync(generatedDir)) {
  mkdirSync(generatedDir, { recursive: true });
}

const result = spawnSync([
  compilerPath,
  "--typescript-out",
  generatedDir,
  corpusDir,
]);

if (result.error) {
  console.error(`Failed to execute ${compilerPath}: ${result.error.message}`);
  process.exit(1);
}

if (result.exitCode !== 0) {
  console.error("Failed to generate TypeScript from IDL:");
  console.error(result.stderr.toString());
  process.exit(1);
}
