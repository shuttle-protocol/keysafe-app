extern crate sgx_types;
extern crate sgx_urts;
extern crate openssl;

use std::sync::Mutex;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use serde_derive::{Deserialize, Serialize};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn exchange_key(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                     some_string: *const u8, len: usize) -> sgx_status_t;
}

fn init_enclave() -> SgxEnclave {
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

async fn app_seal_key(safeMessage: &SafeMessage) {
    println!("{}", safeMessage.secret);
}

async fn app_unseal_key(safeMessage: &SafeMessage) {
    println!("{}", safeMessage.secret);
}

#[derive(Deserialize, Serialize)]
struct SafeMessage {
    secret: String
}

struct AppState {
    enclave: SgxEnclave
}

#[get("/")]
async fn hello() -> impl Responder {
    // for health check
    HttpResponse::Ok().body("Hello world!")
}

#[get("/exchange_key/{user_pub_key}")]
async fn app_exchange_key(
    path: web::Path<(String,)>,
    endex: web::Data<AppState>
) -> String {
    let e = &endex.enclave;
    println!("enclave id {}, user pub key {}.", e.geteid(), path.into_inner().0);
    String::from("SgxPubKey123")
}

#[post("/seal")]
async fn seal(
    req_body: String,
    endex: web::Data<AppState>
) -> impl Responder {
    let e = &endex.enclave;
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let edata: web::Data<AppState> = web::Data::new(AppState{
        enclave: init_enclave()
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&edata))
            .service(app_exchange_key)
            .service(seal)
            .service(hello)
    })
    .bind("0.0.0.0:12345")?
    .run()
    .await
}