use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::sync::{Arc, Mutex};
use std::convert::Infallible;
use log::{debug, error};
use serde_json::Value;
use crate::types;
use crate::spell_actions;

pub fn start_json_server(shared_spell_book: Arc<Mutex<types::SpellBook>>) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async move {
        let addr = ([127, 0, 0, 1], 50017).into();

        let make_svc = make_service_fn(move |_conn| {
            let shared_spell_book = Arc::clone(&shared_spell_book);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let shared_spell_book = Arc::clone(&shared_spell_book);
                    handle_request(req, shared_spell_book)
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        debug!("Server listening on http://{}", addr);

        if let Err(e) = server.await {
            error!("Server error: {}", e);
        }
    });
}
async fn handle_request(
    req: Request<Body>,
    shared_spell_book: Arc<Mutex<types::SpellBook>>,
) -> Result<Response<Body>, Infallible> {
    if req.method() == hyper::Method::POST {
        let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
        
        // Attempt to parse the JSON
        if let Ok(data) = serde_json::from_slice::<Value>(&whole_body) {
            // Check if the required fields "spell_name" and "job" are present
            if let (Some(spell_name), Some(job)) = (data.get("spell_name"), data.get("job")) {
                // Assign values to variables
                let spell_name = spell_name.as_str().unwrap_or_default().to_string();
                let job = job.as_str().unwrap_or_default().to_string();

                // debug!("Received spell_name: {}, job: {}", spell_name, job);

                // Reserve (lock) the spell_book and perform an action
                {
                    let mut spell_book = shared_spell_book.lock().unwrap();
                    spell_actions::apply_spell_cast(&mut spell_book, &job, &spell_name);
                    // spell_actions::apply_spell_cast(&mut spell_book, &job, &spell_name);
                } // The lock is released here when the scope ends

                return Ok(Response::new(Body::from(r#"{"message": "JSON received and processed"}"#)));
            }
        }

        // If JSON is invalid or required fields are missing, do nothing and return
        Ok(Response::new(Body::empty()))
    } else {
        Ok(Response::new(Body::from("Only POST requests are accepted")))
    }
}