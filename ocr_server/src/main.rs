use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use leptess::LepTess;


async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

async fn ocr(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap();
        let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));
        let mut file_bytes = Vec::new();

        // Collecting file bytes
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_bytes.extend_from_slice(&data);
        }

        // Write file bytes to file
        match std::fs::write(&filepath, &file_bytes) {
            Ok(_) => {
                // OCR processing
                let mut lt = LepTess::new(Some("./tests/tessdata"), "eng").unwrap();
                let _ = lt.set_image(&filepath);
                let ocr_text = match lt.get_utf8_text() {
                    Ok(text) => text,
                    Err(_) => return HttpResponse::InternalServerError().body("OCR processing error"),
                };

                // Cleanup: Remove the temporary file after processing
                std::fs::remove_file(&filepath).unwrap();

                return HttpResponse::Ok().body(ocr_text);
            },
            Err(e) => return HttpResponse::InternalServerError().body(format!("File write error: {}", e)),
        }
    }

    HttpResponse::BadRequest().body("No file found")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::resource("/ocr")
                    .route(web::post().to(ocr))
            )
            .route("/", web::get().to(hello_world))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


