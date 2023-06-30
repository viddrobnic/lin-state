use lin_state::macros::Resource;

struct A;
impl lin_state::Resource for A {
    unsafe fn clone_state(&self) -> Self {
        A
    }

    unsafe fn set_cleanup_enabled(&mut self, cleanup_enabled: bool) {}
}

#[derive(Resource)]
struct C(A, A);

fn main() {
    println!("dober dan");
}
