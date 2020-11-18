use crate::adapter::Adapter;

use molecule::prelude::Entity;

pub fn verify_client_owner<T: Adapter>(data_loader: T) {
    let input_script = data_loader.load_input_script();
    let args = input_script.args().raw_data();

    if !args.is_empty() && !data_loader.check_input_owner(&args) {
        panic!("not owner lock");
    }
    if args.is_empty() {
        // eth-light-client requires the first output cell is eth_light_client_cell
        let output_script = data_loader.load_output_script();
        if input_script.as_slice() != output_script.as_slice() {
            panic!("owner changed from none to some")
        }
    }
}
