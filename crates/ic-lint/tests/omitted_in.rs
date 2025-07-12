use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_parameters_without_direction() {
    let source = r"
        interface Service {
            void process(string data);
            long calculate(double x, double y);
            string format(long value, boolean uppercase);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_mixed_direction_specifications() {
    let source = r"
        interface Buffer {
            void write(in octet data);
            void read(out octet data);
            void modify(inout string text);
            void broken(long size);  // Missing direction
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_all_parameters_with_direction() {
    let source = r"
        interface Correct {
            void send(in string message);
            void receive(out string message);
            void transform(inout sequence<long> data);
            boolean compare(in string a, in string b);
        };
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn when all parameters have directions"
    );
}

#[test]
fn test_oneway_operations() {
    let source = r"
        interface Async {
            oneway void notify(string event);
            oneway void log(long level, string message);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_complex_parameter_types() {
    let source = r"
        struct Data {
            long id;
            string name;
        };
        
        interface DataService {
            void store(Data item);
            Data retrieve(long id);
            void update(long id, Data item);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_readonly_attribute() {
    let source = r"
        interface Config {
            readonly attribute string version;
            attribute long timeout;
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for attributes");
}
