use chrono;
use std::env;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fs;
use std::io;
use std::process;

const ROLE_USER: &str = "user";
const ROLE_ASSISTANT: &str = "assistant";
const ROLE_SYSTEM: &str = "system";
const DEFAULT_MODEL: &str = "mistral-medium";
const MISTRAL_API: &str = "https://api.mistral.ai/v1/chat/completions";
const OPENAI_API: &str = "https://api.openai.com/v1/chat/completions";


#[derive(Debug, Clone)]
pub struct Agent {
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
    fn new(key: &String, prompt_text: &String) -> Self {
        let mut prompt = Message {
            role: ROLE_SYSTEM.to_string(),
            content: String::new(),
        };

        if prompt_text.is_empty() {
            // Set default prompt
            prompt.content = String::from("You are a helpful assistant. Please generate truthful, accurate, and honest responses while also keeping your answers succinct and to-the-point. Today's date is: %B %d, %Y");

        } else {
            prompt.content = prompt_text.to_string();
        };

        let mut agent = Self {
            model: String::from(DEFAULT_MODEL),
            token_count: 0,
            api_key: key.to_string(),
            messages: Vec::new(),
        };

        if agent.api_key.is_empty() {
            eprintln!("Environment variable API_KEY not set, exiting...");
            panic!();
        }

        agent
    }

    fn set_prompt(self: &mut Self, prompt_text: &String) {
        let mut prompt = Message {
            role: ROLE_SYSTEM.to_string(),
            content: prompt_text.to_string(),
        };

        self.messages = Vec::new();

        self.messages.push(prompt)
    }

    fn get_model_URL(&self) -> String {
        let url = match &self.model[..] {
            model if model.starts_with("mistral") => MISTRAL_API.to_string(),
            model if model.starts_with("gpt") => OPENAI_API.to_string(),
            _ => {
                println!("Error: Invalid model");
                "".to_string()
            }
        };
        url
    }

    fn add_message(self: &mut Self, role: &str, content: &String) {
        let message = Message {
            role: role.to_string(),
            content: content.to_string(),
        };
        self.messages.push(message)
    }

    fn update_token_count(self: &mut Self, total_tokens: &i32) {
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

        self.update_token_count(&chat_response.usage.total_tokens);

        self.add_message(ROLE_ASSISTANT, &chat_response.choices[0].message.content);

        // println!("{}", chat_response.choices[0].message.content);

        Ok(chat_response.choices[0].message.clone())
    }

    fn get_flags() -> (Self, bool) {
        let args: Vec<String> = env::args().collect();
        let mut model = String::new();
        let mut api_key = String::new();
        let mut flag: &String = &String::new(); 
        let mut arg: &String = &String::new();
        let mut messages: Vec<Message> = vec![];
        let mut consoleflag: bool = false;

        for i in 0..args.len() {
            flag = &args[i];
            
            if flag == "-console" {
                consoleflag = true;
            }

            arg = if i + 1 < args.len() {
                &args[i + 1]
            } else {
                continue;
            };

            match flag.as_str() {
                "-key" => {
                    if arg.to_string().is_empty() {
                        eprintln!("Environment variable API_KEY not set, exiting...");
                        panic!();
                    } else {
                        api_key = arg.to_string();
                    }
                },
                "-prompt" => {
                    let mut newmessage = Message {
                        role: ROLE_SYSTEM.to_string(),
                        content: String::new(),
                    };
                    if arg.to_string().is_empty(){
                        newmessage.content = String::from("You are a helpful assistant. Please generate truthful, accurate, and honest responses while also keeping your answers succinct and to-the-point. Today's date is: %B %d, %Y")
                    } else {
                        newmessage.content = arg.to_string();
                    };   
                    messages.push(newmessage);
                    
                },
                "-model" => {
                    model = arg.to_string();
                    println!("{}", model);
                },
                "-message" => {
                    let newmessage = Message {
                        role: ROLE_USER.to_string(),
                        content: arg.to_string(),
                    };
                    messages.push(newmessage)
                },
                "-messageassistant" => {
                    let newmessage = Message {
                        role: ROLE_ASSISTANT.to_string(),
                        content: arg.to_string(),
                    };
                    messages.push(newmessage)
                },
                _ => {}
            };

        }

        let mut agent = Self {
            model: model,
            token_count: 0,
            api_key: api_key,
            messages: messages,
        };



        (agent, consoleflag)
    }

    fn console(self: &mut Self) {
        loop {
            println!("User:");

            let mut input = String::new();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if input.trim() == "q" {
                process::exit(0);
            };

            self.add_message(ROLE_USER, &input);

            println!("Agent:");
            let result = self.get_response();
            match result {
                Ok(message) => {
                    println!("{}", message.content);
                    println!("Total tokens: {}\n", self.token_count)
                },
                Err(error) => {
                    println!("Error in getting response: {}", error);
                },
            }
        }
    }
}



fn main(){
    let (mut agent, consoleflag) = Agent::get_flags();

    if consoleflag {
        agent.console();
        println!("failed to call");
    } else {
        let result = agent.get_response();
        match result {
            Ok(message) => {
                println!("{}", message.content);
                println!("Total tokens: {}\n", agent.token_count)
            },
            Err(error) => {
                println!("Error in getting response: {}", error);
            },
        }
    }

    // println!("Your agent key is {} and the prompt is {}", agent.api_key, agent.messages[0].content);

    // println!("The prompt is d{}", agent.prompt);


}

fn get_date() -> String {
    let today = chrono::offset::Local::now();
    let today = today.format("%b %d, %Y").to_string().to_string();
    today
}