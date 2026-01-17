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

import interface_types.*;
import java.lang.reflect.Method;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;

class InterfacesTests {

    @Test
    void interface_is_interface() {
        assertTrue(Reader.class.isInterface());
    }

    @Test
    void interface_has_abstract_methods() throws Exception {
        Method read = Reader.class.getMethod("read");
        assertNotNull(read);
        Method hasMore = Reader.class.getMethod("hasMore");
        assertNotNull(hasMore);
    }

    @Test
    void interface_inheritance() {
        assertTrue(ReadWriter.class.isInterface());
        assertTrue(Reader.class.isAssignableFrom(ReadWriter.class));
        assertTrue(Writer.class.isAssignableFrom(ReadWriter.class));
    }

    @Test
    void interface_inherited_has_own_method() throws Exception {
        Method reset = ReadWriter.class.getMethod("reset");
        assertNotNull(reset);
    }

    @Test
    void interface_method_signature_no_params() throws Exception {
        Method read = Reader.class.getMethod("read");
        assertEquals(0, read.getParameterCount());
        assertEquals(String.class, read.getReturnType());
    }

    @Test
    void interface_method_signature_with_params() throws Exception {
        Method add = Calculator.class.getMethod("add", int.class, int.class);
        assertEquals(2, add.getParameterCount());
        assertEquals(int.class, add.getReturnType());
    }

    @Test
    void interface_method_return_types() throws Exception {
        Method read = Reader.class.getMethod("read");
        assertEquals(String.class, read.getReturnType());
        Method hasMore = Reader.class.getMethod("hasMore");
        assertEquals(boolean.class, hasMore.getReturnType());
    }

    @Test
    void interface_void_return() throws Exception {
        Method flush = Writer.class.getMethod("flush");
        assertEquals(void.class, flush.getReturnType());
    }

    @Test
    void empty_interface() {
        assertTrue(Empty.class.isInterface());
        assertEquals(0, Empty.class.getDeclaredMethods().length);
    }

    @Test
    @Disabled("attributes not yet implemented")
    void interface_with_attributes() {
        // TODO: attributes should become getter/setter methods
        assertTrue(WithAttribute.class.isInterface());
    }

    @Test
    void interface_writer_parameter_types() throws Exception {
        Method write = Writer.class.getMethod("write", String.class);
        assertNotNull(write);
        assertEquals(1, write.getParameterCount());
        assertEquals(String.class, write.getParameterTypes()[0]);
    }

    @Test
    void interface_calculator_all_signatures() throws Exception {
        Method add = Calculator.class.getMethod("add", int.class, int.class);
        assertEquals(int.class, add.getReturnType());

        Method subtract = Calculator.class.getMethod("subtract", int.class, int.class);
        assertEquals(int.class, subtract.getReturnType());

        Method divide = Calculator.class.getMethod("divide", double.class, double.class);
        assertEquals(double.class, divide.getReturnType());
    }

    @Test
    void interface_attribute_types() throws Exception {
        assertTrue(WithAttribute.class.isInterface());
    }

    @Test
    void operation_failed_exception() {
        var e = new OperationFailed();
        assertTrue(e instanceof RuntimeException);
        e.setErrorCode(500);
        e.setReason("test");
        assertEquals(500, e.getErrorCode());
        assertEquals("test", e.getReason());
    }

    @Test
    void invalid_input_exception() {
        var e = new InvalidInput();
        assertTrue(e instanceof RuntimeException);
        e.setParameterName("param1");
        assertEquals("param1", e.getParameterName());
    }

    @Test
    void exception_can_be_raised() {
        assertThrows(OperationFailed.class, () -> {
            throw new OperationFailed(404, "not found");
        });
    }

    @Test
    void interface_with_out_params_exists() {
        assertTrue(WithOutParams.class.isInterface());
    }

    @Test
    void interface_with_raises_exists() {
        assertTrue(WithRaises.class.isInterface());
    }

    @Test
    void combined_features_interface() {
        assertTrue(CombinedFeatures.class.isInterface());
    }
}
