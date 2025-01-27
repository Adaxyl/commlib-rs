#[cxx::bridge]
pub mod ffi_sig {

    unsafe extern "C++" {
        include!("signal_bindings.h");

        type SignalCallback = crate::SignalCallback;

        #[namespace = "commlib"]
        fn init_signal_handlers(cb1: SignalCallback, cb2: SignalCallback, cb3: SignalCallback);

        #[namespace = "commlib"]
        fn new_abc();

    }
}

#[repr(transparent)]
pub struct SignalCallback(pub extern "C" fn(sig: i32));

unsafe impl cxx::ExternType for SignalCallback {
    type Id = cxx::type_id!("SignalCallback");
    type Kind = cxx::kind::Trivial;
}
