#![feature(struct_variant)]

extern crate nickel;
extern crate serialize;
extern crate sync;
extern crate mysql;

use nickel::{ Nickel, HttpRouter, StaticFilesHandler };

mod params_body_parser;
mod authentication;
mod config;
mod cookies_parser;
mod database;
mod answer;
mod parse_utils;
mod handlers;
mod photo_store;
mod photo_event;

fn main() {
    let cfg = config::load_or_default( &Path::new( "../etc/photometer.cfg" ) );
    let mut server = Nickel::new();
    let mut authentication_router = Nickel::router();
    let mut router = Nickel::router();

    let db = database::create_db_connection(  
        cfg.db_name.clone(),
        cfg.db_user.clone(),
        cfg.db_password.clone(),
        cfg.db_min_connections,
        cfg.db_max_connections
    ).unwrap_or_else( |e| { fail!( e ) } );


    router.get( "/hello", handlers::hello );
    router.post( "/upload", handlers::upload_photo );

    authentication_router.post( "/login", handlers::login ) ;
    authentication_router.post( "/join_us", handlers::join_us ) ;

    server.utilize( config::middleware( &cfg ) );
    server.utilize( authentication::create_session_store() );
    server.utilize( db );
    server.utilize( params_body_parser::middleware() );
    server.utilize( photo_store::middleware( &cfg.photo_store_path ) );
    server.utilize( cookies_parser::middleware() );
    server.utilize( StaticFilesHandler::new( cfg.static_files_path.as_slice() ) );
    server.utilize( authentication_router );
    server.utilize( authentication::middleware( &cfg.login_page_path ) );
    server.utilize( router );

    server.listen( cfg.server_ip(), cfg.server_port );
}
