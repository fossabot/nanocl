use ntex::web;
use ntex::http;

use nanocl_utils::ntex::middlewares;
use nanocl_error::http::HttpError;

use crate::version;

#[cfg(feature = "dev")]
mod openapi;

mod rule;

pub async fn unhandled() -> Result<web::HttpResponse, HttpError> {
  Err(HttpError {
    status: http::StatusCode::NOT_FOUND,
    msg: "Route or Method unhandled".to_owned(),
  })
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  #[cfg(feature = "dev")]
  {
    use openapi::ApiDoc;
    use utoipa::OpenApi;
    use nanocl_utils::ntex::swagger;
    let api_doc = ApiDoc::openapi();
    std::fs::write(
      "./bin/ncproxy/specs/swagger.yaml",
      api_doc.to_yaml().expect("Unable to convert ApiDoc to yaml"),
    )
    .expect("Unable to write swagger.yaml");
    let swagger_conf =
      swagger::SwaggerConfig::new(api_doc, "/explorer/swagger.json");
    config.service(
      web::scope("/explorer/")
        .state(swagger_conf)
        .configure(swagger::register),
    );
  }
  let versioning = middlewares::Versioning::new(version::VERSION).finish();
  config.service(
    web::scope("/{version}")
      .wrap(versioning)
      .configure(rule::ntex_config),
  );
}
