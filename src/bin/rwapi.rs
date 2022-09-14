use actix_cors::Cors;
use actix_web::{self, middleware::Logger, App, HttpServer};
use env_logger;
use rwlinux::api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "acti_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .wrap(Logger::default())
            .service(api::read_devmem)
            .service(api::write_devmem)
            .service(api::get_pci_devices)
            .service(api::get_pci_dev_config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
