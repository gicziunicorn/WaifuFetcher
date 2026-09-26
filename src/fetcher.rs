use bytes::Bytes;
use log::info;
use tokio::sync::{mpsc, oneshot};
use std::thread;
use rand::{Rng, RngExt, rngs::ThreadRng};
use iced::widget::image as iced_image;

use rfd::MessageButtons::OkCancel;
use wreq::Client;
use crate::{ResultToString, sites::{self, NekosResponse, Rule34Response, Sites}};

static API_URLS: &[(&str, &str)] = &[
    ("rule34", "https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1"),
];
fn get_api_url(key: &str) -> Option<&str> {
    API_URLS.iter().find(|&&(k, _)| k == key).map(|&(_, v)| v)
}

#[derive(Clone)]
pub enum FetchRequestType {
    Image,
    Posts(Sites),
}

// This is what the gui will send here
pub struct FetchRequest {
    pub url: &'static str,
    pub tags: &'static str,
    pub limit: &'static str,
    pub api_key: &'static str,
    pub user_id: &'static str,
}


struct FetchJob {
    fetch_type: FetchRequestType,
    url: String,
    response_tx: oneshot::Sender<Result<FetchResponse, String>>,
}

#[derive(Debug)]
struct PostResponse {
    image_url: String,
}

enum FetchResponse {
    Image(Bytes),
    Posts(Result<Vec<PostResponse>, String>),
}

/// An object that stores and manages the fetching logic.
#[derive(Clone)]
pub struct Fetcher {
    job_sender: mpsc::UnboundedSender<FetchJob>,
}

impl Fetcher {
    /// Start the background thread with tokio for fetching.
    pub fn new(client: Client) -> Self {
        let (job_sender, mut job_receiver) = mpsc::unbounded_channel::<FetchJob>();

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to start the background fetcher thread.");

            // Run an infinite loop waiting for incoming fetch requests
            // thanks to await, the loop won't use any CPU while waiting for requests :D
            rt.block_on(async {
                while let Some(job) = job_receiver.recv().await {
                    let client_ref = &client;
                    let result = async move {
                        // get the response
                        let response = client_ref.get(job.url.as_str()).send().await.err_to_string()?;
                        if !response.status().is_success() {
                            return Err(format!("HTTP Error: {}", response.status()));
                        }
                        // decide what to do with the response
                        match job.fetch_type {
                            FetchRequestType::Image => {
                                let bytes = response.bytes().await.err_to_string()?;
                                Ok(FetchResponse::Image(bytes))
                            }
                            FetchRequestType::Posts(site) => {
                                // get the needed parts of the response for each site
                                match site {
                                    Sites::Nekos => {
                                        let nekos_response = response.json::<NekosResponse>().await.err_to_string()?;
                                        let result = nekos_response.images.into_iter()
                                            .map( |nekos_post|
                                                PostResponse {
                                                    image_url: format!("https://nekos.moe/image/{}", nekos_post.id)
                                                })
                                            .collect::<Vec<_>>();
                                        Ok(FetchResponse::Posts(Ok(result)))
                                    }
                                    _ => {
                                        return Err("Unsupported site".to_string());
                                    }
                                }
                            }
                        }
                    }.await;

                    let _ = job.response_tx.send(result);
                }
            });
        });

        Self { job_sender }
    }

    pub async fn fetch_image_link(&self, url: String) -> Result<Bytes, String> {
        let (response_tx, response_rx) = oneshot::channel();

        let job = FetchJob {
            fetch_type: FetchRequestType::Image,
            url: url,
            response_tx,
        };
        if self.job_sender.send(job).is_err() {
            return Err(String::from("Network thread worker crashed or closed"));
        }

        let res = response_rx.await
            .map_err(|e| format!("Channel error: {}", e))?;

        match res {
            Ok(FetchResponse::Image(bytes)) => Ok(bytes),
            Ok(_) => Err(String::from("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    async fn rule34_fetch(&self, data: FetchRequest) -> Result<Vec<PostResponse>, String> {
        info!("fetching rule34");
        let (response_tx, response_rx) = oneshot::channel();

        let url = format!("{}&tags={}&limit={}&api_key={}&user_id={}", get_api_url("rule34").unwrap(), data.tags, data.limit, data.api_key, data.user_id);

        let job = FetchJob {
            fetch_type: FetchRequestType::Posts(Sites::Rule34),
            url: url,
            response_tx,
        };
        if self.job_sender.send(job).is_err() {
            return Err(String::from("Network thread worker crashed or closed"));
        }

        let res = response_rx.await
            .map_err(|e| format!("Channel error: {}", e))?;

        match res {
            Ok(FetchResponse::Posts(result)) => result,
            Ok(_) => Err(String::from("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    pub async fn rule34_get_random(&self, fetch_data: FetchRequest) -> Result<(Vec<u8>, usize, usize), String> {
        let posts = self.rule34_fetch(fetch_data).await;
        info!("Got response");
        match posts {
            Ok(data) => {
                let sample_url = {
                    // Generate the random inside a block so it's dropped
                    // and won't cause problems with future
                    let mut rng = rand::rng();
                    let r = rng.random_range(0..data.iter().len());
                    data[r].image_url.clone()
                };
                info!("Sample URL: {}", sample_url);
                let bytes = self.fetch_image_link(sample_url).await?;
                let image_vec = bytes.to_vec();
                let size = imagesize::blob_size(&image_vec).err_to_string()?;
                Ok((image_vec, size.width, size.height))
            },
            Err(err) => Err(err)
        }
    }

    pub async fn nekos_get_random(&self, fetch_data: FetchRequest) -> Result<(Vec<u8>, usize, usize), String> {
        // get a few random posts
        let url_random = "https://nekos.moe/api/v1/random/image".to_string();

        let (response_tx, response_rx) = oneshot::channel();
        let job = FetchJob {
            fetch_type: FetchRequestType::Posts(Sites::Nekos),
            url: url_random,
            response_tx,
        };

        if self.job_sender.send(job).is_err() {
            return Err(String::from("Network thread worker crashed or closed"));
        }
        let res = response_rx.await
            .map_err(|e| format!("Channel error: {}", e))?;
        let posts = match res {
            Ok(FetchResponse::Posts(result)) => result,
            Ok(_) => Err(String::from("Unexpected response")),
            Err(e) => Err(e),
        }?;

        info!("Fetched posts from Nekos: \n{:#?}", posts);

        // get the image
        let url = posts[0].image_url.clone();
        let bytes = self.fetch_image_link(url).await?;
        let image_vec = bytes.to_vec();
        let size = imagesize::blob_size(&image_vec).err_to_string()?;
        Ok((image_vec, size.width, size.height))
    }

    pub async fn fetch_site(&self, data: FetchRequest, site: Sites) -> Result<(Vec<u8>, usize, usize), String> {
        match site {
            Sites::Rule34 => self.rule34_get_random(data).await,
            Sites::Nekos => self.nekos_get_random(data).await,
        }
    }
}
