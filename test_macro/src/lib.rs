use generate_macro::tonic_rpc;

#[tonic_rpc]
trait Foo {
    fn bar(x: i32) -> String;
}
