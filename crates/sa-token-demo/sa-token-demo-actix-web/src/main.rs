//! Sa-Token-Rs Actix-Web Demo
//!
//! 框架映射：Quarkus → actix-web，Jackson → serde
//! 行为对齐：Java `sa-token-demo-springboot` 核心接口

mod controllers;
mod current;
mod satoken;
mod util;

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::middleware::{DefaultHeaders, from_fn};
use actix_web::{App, HttpServer, web};
use sa_token_web_actix::require_login;

use crate::controllers::{at_controller, login_controller, test_controller};
use crate::satoken::build_stp_util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_stp_util();
    let addr = ("0.0.0.0", 8082);

    println!("🚀 Sa-Token-Rs Actix Demo（Quarkus → actix-web）");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   示例: GET /acc/doLogin?name=zhang&pwd=123456");
    println!("   登录后请求头携带: satoken: <token>");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .wrap(
                DefaultHeaders::new()
                    .add((
                        HeaderName::from_static("x-frame-options"),
                        HeaderValue::from_static("SAMEORIGIN"),
                    ))
                    .add((
                        HeaderName::from_static("x-xss-protection"),
                        HeaderValue::from_static("1; mode=block"),
                    ))
                    .add((
                        HeaderName::from_static("x-content-type-options"),
                        HeaderValue::from_static("nosniff"),
                    ))
                    .add((
                        HeaderName::from_static("server"),
                        HeaderValue::from_static("sa-server"),
                    )),
            )
            // 公开接口
            .route("/acc/doLogin", web::get().to(login_controller::do_login))
            .route("/acc/doLogin", web::post().to(login_controller::do_login))
            .route("/acc/isLogin", web::get().to(login_controller::is_login))
            .route("/test/login", web::get().to(test_controller::login))
            .route("/test/login2", web::get().to(test_controller::login2))
            .route("/test/kickOut", web::get().to(test_controller::kick_out))
            .route("/test/test", web::get().to(test_controller::test))
            .route("/test/test2", web::get().to(test_controller::test2))
            // 需登录接口（RequireLogin 提取器 / middleware）
            .service(
                web::scope("/acc")
                    .wrap(from_fn(require_login))
                    .route("/tokenInfo", web::get().to(login_controller::token_info))
                    .route("/logout", web::get().to(login_controller::logout)),
            )
            .service(
                web::scope("/test")
                    .wrap(from_fn(require_login))
                    .route("/logout", web::get().to(test_controller::logout))
                    .route("/testRole", web::get().to(test_controller::test_role))
                    .route("/testJur", web::get().to(test_controller::test_jur))
                    .route("/session", web::get().to(test_controller::session))
                    .route("/tokenInfo", web::get().to(test_controller::token_info)),
            )
            .service(
                web::scope("/at")
                    .wrap(from_fn(require_login))
                    .route("/checkLogin", web::get().to(at_controller::check_login))
                    .route(
                        "/checkPermission",
                        web::get().to(at_controller::check_permission),
                    )
                    .route(
                        "/checkPermissionAnd",
                        web::get().to(at_controller::check_permission_and),
                    )
                    .route(
                        "/checkPermissionOr",
                        web::get().to(at_controller::check_permission_or),
                    )
                    .route("/checkRole", web::get().to(at_controller::check_role))
                    .route("/openSafe", web::get().to(at_controller::open_safe))
                    .route("/checkSafe", web::get().to(at_controller::check_safe)),
            )
    })
    .bind(addr)?
    .run()
    .await
}
