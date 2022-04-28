extern crate openssl;
#[macro_use]
extern crate log;
extern crate log4rs;

use std::str;
use std::ffi::CStr;

use serde_derive::{Deserialize, Serialize};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_files as afs;

use log::{error, info, warn};

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use mysql::prelude::*;
use mysql::*;

mod ecall;
mod endpoint;
mod persistence;

static ENCLAVE_FILE: &'static str = "libenclave_ks.signed.so";

#[no_mangle]
pub extern "C"
fn oc_print(some_string: *const c_char) -> sgx_status_t {
    let c_str: &CStr = unsafe { CStr::from_ptr(some_string)};
    let result = c_str.to_str();
    match result {
        Ok(v) => println!("enclave: {}", v),
        Err(e) => {
            let plaintext = c_str.to_bytes();
            println!("enclave: {:?}", plaintext);        
        }
    }
    return sgx_status_t::SGX_SUCCESS;    
}

fn init_enclave() -> SgxEnclave {
    error!("{}", "abc");
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    let sgxResult = SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr);
    match sgxResult {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            return r;
        },
        Err(x) => {
            panic!("[-] Init Enclave Failed {}!", x.as_str());
        },
    };
}

fn init_enclave_and_genkey() -> SgxEnclave {
    let enclave = init_enclave();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let result = unsafe {
        ecall::ec_gen_key(enclave.geteid(), &mut retval)
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {},
        _ => panic!("Enclave generate key-pair failed!")
    }
    return enclave;
}

fn init_db_pool() -> Pool {
    let ops = Opts::from_url("mysql://chanrw:Oracle!23@localhost:3306/keysafe").unwrap();
    let pool = mysql::Pool::new(ops).unwrap();
    return pool;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    println!("logging!");
    let edata: web::Data<endpoint::AppState> = web::Data::new(endpoint::AppState{
        enclave: init_enclave_and_genkey(),
        db_pool: init_db_pool()
    });
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("certs/MyKey.key", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("certs/MyCertificate.crt").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&edata))
            .service(endpoint::exchange_key)
            .service(endpoint::auth)
            .service(endpoint::auth_confirm)
            .service(endpoint::info)
            .service(endpoint::register_mail_auth)
            .service(endpoint::register_mail)
            .service(endpoint::register_gauth)
            .service(endpoint::register_password)
            .service(endpoint::seal)
            .service(endpoint::unseal)
            .service(endpoint::delegate)
            // .service(endpoint::notify_user)
            // .service(endpoint::prove_user)
            // .service(endpoint::prove_code)
            .service(endpoint::hello)
            // .service(endpoint::require_secret)
            .service(afs::Files::new("/", "./public").index_file("index.html"))
    })
    .bind_openssl("0.0.0.0:30000", builder)?
    .run()
    .await
}
