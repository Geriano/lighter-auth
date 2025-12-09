use actix_web::{HttpResponse, Responder};

/// Convert anyhow::Result to impl Responder
/// This is a temporary bridge until proper error handling is implemented
pub struct AnyhowResponder<T>(pub anyhow::Result<T>);

impl<T> Responder for AnyhowResponder<T>
where
    T: Responder,
{
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        match self.0 {
            Ok(data) => data.respond_to(req).map_into_boxed_body(),
            Err(e) => {
                // Log the error
                eprintln!("Service error: {:?}", e);

                // Return generic internal server error
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string(),
                }))
            }
        }
    }
}
