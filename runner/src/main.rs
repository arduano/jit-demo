use runner::{build_complex_filter, interpreted, jit::build_module, read_data};

fn main() {
    let users = read_data();
    let filters = build_complex_filter();
    unsafe {
        let jit_fn = build_module(&filters);

        let filtered_users = interpreted::filter_vec_with_filters(&users, &filters);
        println!("Interpreted len: {}", filtered_users.len());

        let jit_filtered_users = jit_fn.execute(&users);
        println!("JIT len: {}", jit_filtered_users.len());
    }
}
