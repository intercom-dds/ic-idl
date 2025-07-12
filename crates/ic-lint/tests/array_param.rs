use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_array_as_parameter() {
    let source = r"
        interface Test {
            void process_array(in long data[10]);
            void process_matrix(in double matrix[3][3]);
            void process_string_array(in string names[]);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_no_warning_for_sequences() {
    let source = r"
        interface Test {
            void process_sequence(in sequence<long> data);
            void process_bounded_seq(in sequence<double, 10> values);
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for sequence parameters");
}

#[test]
fn test_array_in_struct_member() {
    let source = r"
        struct Data {
            long values[100];
        };
        
        interface Test {
            void process_data(in Data d);
        };
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for arrays in struct members"
    );
}

#[test]
fn test_multiple_array_params() {
    let source = r"
        interface Calculator {
            double dot_product(in double vec1[3], in double vec2[3]);
            void matrix_multiply(
                in double a[4][4],
                in double b[4][4],
                out double result[4][4]
            );
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_array_with_different_directions() {
    let source = r"
        interface Buffer {
            void fill_buffer(out octet buffer[1024]);
            void process_buffer(inout octet buffer[512]);
        };
    ";

    assert_snapshot!(test_lint(source));
}
