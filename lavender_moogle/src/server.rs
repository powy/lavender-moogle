use crate::job_management;
use crate::spell;
use crate::types;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log::{debug, error};
use serde_json::Value;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

pub fn start_json_server(shared_character_details: Arc<Mutex<types::CharacterDetails>>) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async move {
        let addr = ([127, 0, 0, 1], 50017).into();

        let make_svc = make_service_fn(move |_conn| {
            let shared_character_details = Arc::clone(&shared_character_details);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let shared_character_details = Arc::clone(&shared_character_details);
                    handle_request(req, shared_character_details)
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
    shared_character_details: Arc<Mutex<types::CharacterDetails>>,
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

                // Reserve (lock) the character_details and perform an action
                {
                    let mut character_details = shared_character_details.lock().unwrap();

                    // Handle "ANY" job - use current job for spell application
                    let effective_job = if job == "ANY" {
                        let current_job = job_management::get_current_job();
                        debug!(
                            "Received ANY job message - spell_name: {}, using current job: {}",
                            spell_name, current_job
                        );
                        current_job
                    } else {
                        job_management::update_current_job(&mut character_details, &job);
                        job
                    };

                    if spell_name == "reset" || spell_name == "wipe reset" {
                        // Receiving reset spell, means reset all status and CD
                        types::reset_character_status(&mut character_details);
                    } else if spell_name == "gauge" || spell_name == "job gauge" {
                        handle_gauge(&data, &mut character_details, &effective_job);
                    } else if spell_name == "Deactivate Burst" {
                        // Activate "Delayed Burst" status on all jobs (backward compatibility)
                        spell::activate_status_for_any_job(&mut character_details, "Delayed Burst");
                        // Also set the burst_allowed flag to false on all jobs
                        types::set_burst_allowed(&mut character_details, false);
                    } else if spell_name == "Activate Burst" {
                        // Deactivate "Delayed Burst" status on all jobs (backward compatibility)
                        spell::deactivate_status_for_any_job(
                            &mut character_details,
                            "Delayed Burst",
                        );
                        // Also set the burst_allowed flag to true on all jobs
                        types::set_burst_allowed(&mut character_details, true);
                    } else if spell_name == "dump resources start" {
                        // Set dump_resources flag to true on all jobs
                        types::set_dump_resources(&mut character_details, true);
                    } else if spell_name == "dump resources stop" {
                        // Set dump_resources flag to false on all jobs
                        types::set_dump_resources(&mut character_details, false);
                    } else if spell_name == "buff" {
                        debug!("Received buffs !");
                        if let Some(buff_name) = data.get("buff_name").and_then(|v| v.as_str()) {
                            spell::apply_new_buff(
                                &mut character_details,
                                &effective_job,
                                buff_name,
                            );
                        }
                    } else if spell_name == "death" {
                        // Handle character death - clear buffs but preserve cooldowns
                        spell::handle_character_death(&mut character_details);
                    } else {
                        // Update the status following received spell, and apply rotation
                        spell::apply_spell_cast(
                            &mut character_details,
                            &effective_job,
                            &spell_name,
                        );
                        job_management::apply_current_job_rotation(&mut character_details);
                    }
                } // The lock is released here when the scope ends

                return Ok(Response::new(Body::from(
                    r#"{"message": "JSON received and processed"}"#,
                )));
            }
        }

        // If JSON is invalid or required fields are missing, do nothing and return
        Ok(Response::new(Body::empty()))
    } else {
        Ok(Response::new(Body::from("Only POST requests are accepted")))
    }
}

fn handle_gauge(data: &Value, character_details: &mut types::CharacterDetails, job: &str) {
    if let (Some(data0), Some(data1), Some(data2), Some(data3)) = (
        data.get("data0"),
        data.get("data1"),
        data.get("data2"),
        data.get("data3"),
    ) {
        // Assign values to variables
        let data0 = data0.as_str().unwrap_or_default().to_string();
        let data1 = data1.as_str().unwrap_or_default().to_string();
        let data2 = data2.as_str().unwrap_or_default().to_string();
        let data3 = data3.as_str().unwrap_or_default().to_string();

        // Update the job gauge with the new data
        job_management::update_job_gauge(character_details, job, &data0, &data1, &data2, &data3);

        // Apply the job rotation after the gauge update
        job_management::apply_current_job_rotation(character_details);
    }
}
