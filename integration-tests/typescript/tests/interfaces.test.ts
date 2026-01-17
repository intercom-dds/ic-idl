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

import { describe, expect, test } from "bun:test";
import type {
  Reader,
  Writer,
  ReadWriter,
  Calculator,
  Empty,
  WithAttribute,
  WithOutParams,
  WithRaises,
  CombinedFeatures,
} from "../generated/interfaceTypes";
import { OperationFailed, InvalidInput } from "../generated/interfaceTypes";

describe("interfaces", () => {
  test("Reader interface can be implemented", () => {
    const reader: Reader = {
      read: () => "data",
      hasMore: () => true,
    };
    expect(reader.read()).toBe("data");
    expect(reader.hasMore()).toBe(true);
  });

  test("Writer interface can be implemented", () => {
    let written = "";
    const writer: Writer = {
      write: (data: string) => {
        written = data;
      },
      flush: () => {},
    };
    writer.write("test");
    expect(written).toBe("test");
  });

  test("ReadWriter extends Reader and Writer", () => {
    const rw: ReadWriter = {
      read: () => "data",
      hasMore: () => false,
      write: () => {},
      flush: () => {},
      reset: () => {},
    };
    expect(rw.read()).toBe("data");
    expect(rw.hasMore()).toBe(false);
  });

  test("Calculator interface methods", () => {
    const calc: Calculator = {
      add: (a, b) => a + b,
      subtract: (a, b) => a - b,
      divide: (a, b) => a / b,
    };
    expect(calc.add(2, 3)).toBe(5);
    expect(calc.subtract(5, 3)).toBe(2);
    expect(calc.divide(10, 2)).toBe(5);
  });

  test("Empty interface exists", () => {
    const empty: Empty = {};
    expect(empty).toBeDefined();
  });

  test("WithAttribute has readonly and mutable attributes", () => {
    const obj: WithAttribute = {
      name: "readonly",
      count: 0,
    };
    expect(obj.name).toBe("readonly");
    obj.count = 10;
    expect(obj.count).toBe(10);
  });

  test("WithOutParams returns output parameters", () => {
    const impl: WithOutParams = {
      getValues: () => ({ x: 10, y: 20 }),
      swap: (a, b) => ({ a: b, b: a }),
      process: (input) => ({ $return: input * 2, result: "done" }),
      mixedParams: (_name, counter) => ({
        counter: counter + 1,
        success: true,
      }),
    };

    const values = impl.getValues();
    expect(values.x).toBe(10);
    expect(values.y).toBe(20);

    const swapped = impl.swap(1, 2);
    expect(swapped.a).toBe(2);
    expect(swapped.b).toBe(1);

    const processed = impl.process(5);
    expect(processed.$return).toBe(10);
    expect(processed.result).toBe("done");

    const mixed = impl.mixedParams("test", 5);
    expect(mixed.counter).toBe(6);
    expect(mixed.success).toBe(true);
  });

  test("WithRaises interface exists", () => {
    const impl: WithRaises = {
      safeOperation: () => {},
      riskyOperation: () => {},
      complexOperation: () => {},
      compute: (value) => value * 2,
    };
    expect(impl.compute(5)).toBe(10);
  });

  test("CombinedFeatures interface exists", () => {
    const impl: CombinedFeatures = {
      doWork: () => 42,
      update: (value) => value + 1,
    };
    expect(impl.doWork("task")).toBe(42);
    expect(impl.update(10)).toBe(11);
  });
});

describe("exceptions", () => {
  test("OperationFailed extends Error", () => {
    const err = new OperationFailed(500, "server error");
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(OperationFailed);
    expect(err.name).toBe("OperationFailed");
    expect(err.errorCode).toBe(500);
    expect(err.reason).toBe("server error");
  });

  test("OperationFailed can be thrown and caught", () => {
    expect(() => {
      throw new OperationFailed(404, "not found");
    }).toThrow(OperationFailed);
  });

  test("InvalidInput extends Error", () => {
    const err = new InvalidInput("userId");
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(InvalidInput);
    expect(err.name).toBe("InvalidInput");
    expect(err.parameterName).toBe("userId");
  });

  test("InvalidInput can be thrown and caught", () => {
    expect(() => {
      throw new InvalidInput("email");
    }).toThrow(InvalidInput);
  });

  test("exceptions can be caught specifically", () => {
    const throwOp = () => {
      throw new OperationFailed(500, "fail");
    };

    try {
      throwOp();
    } catch (e) {
      if (e instanceof OperationFailed) {
        expect(e.errorCode).toBe(500);
      } else {
        throw new Error("wrong exception type");
      }
    }
  });
});
