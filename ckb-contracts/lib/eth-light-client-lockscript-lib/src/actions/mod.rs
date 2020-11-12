use crate::adapter::Adapter;

use molecule::prelude::Entity;

pub fn verify_client_owner<T: Adapter>(data_loader: T) {
    let input_script = data_loader.load_input_script();
    let args = input_script.args().raw_data();

    // eth-light-client requires the first output cell is eth_light_client_cell
    if !args.is_empty() && !data_loader.check_input_owner(args.clone()) {
        panic!("not owner lock");
    }
    if args.is_empty() {
        let output_script = data_loader.load_output_script();
        assert_eq!(input_script.as_slice(), output_script.as_slice());
    }
}
