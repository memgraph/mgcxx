#[cxx::bridge]
mod ffi {
    #[namespace = "shared"]
    struct Dummy {
        a: u8,
    }

    #[namespace = "dummy"]
    extern "Rust" {
        fn print_dummy(data: &Dummy);
    }
}

fn print_dummy(data: &ffi::Dummy) {
    println!("printing data");
    println!("{}", data.a);
}
