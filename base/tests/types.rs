extern crate gluon_base as base;
extern crate collect_mac;
extern crate pretty;

use std::ops::Deref;

use pretty::{Arena, DocAllocator};

use base::kind::Kind;
use base::types::*;

fn type_con<I, T>(s: I, args: Vec<T>) -> Type<I, T>
where
    I: Deref<Target = str>,
    T: From<Type<I, T>>,
{
    assert!(s.len() != 0);
    match s.parse() {
        Ok(b) => Type::Builtin(b),
        Err(()) if s.starts_with(char::is_lowercase) => {
            Type::Generic(Generic::new(s, Kind::typ()))
        }
        Err(()) => Type::App(Type::ident(s), args.into_iter().collect()),
    }
}

macro_rules! assert_eq_display {
    ($l: expr, $r: expr) => {
        let l = $l;
        let r = $r;
        if l != r {
            panic!("Assertion failed: {} != {}\nleft:\n{}\nright:\n{}",
                stringify!($l), stringify!($r), l, r);
        }
    }
}

#[test]
fn show_function() {
    let int: ArcType<&str> = Type::int();
    let int_int = Type::function(vec![int.clone()], int.clone());
    assert_eq_display!(format!("{}", int_int), "Int -> Int");

    assert_eq_display!(
        format!("{}", Type::function(vec![int_int.clone()], int.clone())),
        "(Int -> Int) -> Int"
    );

    assert_eq_display!(
        format!("{}", Type::function(vec![int.clone()], int_int.clone())),
        "Int -> Int -> Int"
    );
}

fn some_record() -> ArcType<&'static str> {
    let data = |s, a| ArcType::from(type_con(s, a));
    let f = Type::function(vec![data("a", vec![])], Type::string());

    let test = data("Test", vec![data("a", vec![])]);
    Type::record(
        vec![
            Field::new(
                "Test",
                Alias::new("Test", vec![Generic::new("a", Kind::typ())], f.clone()),
            ),
        ],
        vec![
            Field::new("x", Type::int()),
            Field::new("test", test.clone()),
            Field::new(
                "+",
                Type::function(vec![Type::int(), Type::int()], Type::int()),
            ),
        ],
    )
}

#[test]
fn show_record() {
    assert_eq_display!(
        format!("{}", Type::<&str, ArcType<&str>>::record(vec![], vec![])),
        "()"
    );
    let typ = Type::record(
        vec![],
        vec![Field::new("x", Type::<&str, ArcType<&str>>::int())],
    );
    assert_eq_display!(format!("{}", typ), "{ x : Int }");

    let data = |s, a| ArcType::from(type_con(s, a));
    let f = Type::function(vec![data("a", vec![])], Type::string());
    let typ = Type::record(
        vec![
            Field::new(
                "Test",
                Alias::new("Test", vec![Generic::new("a", Kind::typ())], f.clone()),
            ),
        ],
        vec![Field::new("x", Type::int())],
    );
    assert_eq_display!(format!("{}", typ), "{ Test a = a -> String, x : Int }");
    assert_eq_display!(
        format!("{}", some_record()),
        "{ Test a = a -> String, x : Int, test : Test a, (+) : Int -> Int -> Int }"
    );
    let typ = Type::record(
        vec![
            Field::new(
                "Test",
                Alias::new("Test", vec![Generic::new("a", Kind::typ())], f.clone()),
            ),
        ],
        vec![],
    );
    assert_eq_display!(format!("{}", typ), "{ Test a = a -> String }");
}

#[test]
fn show_record_multi_line() {

    let data = |s, a| ArcType::from(type_con(s, a));
    let f = Type::function(vec![data("a", vec![])], Type::string());
    let test = data("Test", vec![data("a", vec![])]);
    let typ = Type::record(
        vec![
            Field::new(
                "Test",
                Alias::new("Test", vec![Generic::new("a", Kind::typ())], f.clone()),
            ),
        ],
        vec![
            Field::new("x", Type::int()),
            Field::new(
                "test",
                Type::function(
                    vec![
                        data("Test", vec![Type::int(), f.clone()]),
                        Type::float(),
                        f.clone(),
                        f.clone(),
                    ],
                    f.clone(),
                ),
            ),
            Field::new(
                "record_looooooooooooooooooooooooooooooooooong",
                some_record(),
            ),
            Field::new("looooooooooooooooooooooooooooooooooong_field", test.clone()),
        ],
    );
    let expected = r#"{
    Test a = a -> String,
    x : Int,
    test : Test Int (a -> String)
        -> Float
        -> (a -> String)
        -> (a -> String)
        -> a
        -> String,
    record_looooooooooooooooooooooooooooooooooong : {
        Test a = a -> String,
        x : Int,
        test : Test a,
        (+) : Int -> Int -> Int
    },
    looooooooooooooooooooooooooooooooooong_field : Test a
}"#;

    assert_eq_display!(format!("{}", typ), expected);
}

#[test]
fn show_variant() {
    let typ: ArcType<&str> = Type::variant(vec![
        Field::new("A", Type::function(vec![Type::int()], Type::ident("A"))),
        Field::new("B", Type::ident("A")),
    ]);
    assert_eq_display!(format!("{}", typ), "| A Int | B");
}

#[test]
fn show_kind() {
    let two_args = Kind::function(Kind::typ(), Kind::function(Kind::typ(), Kind::typ()));
    assert_eq_display!(format!("{}", two_args), "Type -> Type -> Type");
    let function_arg = Kind::function(Kind::function(Kind::typ(), Kind::typ()), Kind::typ());
    assert_eq_display!(format!("{}", function_arg), "(Type -> Type) -> Type");
}

#[test]
fn show_polymorphic_record() {
    let fields = vec![Field::new("x", Type::string())];
    let typ: ArcType<&str> = Type::poly_record(vec![], fields, Type::ident("r"));
    assert_eq_display!(format!("{}", typ), "{ x : String | r }");
}

#[test]
fn show_polymorphic_record_associated_type() {
    let type_fields = vec![
        Field::new(
            "Test",
            Alias::new(
                "Test",
                vec![Generic::new("a", Kind::typ())],
                Type::ident("a"),
            ),
        ),
    ];
    let typ: ArcType<&str> = Type::poly_record(type_fields, vec![], Type::ident("r"));
    assert_eq_display!(format!("{}", typ), "{ Test a = a | r }");
}

#[test]
fn break_record() {
    let data = |s, a| ArcType::from(type_con(s, a));

    let test = data("Test", vec![data("a", vec![])]);
    let typ: ArcType<&str> = Type::record(
        vec![],
        vec![
            Field::new("x", Type::int()),
            Field::new("test", test.clone()),
            Field::new(
                "+",
                Type::function(vec![Type::int(), Type::int()], Type::int()),
            ),
        ],
    );
    let arena = Arena::new();
    let typ = arena
        .text("aaaaaaaaabbbbbbbbbbcccccccccc ")
        .append(pretty_print(&arena, &typ))
        .append(arena.newline());
    assert_eq_display!(
        format!("{}", typ.1.pretty(80)),
        r#"aaaaaaaaabbbbbbbbbbcccccccccc {
    x : Int,
    test : Test a,
    (+) : Int -> Int -> Int
}
"#
    );
}
