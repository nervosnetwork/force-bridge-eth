use crate::adapter::Adapter;

pub fn verify_manage_mode<T: Adapter>(data_loader: &T) {
    let owner = data_loader.load_script_args();
    if !data_loader.lock_script_exists_in_inputs(owner.as_ref()) {
        panic!("not authorized to unlock the cell");
    }
}
