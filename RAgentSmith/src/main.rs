use chrono;
use std::env;

const ROLE_USER: &str = "user";
const ROLE_ASSISTANT: &str = "assistant";
const ROLE_SYSTEM: &str = "system";
const DEFAULT_MODEL: &str = "mistral-medium";



#[derive(Debug, Clone)]
pub struct PromptDefinition {
    name: String,
    description: String,
    parameters: String,
}

#[derive(Debug, Clone)]
pub struct Agent {
    prompt: PromptDefinition,
    token_count: i32,
    api_key: String,
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone)]
pub struct RequestBody {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Vec<Usage>
}

#[derive(Debug, Clone)]
pub struct Choice {
    index: i32,
    message: Vec<Message>,
    finishreason: String,
}

#[derive(Debug, Clone)]
pub struct Usage {
    prompttokens: i32,
    totaltokens: i32,
    completiontokens: i32,
}

impl Agent {
    fn new(key: String, prompt_definition: PromptDefinition) -> Self {
        let mut agent = Self {
            prompt: prompt_definition,
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
}

impl PromptDefinition {
    fn default_prompt() -> Self {
        PromptDefinition{
            name: String::from("Default"),
            description: String::from("Default Prompt"),
            parameters: String::from("You are a helpful assistant. Please generate truthful, accurate, and honest responses while also keeping your answers succinct and to-the-point. Today's date is: %B %d, %Y"),
        }
    }   
}

fn main(){
    let agent = Agent::new(String::from("test"), PromptDefinition::default_prompt());
    println!("Hello, {ROLE_USER}! Today is {}. Enjoy your day!", get_date());
    println!("Your agent key is {} and the prompt is {}", agent.api_key, agent.prompt.parameters)
}

fn get_date() -> String {
    let today = chrono::offset::Local::now();
    let today = today.format("%b %d, %Y").to_string().to_string();
    today
}