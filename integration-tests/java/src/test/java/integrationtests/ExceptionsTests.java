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

package integrationtests;

import static org.junit.jupiter.api.Assertions.*;

import exception_types.*;
import org.junit.jupiter.api.Test;

class ExceptionsTests {

    @Test
    void simpleError_extendsRuntimeException() {
        var e = new SimpleError();
        assertInstanceOf(RuntimeException.class, e);
    }

    @Test
    void simpleError_defaults() {
        var e = new SimpleError();
        assertEquals(0, e.getErrorCode());
        assertEquals("", e.getMessage());
    }

    @Test
    void simpleError_parameterizedConstructor() {
        var e = new SimpleError(404, "Not found");
        assertEquals(404, e.getErrorCode());
        assertEquals("Not found", e.getMessage());
    }

    @Test
    void simpleError_setters() {
        var e = new SimpleError();
        e.setErrorCode(500);
        e.setMessage("Internal error");
        assertEquals(500, e.getErrorCode());
        assertEquals("Internal error", e.getMessage());
    }

    @Test
    void simpleError_copyConstructor() {
        var e1 = new SimpleError(400, "Bad request");
        var e2 = new SimpleError(e1);
        assertEquals(e1.getErrorCode(), e2.getErrorCode());
        assertEquals(e1.getMessage(), e2.getMessage());
        assertNotSame(e1, e2);
    }

    @Test
    void simpleError_canBeThrown() {
        var thrown = assertThrows(SimpleError.class, () -> {
            throw new SimpleError(403, "Forbidden");
        });
        assertEquals(403, thrown.getErrorCode());
        assertEquals("Forbidden", thrown.getMessage());
    }

    @Test
    void simpleError_canBeCaught() {
        try {
            throw new SimpleError(401, "Unauthorized");
        } catch (SimpleError e) {
            assertEquals(401, e.getErrorCode());
        }
    }

    @Test
    void validationError_extendsRuntimeException() {
        var e = new ValidationError();
        assertInstanceOf(RuntimeException.class, e);
    }

    @Test
    void validationError_defaults() {
        var e = new ValidationError();
        assertEquals("", e.getFieldName());
        assertEquals("", e.getErrorMessage());
        assertEquals(0, e.getPosition());
    }

    @Test
    void validationError_parameterizedConstructor() {
        var e = new ValidationError("email", "Invalid format", 10);
        assertEquals("email", e.getFieldName());
        assertEquals("Invalid format", e.getErrorMessage());
        assertEquals(10, e.getPosition());
    }

    @Test
    void validationError_setters() {
        var e = new ValidationError();
        e.setFieldName("username");
        e.setErrorMessage("Too short");
        e.setPosition(5);
        assertEquals("username", e.getFieldName());
        assertEquals("Too short", e.getErrorMessage());
        assertEquals(5, e.getPosition());
    }

    @Test
    void validationError_copyConstructor() {
        var e1 = new ValidationError("password", "Weak", 0);
        var e2 = new ValidationError(e1);
        assertEquals(e1.getFieldName(), e2.getFieldName());
        assertEquals(e1.getErrorMessage(), e2.getErrorMessage());
        assertEquals(e1.getPosition(), e2.getPosition());
        assertNotSame(e1, e2);
    }

    @Test
    void validationError_canBeThrown() {
        var thrown = assertThrows(ValidationError.class, () -> {
            throw new ValidationError("age", "Must be positive", 3);
        });
        assertEquals("age", thrown.getFieldName());
    }

    @Test
    void detailedError_extendsRuntimeException() {
        var e = new DetailedError();
        assertInstanceOf(RuntimeException.class, e);
    }

    @Test
    void detailedError_defaults() {
        var e = new DetailedError();
        assertEquals(0, e.getCode());
        assertEquals("", e.getMessage());
        assertEquals("", e.getDetails());
        assertFalse(e.getRecoverable());
    }

    @Test
    void detailedError_parameterizedConstructor() {
        var e = new DetailedError(500, "Server error", "Stack trace here", true);
        assertEquals(500, e.getCode());
        assertEquals("Server error", e.getMessage());
        assertEquals("Stack trace here", e.getDetails());
        assertTrue(e.getRecoverable());
    }

    @Test
    void detailedError_setters() {
        var e = new DetailedError();
        e.setCode(503);
        e.setMessage("Service unavailable");
        e.setDetails("Retry later");
        e.setRecoverable(true);
        assertEquals(503, e.getCode());
        assertEquals("Service unavailable", e.getMessage());
        assertEquals("Retry later", e.getDetails());
        assertTrue(e.getRecoverable());
    }

    @Test
    void detailedError_copyConstructor() {
        var e1 = new DetailedError(404, "Not found", "Resource missing", false);
        var e2 = new DetailedError(e1);
        assertEquals(e1.getCode(), e2.getCode());
        assertEquals(e1.getMessage(), e2.getMessage());
        assertEquals(e1.getDetails(), e2.getDetails());
        assertEquals(e1.getRecoverable(), e2.getRecoverable());
        assertNotSame(e1, e2);
    }

    @Test
    void emptyError_extendsRuntimeException() {
        var e = new EmptyError();
        assertInstanceOf(RuntimeException.class, e);
    }

    @Test
    void emptyError_canBeInstantiated() {
        var e = new EmptyError();
        assertNotNull(e);
    }

    @Test
    void emptyError_copyConstructor() {
        var e1 = new EmptyError();
        var e2 = new EmptyError(e1);
        assertNotSame(e1, e2);
    }

    @Test
    void emptyError_canBeThrown() {
        assertThrows(EmptyError.class, () -> {
            throw new EmptyError();
        });
    }
}
