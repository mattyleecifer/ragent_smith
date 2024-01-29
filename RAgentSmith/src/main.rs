use chrono;
use std::env;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fs;

const ROLE_USER: &str = "user";
const ROLE_ASSISTANT: &str = "assistant";
const ROLE_SYSTEM: &str = "system";
const DEFAULT_MODEL: &str = "mistral-medium";



#[derive(Debug, Clone)]
pub struct Agent {
    prompt: String,
    token_count: i32,
    api_key: String,
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestBody {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    index: i32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    prompt_tokens: i32,
    total_tokens: i32,
    completion_tokens: i32,
}

impl Agent {
    fn new(key: String, prompt: String) -> Self {
        if prompt.is_empty() {
            // Set default prompt
            let prompt = String::from("You are a helpful assistant. Please generate truthful, accurate, and honest responses while also keeping your answers succinct and to-the-point. Today's date is: %B %d, %Y");
        };

        let mut agent = Self {
            prompt: prompt,
            model: String::from(DEFAULT_MODEL),
            token_count: 0,
            api_key: key,
            messages: Vec::new(),
        };

        if agent.api_key.is_empty() {
            eprintln!("Environment variable API_KEY not set, exiting...");
            panic!();
        }

        agent
    }

    fn set_prompt(self: &mut Self, prompt_text: String) {
        self.prompt = String::from(prompt_text); 
    }

    fn get_model_URL(&self) -> String {
        let url = match &self.model[..] {
            model if model.starts_with("mistral") => "https://api.mistral.ai/v1/chat/completions".to_string(),
            model if model.starts_with("gpt") => "https://api.openai.com/v1/chat/completions".to_string(),
            _ => {
                println!("Error: Invalid model");
                "".to_string()
            }
        };
        url
    }

    fn add_message(self: &mut Self, role: String, content: String) {
        let message = Message {
            role,
            content,
        };
        self.messages.push(message)
    }

    fn update_token_count(self: &mut Self, total_tokens: i32) {
        self.token_count += total_tokens
    }

    fn get_response(self: &mut Self) -> Result<Message, Box<dyn Error>> {
        let request_body = RequestBody {
            model: self.model.clone(),
            messages: self.messages.clone(),
        };

        let json_data = serde_json::to_string(&request_body)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.api_key))?);

        let client = Client::new();
        let response = client.post(&self.get_model_URL())
                             .headers(headers)
                             .body(json_data)
                             .send()?;

        let json_data = response.text()?;
        let chat_response: ChatResponse = serde_json::from_str(&json_data)?;

        if chat_response.choices.len() == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Error with response",
            )));
        }

        self.update_token_count(chat_response.usage.total_tokens);

        self.add_message(String::from("assistant"), chat_response.choices[0].message.content.clone());

        println!("{}", chat_response.choices[0].message.content);

        Ok(chat_response.choices[0].message.clone())
    }

}



fn main(){
    let mut agent = Agent::new(String::from("4W2bNZ1ga3xwgxYVaRPxBIre417uzLxt"), String::from("whewrwef"));
    println!("Hello, {ROLE_USER}! Today is {}. Enjoy your day!", get_date());
    println!("Your agent key is {} and the prompt is {}", agent.api_key, agent.prompt);

    agent.set_prompt(String::from("Wow this shit is hard"));

    println!("The prompt is {}", agent.prompt);

    agent.add_message(String::from("user"), String::from("Are you there"));
    agent.get_response();

}

fn get_date() -> String {
    let today = chrono::offset::Local::now();
    let today = today.format("%b %d, %Y").to_string().to_string();
    today
}