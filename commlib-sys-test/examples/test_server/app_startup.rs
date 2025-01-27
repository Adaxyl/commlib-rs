//!
//! G_APP_STARTUP
//!
//! example for resume:
//! '''
//!     G_APP_STARTUP.with(|g| {
//!         let mut startup = g.borrow_mut();
//!         startup.resume();
//!     });
//! '''

use std::sync::Arc;

use commlib_sys::listen_tcp_addr;
use commlib_sys::{ConnId, NetPacketGuard, NodeState, ServiceRs};
use commlib_sys::{G_SERVICE_NET, G_SERVICE_SIGNAL};

use app_helper::Startup;

use crate::test_conf::G_TEST_CONF;
use crate::test_manager::G_MAIN;

use super::test_service::TestService;
use super::test_service::G_TEST_SERVICE;

thread_local! {
    ///
    pub static G_APP_STARTUP: std::cell::RefCell<Startup> = {
        std::cell::RefCell::new(Startup::new("app"))
    };
}

///
pub fn resume(srv: &Arc<TestService>) {
    srv.run_in_service(Box::new(|| {
        //
        G_APP_STARTUP.with(|g| {
            let mut startup = g.borrow_mut();
            startup.resume();
        });
    }));
}

///
pub fn exec(srv: &Arc<TestService>) {
    //
    test_service_init(srv);

    //
    let srv2 = srv.clone();
    G_APP_STARTUP.with(|g| {
        let mut startup = g.borrow_mut();

        //
        startup.add_step("start network listen", move || {
            startup_network_listen(&srv2)
        });

        // run
        startup.exec();
    });
}

///
pub fn startup_network_listen(srv: &Arc<TestService>) -> bool {
    // TODO: let thread_num: u32 = 1;
    // TODO: let connection_limit: u32 = 0; // 0=no limit

    let conn_fn = |hd: ConnId| {
        log::info!("[hd={}] conn_fn", hd);

        //
        G_MAIN.with(|g| {
            let mut test_manager = g.borrow_mut();

            let push_encrypt_token = true; // 是否推送 encrypt token
            test_manager
                .c2s_proxy
                .on_incomming_conn(hd, push_encrypt_token);
        });
    };

    let pkt_fn = |hd: ConnId, pkt: NetPacketGuard| {
        log::info!("[hd={}] msg_fn", hd);

        G_MAIN.with(|g| {
            let mut test_manager = g.borrow_mut();
            test_manager.c2s_proxy.on_net_packet(hd, pkt);
        });
    };

    let close_fn = |hd: ConnId| {
        log::info!("[hd={}] close_fn", hd);
    };

    //
    app_helper::with_conf!(G_TEST_CONF, cfg, {
        let listener_id = listen_tcp_addr(
            srv,
            cfg.my.addr.clone(),
            cfg.my.port,
            conn_fn,
            pkt_fn,
            close_fn,
            &G_SERVICE_NET,
        );
        log::info!("listener {} ready.", listener_id);
    });

    //
    true
}

/// 初始化
fn test_service_init(srv: &Arc<TestService>) -> bool {
    let handle = srv.get_handle();

    // ctrl-c stop, DEBUG ONLY
    G_SERVICE_SIGNAL.listen_sig_int(G_TEST_SERVICE.as_ref(), || {
        println!("WTF!!!!");
    });
    log::info!("\nTest init ...\n");

    //
    app_helper::with_conf_mut!(G_TEST_CONF, cfg, { cfg.init(handle.xml_config()) });

    //
    handle.set_state(NodeState::Start);
    true
}
