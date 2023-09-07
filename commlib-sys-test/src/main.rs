use app_helper::App;

pub mod proto {
    include!("../protos/out/proto.rs");
}

mod cli_conf;
mod cli_service;

mod cli_app_startup;
mod cli_manager;
//配置文件
mod config;
mod config_manager;
mod config_table;
fn main() {
    // panic hook
    std::panic::set_hook(Box::new(|panic_info| {
        println!(
            "panic info: {:?}, {:?}, panic occurred in {:?}",
            panic_info.payload().downcast_ref::<&str>(),
            panic_info.to_string(),
            panic_info.location()
        );
        log::error!(
            "panic info: {:?}, {:?}, panic occurred in {:?}",
            panic_info.payload().downcast_ref::<&str>(),
            panic_info.to_string(),
            panic_info.location()
        );
    }));

    //
    let arg_vec: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let mut app = App::new(&arg_vec, "clisrv");
    app.init(
        || cli_service::G_CLI_SERVICE.as_ref(),
        || {
            cli_app_startup::exec(&cli_service::G_CLI_SERVICE);
        },
    );
    app.run();
}
