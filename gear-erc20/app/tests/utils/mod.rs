use gtest::{Program, System};
use gstd::Encode;

pub const USERS: &[u64] = &[3, 4, 5];

#[macro_export]
macro_rules! send_request {
    (ft: $ft: expr, user: $user: expr, service_name: $name: literal, action: $action: literal, payload: ($($val: expr),*)) => {
        {
            let request = [
                $name.encode(),
                $action.to_string().encode(),
                ( $( $val, )*).encode(),
            ]
            .concat();

            $ft.send_bytes($user, request)

        }

    };
}

pub fn init(sys: &System) -> Program {
    let ft = Program::from_file(
        sys,
        "../target/wasm32-unknown-unknown/release/erc20_wasm.opt.wasm",
    );

    let init = ("TokenName".to_owned(), "TokenSymbol".to_owned(), 10_u8);
    let request = ["New".encode(), init.encode()].concat();
    let res = ft.send_bytes(USERS[0], request);
    assert!(!res.main_failed());

    ft
}
