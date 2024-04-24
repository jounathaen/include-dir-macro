use std::collections::HashMap;
use std::path::{Path, PathBuf};

use include_dir_macro::include_dir; // proc-macro
use rocket::http::{Header, Status};
use rocket::{
    response::{self, Responder},
    Request, State,
};

struct StaticFiles {
    files: HashMap<&'static Path, &'static [u8]>,
}

fn expected_type(mimetype: &str, input: &[u8]) -> Option<String> {
    if tree_magic::match_u8(mimetype, input) {
        Some(mimetype.to_owned())
    } else {
        None
    }
}

#[derive(Debug)]
struct InvalidFile(PathBuf);

#[derive(Debug)]
struct StaticFile {
    mimetype: String,
    raw: &'static [u8],
}
impl<'r> Responder<'r, 'static> for StaticFile {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Ok(rocket::Response::build()
            .status(Status::Ok)
            .header(Header::new("Content-Type", self.mimetype))
            .sized_body(self.raw.len(), ::std::io::Cursor::new(self.raw.to_owned()))
            .finalize())
    }
}

impl StaticFiles {
    pub fn new(files: HashMap<&'static Path, &'static [u8]>) -> StaticFiles {
        StaticFiles { files }
    }

    pub fn get_raw(&self, path: &Path) -> Option<&'static [u8]> {
        self.files.get(&path).map(|x| *x)
    }

    pub fn get_response(&self, path: &Path) -> Option<StaticFile> {
        match self.get_raw(path) {
            None => None,
            Some(raw) => {
                let extension = path.extension().and_then(|ext| ext.to_str());
                let mimetype = match extension {
                    Some("png") => expected_type("image/png", raw),
                    Some("jpg") | Some("jpeg") => expected_type("image/jpeg", raw),
                    Some("gif") => expected_type("image/gif", raw),
                    Some("js") => expected_type("application/javascript", raw),
                    Some("json") => expected_type("text/json", raw),
                    Some("html") => expected_type("text/html", raw),
                    _ => Some(tree_magic::from_u8(raw)),
                }?;
                Some(StaticFile { raw, mimetype })
            }
        }
    }
}

use rocket::{get, routes};

#[get("/")]
fn hello() -> &'static str {
    "Hello world"
}

#[get("/static/<path..>")]
fn staticfiles(path: PathBuf, store: &State<StaticFiles>) -> Option<StaticFile> {
    store.inner().get_response(&path)
}

#[get("/raw/<path..>")]
fn rawfiles(path: PathBuf, store: &State<StaticFiles>) -> Option<&'static str> {
    store
        .get_raw(&path)
        .map(|data| ::std::str::from_utf8(data).unwrap())
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .manage(StaticFiles::new(include_dir!("examples/static/web")))
        .mount("/", routes![hello, staticfiles, rawfiles])
}
