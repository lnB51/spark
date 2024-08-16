use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use embedded_svc::http::{client::Client, Method};
use esp_idf_svc::io::EspIOError;
use frankenstein::ErrorResponse;
use frankenstein::TelegramApi;
use std::path::PathBuf;
use thiserror::Error;

/// Sends an HTTP POST request with the given URL and data.
///
/// # Arguments
///
/// * `url` - The URL to send the POST request to.
/// * `data` - The data to include in the POST request.
///
/// # Returns
///
/// Returns a `Result` containing the response as a vector of bytes, or an `EspBotError` on failure.
pub fn http_post(url: impl AsRef<str>, data: &[u8]) -> Result<Vec<u8>, EspBotError> {

    // Create a new EspHttpConnection with default Configuration.
    let configuration = Configuration {
        timeout: Some(core::time::Duration::from_secs(130)),
        ..Default::default()
    };

    // Initialize the HTTP connection with the specified configuration
    let connection = EspHttpConnection::new(&configuration)?;

    // Get a client using the embedded_svc Client::wrap method.
    let mut client = Client::wrap(connection);

    // Set the necessary headers for the POST request
    let headers = [("Content-Type", "application/json")];

    // Create a new POST request with the specified URL and headers
    let mut request = client.request(Method::Post, url.as_ref(), &headers)?;

    // Write the data to the request body
    request.write(data)?;

    // Submit the request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let mut response = request.submit()?;
    let status = response.status();
    match status {
        200..=299 => {
            // Read the response data in chunks and store it in the output vector
            let mut buf = [0_u8; 4];
            let mut output = Vec::new();

            loop {
                match response.read(&mut buf)? {
                    0 => break, // No more data to read, break the loop
                    b => {
                        output.extend_from_slice(&buf[..b]); // Append the read data to the output vector
                    }
                }
            }

            Ok(output) // Return the successful response data
        }
        _ => {
            // Handle error responses by reading the response body
            let mut buf = [0_u8; 256];
            response.read(buf.as_mut())?;
            let resp_string =
                core::str::from_utf8(&buf).unwrap_or("invalid utf8 when parsing error");

            log::error!("{}\n", resp_string);

            // Return an HTTP error with the status code and response message
            Err(EspBotError::Http(HttpError {
                _code: status,
                _message: format!("response code: {}", status),
            }))
        }
    }
}

pub struct Esp32Api {
    pub api_url: String, // Base URL for the Telegram API
}

#[derive(Error, Debug)]
pub enum EspBotError {
    #[error("HTTP error")]
    Http(HttpError), // HTTP-related errors
    #[error("API error")]
    Api(ErrorResponse), // API-related errors (e.g., from Telegram)
    #[error("ESP error")]
    Esp(#[from] esp_idf_svc::hal::sys::EspError), // ESP-related errors
    #[error("IO error")]
    Io(#[from] EspIOError), // I/O-related errors
    #[error("utf8 error")]
    Json(#[from] core::str::Utf8Error), // UTF-8 decoding errors
    #[error("serde error")]
    Serde(#[from] serde_json::Error), // Serialization/deserialization errors
}

#[derive(Debug)]
pub struct HttpError {
    pub _code: u16, // HTTP status code
    pub _message: String, // Error message associated with the status code
}

// Base URL for the Telegram Bot API
static BASE_API_URL: &str = "https://api.telegram.org/bot";

impl Esp32Api {
    /// Creates a new instance of `Esp32Api` with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for accessing the Telegram Bot API.
    ///
    /// # Returns
    ///
    /// Returns a new `Esp32Api` instance with the base URL configured.
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        let api_url = format!("{BASE_API_URL}{api_key}");
        Self { api_url }
    }
}

impl From<std::io::Error> for EspBotError {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{error:?}");
        let error = HttpError { _code: 500, _message: message };
        Self::Http(error)
    }
}

impl TelegramApi for Esp32Api {
    type Error = EspBotError;

    /// Sends a request to the Telegram API with the given method and parameters.
    ///
    /// # Arguments
    ///
    /// * `method` - The API method to call (e.g., "sendMessage").
    /// * `params` - The parameters for the API call, serialized as JSON.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed response, or an `EspBotError` on failure.
    fn request<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, EspBotError> {
        // Construct the full URL for the API request
        let url = format!("{}/{method}", self.api_url);

        // Send the request with or without parameters
        let response = match params {
            None => http_post(url, &[])?,
            Some(data) => {
                let json = serde_json::to_string(&data)?;
                http_post(url, json.as_bytes())?
            }
        };

        // Parse the response as a UTF-8 string
        let text = core::str::from_utf8(&response)?;

        // Attempt to parse the response as the expected type
        let parsed_result: Result<T2, serde_json::Error> = serde_json::from_str(text);

        // Handle errors during response parsing
        parsed_result.map_err(|_| {
            let parsed_error: Result<ErrorResponse, serde_json::Error> = serde_json::from_str(text);

            match parsed_error {
                Ok(result) => EspBotError::Api(result),
                Err(error) => {
                    let message = format!("{error:?}");
                    let error = HttpError { _code: 500, _message: message };
                    EspBotError::Http(error)
                }
            }
        })
    }
    /// This method is not supported by the current implementation.
    ///
    /// Attempts to send a request with form data will result in an error.
    ///
    /// # Arguments
    ///
    /// * `_method` - The API method to call.
    /// * `_params` - The parameters for the API call.
    /// * `_files` - A list of files to upload with the request.
    ///
    /// # Returns
    ///
    /// Always returns an `EspBotError::Http` error indicating that form data requests are not supported because
    /// isahc doesn't support multipart uploads
    /// https://github.com/sagebind/isahc/issues/14
    fn request_with_form_data<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        _method: &str,
        _params: T1,
        _files: Vec<(&str, PathBuf)>,
    ) -> Result<T2, EspBotError> {
        let error = HttpError {
            _code: 500,
            _message: "isahc doesn't support form data requests".to_string(),
        };

        Err(EspBotError::Http(error))
    }
}