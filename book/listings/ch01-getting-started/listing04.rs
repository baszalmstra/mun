use mun_runtime::RuntimeBuilder;
use std::{cell::RefCell, env, rc::Rc};

fn main() {
    let lib_path = env::args().nth(1).expect("Expected path to a Mun library.");

    let mut runtime = RuntimeBuilder::new(lib_path)
        .spawn()
        .expect("Failed to spawn Runtime");

    loop {
        {
            let runtime_ref = runtime.borrow();
            let arg: i64 = runtime_ref.invoke("arg", ()).unwrap();
            let result: i64 = runtime_ref.invoke("fibonacci", (arg,)).unwrap();
            println!("fibonacci({}) = {}", arg, result);
        }
        runtime.borrow_mut().update();
    }
}
