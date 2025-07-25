use reqwest::blocking::Client;
use serde::{Deserialize, Serialize}; // Import both Serialize and Deserialize
use std::env;
use std::sync::mpsc::{channel, Receiver, Sender};
// Import serde_json at the top of your file
use serde_json; // <-- Add this import
pub struct Llm {
    model: String,
    api: String,
    url: String,
}

// --- Define data structures for the request and response ---

// Message struct used for BOTH request and response
// It needs both Serialize (for sending) and Deserialize (for receiving)
#[derive(Serialize, Deserialize, Debug, Clone)] // <-- Add Serialize, Deserialize, Debug, Clone
struct Message {
    role: String,
    content: String,
}

// Define the request structure according to the OpenAI API spec
#[derive(Serialize)] // <-- Only Serialize needed for the request body
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>, // Use the Message struct here
                            // Add enable_thinking if needed for Qwen models
                            // enable_thinking: Option<bool>,
}

// Define the simplified response structure to deserialize the relevant parts
#[derive(Deserialize, Debug, Serialize)] // <-- Add Deserialize and Debug for the response structs
struct Choice {
    index: u32,
    // Deserialize the message field correctly
    message: Message, // <-- This Message now implements Deserialize
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)] // <-- Add Deserialize and Debug
pub struct ChatCompletionResponse {
    id: Option<String>,
    choices: Vec<Choice>,
    // You can add usage: Option<Usage> here if you need token usage info
    // usage: Option<Usage>,
}

// --- Main function using blocking reqwest ---
//#[tokio::main]
impl Llm {
    pub fn new(model: &str, api: &str, url: &str) -> Self {
        Self {
            model: model.to_string(),
            api: api.to_string(),
            url: url.to_string(),
        }
    }
    pub fn llm_get(prompt: &str, mes: Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Get API Key
        let api_key =
            env::var("DASHSCOPE_API_KEY").unwrap_or_else(|_| "YOUR_DASHSCOPE_API_KEY".to_string());

        if api_key == "YOUR_DASHSCOPE_API_KEY" {
            eprintln!(
                "Warning: Using placeholder API key. Set DASHSCOPE_API_KEY environment variable."
            );
        }

        // 2. Create the HTTP client
        let client = Client::new();

        // 3. Define the endpoint URL
        let url = "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions";

        // 4. Create the request payload
        let request_body = ChatCompletionRequest {
            model: "qwen-plus".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            // enable_thinking: Some(false), // Uncomment if needed
        };

        // 5. Send the POST request
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()?;

        // 6. Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            eprintln!("HTTP Error: {}", status);
            let error_text = response.text()?;
            eprintln!("Error Body: {}", error_text);
            return Err(format!("Request failed with status: {}", status).into());
        }

        // 7. Deserialize the JSON response
        let completion_response: ChatCompletionResponse = response.json()?;

        // 8. Print the result using serde_json for pretty printing
        // Make sure serde_json is imported
        let response_json = serde_json::to_string_pretty(&completion_response)?; // <-- This line should now work
                                                                                 //println!("{}", response_json.clone());

        // Alternatively, print just the assistant's reply content:
        if let Some(first_choice) = completion_response.choices.first() {
            //println!("--- Assistant's Reply ---");
            //println!("{}", first_choice.message.content);
            mes.send(first_choice.message.content.clone()).unwrap();
        } else {
            println!("No choices returned in the response.");
        }

        Ok(())
    }
}
