// ref: https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-system-script-list/0024-ckb-system-script-list.md#simple-udt
cfg_if::cfg_if! {
    if #[cfg(feature = "lina")] {
        pub const SUDT_CODE_HASH: [u8; 32] = [
            94, 122, 54, 167, 126, 104, 238, 204, 1, 61, 250, 47, 230, 162, 63, 59, 108, 52, 75, 4,
            0, 88, 8, 105, 74, 230, 221, 69, 238, 164, 207, 213,
        ];
        pub const SUDT_HASH_TYPE: u8 = 1;
    } else if #[cfg(feature = "aggron")] {
        pub const SUDT_CODE_HASH: [u8; 32] = [
            197, 229, 220, 242, 21, 146, 95, 126, 244, 223, 175, 95, 75, 79, 16, 91, 195, 33, 192,
            39, 118, 214, 231, 213, 42, 29, 179, 252, 217, 208, 17, 164,
        ];
        pub const SUDT_HASH_TYPE: u8 = 1;
    } else if #[cfg(feature = "devnet")] {
        pub const SUDT_CODE_HASH: [u8; 32] = [
            19, 19, 160, 234, 165, 113, 169, 22, 142, 68, 206, 186, 26, 13, 10, 50, 136, 64, 217, 222, 67, 170, 178, 56, 138, 247, 200, 96, 181, 124, 154, 12,
        ];
        pub const SUDT_HASH_TYPE: u8 = 1;
    }
}

pub const UDT_LEN: usize = 16;

pub const CKB_UNITS: u64 = 100_000_000;

pub const CONFIRM: usize = 15;
