//!
//! CliManager
//!

use commlib_sys::service_net::PacketType;
use commlib_sys::{NetProxy, G_SERVICE_NET};

thread_local! {
    ///
    pub static G_MAIN: std::cell::RefCell<CliManager> = {
        std::cell::RefCell::new(CliManager::new())
    };
}

///
pub struct CliManager {
    pub proxy: NetProxy,
}

impl CliManager {
    ///
    pub fn new() -> CliManager {
        CliManager {
            proxy: NetProxy::new(PacketType::Server, &G_SERVICE_NET),
        }
    }
}
