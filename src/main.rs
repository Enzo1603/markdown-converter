#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::response::Response;
use rocket::Data;
use rocket_dyn_templates::{context, Template};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use std::fs;
use std::io::Cursor;
use std::process::Command;

#[get("/")]
fn home() -> Redirect {
    Redirect::to(uri!(markdown_to_html))
}

#[get("/to/html")]
fn markdown_to_html() -> Template {
    Template::render("markdown_to_html", context! {})
}

#[post("/to/html/convert", data = "<data>")]
async fn convert_markdown_to_html(
    content_type: &ContentType,
    data: Data<'_>,
) -> Result<Response<'static>, (Status, String)> {
    let mut options = MultipartFormDataOptions::new();

    options
        .allowed_fields
        .push(MultipartFormDataField::file("mdfile"));

    options
        .allowed_fields
        .push(MultipartFormDataField::text("theme-select"));

    let multipart_form = MultipartFormData::parse(content_type, data, options)
        .await
        .map_err(|e| {
            (
                Status::BadRequest,
                format!("Multipart parsing error: {e:?}"),
            )
        })?; // (Status, String)

    let upload_path = "/tmp/uploaded.md";
    let output_light_path = "/tmp/output_light.html";
    let output_dark_path = "/tmp/output_dark.html";
    let output_zip_path = "/tmp/output.zip";

    // Template und CSS-Pfade anpassen
    let template_path = "templates/github-markdown-template.html";
    let light_css_path = "static/markdown-css/github-markdown-light.css";
    let dark_css_path = "static/markdown-css/github-markdown-dark.css";

    // Die hochgeladene Datei extrahieren
    if let Some(file_fields) = multipart_form.files.get("file-upload") {
        let file_field = &file_fields[0];
        let temp_path = &file_field.path;

        // Kopiere die hochgeladene Datei in den definierten Pfad
        fs::copy(&temp_path, upload_path)
            .map_err(|e| (Status::InternalServerError, format!("File copy error: {e}")))?;
    } else {
        return Err((Status::BadRequest, "No markdown file uploaded".to_string()));
    }

    // Theme auslesen
    let theme = if let Some(text_fields) = multipart_form.texts.get("theme-select") {
        let text_field = &text_fields[0];
        text_field.text.clone()
    } else {
        "light".to_string()
    };

    // Pandoc-Funktion
    let run_pandoc = |input: &str, output: &str, css: &str| -> Result<(), (Status, String)> {
        let status = Command::new("pandoc")
            .arg(input)
            .arg("-o")
            .arg(output)
            .arg("--standalone")
            .arg("--embed-resources")
            .arg("--template")
            .arg(template_path)
            .arg("--css")
            .arg(css)
            .status()
            .map_err(|e| {
                (
                    Status::InternalServerError,
                    format!("Failed to run pandoc: {e}"),
                )
            })?;

        if !status.success() {
            return Err((
                Status::InternalServerError,
                format!("Pandoc conversion failed for {output}"),
            ));
        }
        Ok(())
    };

    match theme.as_str() {
        "light" => {
            run_pandoc(upload_path, output_light_path, light_css_path)?;
            let html_bytes = fs::read(output_light_path).map_err(|_| {
                (
                    Status::InternalServerError,
                    "Error reading converted file".to_string(),
                )
            })?;
            let response = Response::build()
                .header(ContentType::HTML)
                .raw_header(
                    "Content-Disposition",
                    "attachment; filename=\"converted_light.html\"",
                )
                .sized_body(html_bytes.len(), Cursor::new(html_bytes))
                .finalize();
            Ok(response)
        }
        "dark" => {
            run_pandoc(upload_path, output_dark_path, dark_css_path)?;
            let html_bytes = fs::read(output_dark_path).map_err(|_| {
                (
                    Status::InternalServerError,
                    "Error reading converted file".to_string(),
                )
            })?;

            let response = Response::build()
                .header(ContentType::HTML)
                .raw_header(
                    "Content-Disposition",
                    "attachment; filename=\"converted_dark.html\"",
                )
                .sized_body(html_bytes.len(), Cursor::new(html_bytes))
                .finalize();
            Ok(response)
        }
        "both" => {
            run_pandoc(upload_path, output_light_path, light_css_path)?;
            run_pandoc(upload_path, output_dark_path, dark_css_path)?;

            // ZIP erstellen (zip im Container installiert haben)
            let status = Command::new("zip")
                .arg("-j") // keine Verzeichnisstruktur
                .arg(output_zip_path)
                .arg(output_light_path)
                .arg(output_dark_path)
                .status()
                .map_err(|e| {
                    (
                        Status::InternalServerError,
                        format!("Failed to run zip: {}", e),
                    )
                })?;

            if !status.success() {
                return Err((Status::InternalServerError, "Zipping failed".to_string()));
            }

            let zip_bytes = fs::read(output_zip_path).map_err(|_| {
                (
                    Status::InternalServerError,
                    "Error reading zip file".to_string(),
                )
            })?;
            let response = Response::build()
                .header(ContentType::new("application", "zip"))
                .raw_header(
                    "Content-Disposition",
                    "attachment; filename=\"converted.zip\"",
                )
                .sized_body(zip_bytes.len(), Cursor::new(zip_bytes))
                .finalize();
            Ok(response)
        }
        _ => Err((Status::BadRequest, "Invalid theme option".to_string())),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 54321,
            ..Default::default()
        })
        .mount(
            "/",
            routes![home, markdown_to_html, convert_markdown_to_html],
        )
        .mount("/static", FileServer::from("./static"))
        .attach(Template::fairing())
}
