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

use app_helper::Startup;
use commlib_sys::service_net::TcpConn;
use commlib_sys::{connect_to_tcp_server, data_schema, G_SERVICE_NET};
use commlib_sys::{ConnId, NetPacketGuard, ServiceRs};
use std::sync::Arc;

use super::cli_service::CliService;
use crate::cli_conf::G_CLI_CONF;
use crate::cli_manager::G_MAIN;
use crate::config::role_table::{self, RoleTable};
use crate::config_manager::ConfigManager;
use crate::config_table::ConfigCid;

thread_local! {
    ///
    pub static G_APP_STARTUP: std::cell::RefCell<Startup> = {
        std::cell::RefCell::new(Startup::new("app"))
    };
}

///
pub fn resume(srv: &Arc<CliService>) {
    srv.run_in_service(Box::new(|| {
        //
        G_APP_STARTUP.with(|g| {
            let mut startup = g.borrow_mut();
            startup.resume();
        });
    }));
}

///
pub fn exec(srv: &Arc<CliService>) {
    // pre-startup, main manager init
    G_MAIN.with(|g| {
        let mut main_manager = g.borrow_mut();
        main_manager.init(srv);
    });

    // startup step by step
    let srv2 = srv.clone();
    G_APP_STARTUP.with(|g| {
        let mut startup = g.borrow_mut();
        //配置文件读取
        startup.add_step("load config table", move || do_load_config_data(&srv2));
        //
        // startup.add_step("connect", move || do_connect_to_test_server(&srv2));

        // run startup
        startup.exec();
    });

    // startup over, main manager lazy init
    G_MAIN.with(|g| {
        let mut main_manager = g.borrow_mut();
        main_manager.lazy_init(srv);
    });
    let mut binding = ConfigManager::get_instance();
    let mut g = binding.lock().unwrap();
    let data = &mut *g;
    data.init();
}

///
pub fn do_connect_to_test_server(srv: &Arc<CliService>) -> bool {
    //
    let raddr = app_helper::with_conf!(G_CLI_CONF, cfg, {
        std::format!("{}:{}", cfg.remote.addr, cfg.remote.port)
    });

    let conn_fn = |conn: Arc<TcpConn>| {
        let hd = conn.hd;
        log::info!("[hd={}] conn_fn", hd);

        //
        G_MAIN.with(|g| {
            let mut cli_manager = g.borrow_mut();

            let push_encrypt_token = false;
            cli_manager
                .proxy
                .on_incomming_conn(conn.as_ref(), push_encrypt_token);
        });
    };

    let pkt_fn = |conn: Arc<TcpConn>, pkt: NetPacketGuard| {
        let hd = conn.hd;
        log::info!("[hd={}] msg_fn", hd);

        G_MAIN.with(|g| {
            let mut main_manager = g.borrow_mut();
            main_manager.proxy.on_net_packet(conn.as_ref(), pkt);
        });
    };

    let close_fn = |hd: ConnId| {
        log::info!("[hd={}] close_fn", hd);

        G_MAIN.with(|g| {
            let mut main_manager = g.borrow_mut();
            main_manager.proxy.on_hd_lost(hd);
        });
    };

    //
    let hd_opt = connect_to_tcp_server(
        srv,
        "cli",
        raddr.as_str(),
        conn_fn,
        pkt_fn,
        close_fn,
        &G_SERVICE_NET,
    );

    //
    hd_opt.is_some()
}
pub fn do_load_config_data(srv: &Arc<CliService>) -> bool {
    let callback = Box::new(|ds: Box<data_schema::DataSchema>| {
        // 在闭包函数内处理加载完成后的操作
        println!("Data schema loaded with");
        // 处理 data_schema 对象
        let mut binding = ConfigManager::get_instance();
        let mut g = binding.lock().unwrap();
        let data = &mut *g;
        data.reload_all(ds);
    });
    data_schema::load_data_schema_from_xml(srv, "data\\", callback);

    // 获取配置项映射
    let binding = ConfigManager::get_instance();
    let mut ac = binding.lock().unwrap();
    let cc = &mut *ac;
    //let ct = cc.get_config_table(ConfigCid::Cid_Role);

    let role_binding = RoleTable::get_instance();
    let mut role_ac = role_binding.lock().unwrap();
    let role_cc = &mut *role_ac;
    let config_map = role_cc.get_role_configs();
    for (key, value) in config_map {
        println!("Key: {}, Name: {}, ID: {}", key, value.name, value.id);
    }

    true
}
