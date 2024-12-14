mod code {
    pub mod cli_error;
    pub mod my_helper;
    pub mod shared_state;
}
use chrono::Local;
use code::my_helper::MyHelper;
use code::shared_state::SharedState;
use context::Context;
use directories::ProjectDirs;
use image::ImageFormat;
use llm_rs::model_mode::ModelMode;
use openai_interface::ApiInterface;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use reqwest::blocking::get;
use rustyline::completion::FilenameCompleter;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, Event, EventHandler, KeyEvent};
use std::collections::HashMap;
use std::env::current_dir;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::{env, fs};
extern crate llm_rs;

use clap::Parser;
use llm_rs::context;
use llm_rs::openai_interface;

const DEFAULT_MODEL: &str = "gpt-4";
const DEFAULT_TOKENS: u32 = 2_000_u32;
const DEFAULT_TEMPERATURE: f32 = 0.9_f32;
const DEFAULT_MODE: &str = "chat";
const DEFAULT_RECORD_FILE: &str = "reply.txt";
const DEFAULT_HISTORY_FILE: &str = "history.txt";

/// Command line argument definitions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// The model to use
    #[arg(long, short = 'm',default_value=DEFAULT_MODEL)]
    model: String,

    /// Maximum tokens to return
    #[arg(long, short = 't', default_value_t=DEFAULT_TOKENS)]
    max_tokens: u32,

    /// Temperature for the model.
    #[arg(long, short = 'T', default_value_t = DEFAULT_TEMPERATURE)]
    temperature: f32,

    /// The secret key.  [Default: environment variable `OPENAI_API_KEY`]
    #[arg(long)]
    api_key: Option<String>,

    /// The initial mode (API endpoint)
    #[arg(long, short='d', default_value=DEFAULT_MODE)]
    mode: String,

    /// The file name that prompts and replies are recorded in
    #[arg(long, short='r', default_value=DEFAULT_RECORD_FILE)]
    record_file: String,

    /// The system prompt sent to the chat model
    #[arg(long, short='p', default_value=None)]
    system_prompt: Option<String>,
}

/// A structure to hold data for the interface.
struct CliInterface {
    /// If this is > 0 output status messages.  Information about
    /// queries, responses, etcetera.
    verbose: usize,

    history_file: String,

    record_file: String,

    audio_file: Option<String>,

    model_mode: ModelMode,

    model: String,

    /// The image model URL for the image that we are paying attention
    /// to.  Openai generated images
    pub focus_image_url: Option<String>,

    /// Image to use with image_edit mode.  User supplied or copied
    /// from `focus_image_url`
    pub image: Option<PathBuf>,

    /// Mask to use with image_edit mode.
    pub mask: Option<PathBuf>,

    /// Header cache.  This is used to monitor the headers.  I want to
    /// see what headers are coming back frmo OpenAI but they clutter
    /// things.  Cache them here and only report on headers that
    /// change
    header_cache: HashMap<String, String>,

    /// Cost in cents, often fraction of a cent.  This is not precise,
    /// only calculated for chat
    cost: f64,

    /// Local data.  Generally this is reading local files of data
    local_data: HashMap<String, String>,
}

impl CliInterface {
    /// Generate a file to store data locally
    fn make_file(suffix: &str) -> Result<PathBuf, Box<dyn Error>> {
        // Get the config directory for the current user in a
        // platform-specific way
        let project_dir = ProjectDirs::from("worik", "org", "llm-rs").unwrap();
        println!("project_dir ({:?})", project_dir);

        // Create the config directory, if it doesn't exist
        std::fs::create_dir_all(project_dir.config_dir())?;

        // Generate a random file name
        let rand_file_name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let file_path: PathBuf = project_dir
            .config_dir()
            .join(rand_file_name)
            .with_extension(suffix);

        // Create the file
        Ok(file_path)
    }

    /// Called for an image that OpenAI generates.
    fn process_image_url(&mut self, url: &str) -> Result<(), Box<dyn Error>> {
        println!("process_image_url({url})");
        let start = Instant::now();

        // Must convert the image
        // convert otter.png -type TrueColor -define png:color-type=6 otter_rgba.png

        let mut img_data: Vec<u8> = Vec::new();
        get(url).unwrap().read_to_end(&mut img_data).unwrap();
        println!("Down loaded URL: {} bytes", img_data.len());

        let incomming_image_file_path = Self::make_file("png")?;
        println!("incomming_image_file_path {:?}", incomming_image_file_path);
        let mut incomming_image_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&incomming_image_file_path)?;
        println!("Created {:?}", incomming_image_file_path);
        incomming_image_file.write_all(&img_data)?;
        let img = image::open(&incomming_image_file_path)?;
        println!("Opened {:?}", incomming_image_file);
        incomming_image_file.write_all(&img_data)?;
        println!(
            "Wrote image: {:?} {:#?}",
            start.elapsed(),
            incomming_image_file_path,
        );

        // Ensure the image has an alpha channel
        let img_rgba = img.into_rgba8();

        self.image = Some(incomming_image_file_path.as_path().to_owned());
        img_rgba.save_with_format(self.image.clone().unwrap(), ImageFormat::Png)?;
        webbrowser::open(self.image.clone().unwrap().as_os_str().to_str().unwrap())?;

        Ok(())
    }

    fn set_up_read_line(&self) -> rustyline::Result<Editor<MyHelper, FileHistory>> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let h = MyHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "".to_owned(),
            validator: MatchingBracketValidator::new(),
        };
        let mut read_line = Editor::with_config(config)?;
        read_line.set_helper(Some(h));
        read_line.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
        read_line.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
        if read_line.load_history(self.history_file.as_str()).is_err() {
            println!("No previous history.");
        }

        // Set control key C-q to quit.  Not really needed.  C-c does this
        // auto-magically
        read_line.bind_sequence(
            Event::KeySeq(vec![KeyEvent::ctrl('q')]),
            EventHandler::Simple(Cmd::Interrupt),
        );
        Ok(read_line)
    }

    fn expand_variables(&self, input: String) -> Result<String, Box<dyn Error>> {
        let re = Regex::new(r"\{(\w+)\}").unwrap();
        let result = re
            .replace_all(&input, |caps: &regex::Captures| {
                self.local_data
                    .get(&caps[1])
                    .unwrap_or(&caps[0].to_string())
                    .to_string()
            })
            .to_string();
        Ok(result)
    }

    /// Process prompts that are to effect or inspect the programme itself
    /// `prommpt` is what the user entered after the initial "!"
    fn process_meta(
        &mut self,
        prompt: &str,
        api_interface: &mut ApiInterface,
    ) -> Result<String, Box<dyn Error>> {
        let mut meta = prompt.split_whitespace();
        // The first word is: "!"
        // The rest of the words are commands for the programme to interpret.

        let response_text: String;
        if let Some(cmd) = meta.nth(1) {
            // Handle commands here
            match cmd {
                "f" => {
                    // List files
                    let vl = api_interface.files_list().unwrap();

		    // Sort the returned list.  The OpenAI API returns
		    // files in a random order
		    let mut sorted_vec = vl.body;
		    sorted_vec.sort();
                    response_text = format!(
                        ".....File ID...................Name{}",
                        sorted_vec
                            .iter()
                            .fold(String::new(), |a, b| format!("{a}\n{}: {}", b.1, b.0))
                    );
                }
                "fu" => {
                    let file_name: String = meta.collect::<Vec<&str>>().join(" ");
                    if file_name.is_empty() {
                        response_text = format!(
                            "Enter an audio file to transcribe: {}",
                            current_dir()?.display()
                        );
                    } else if PathBuf::from(file_name.as_str()).exists() {
                        response_text = match api_interface
                            .files_upload_fine_tuning(Path::new(file_name.as_str()))
                        {
                            Ok(r) => r.body,
                            Err(err) => format!("{err}: Failed to upload {file_name}"),
                        };
                    } else {
                        response_text = format!(
                            "{file_name} dose not exist.  Paths relative to {}",
                            current_dir()?.display()
                        );
                    }
                }
		"fi" => {
		    // File info
                    if let Some(file_id) =  meta.next() {
			response_text = match api_interface.file_info(file_id.to_string()) {
			    Ok(s) => s.body,
			    Err(err) => format!("{err} Failed to delete"),
			};
		    }else{
			response_text = "Enter a file ID".to_string();
		    }
		}
		"fc" => {
		    // File contents
                    if let Some(file_id) =  meta.next() {
			let local_file = meta.next();
			response_text = match api_interface.file_contents(file_id.to_string()) {
			    Ok(s) => if let Some(local_file) = local_file {
				let mut file = File::create(local_file)?;
				file.write_all(s.body.as_bytes())?;
				"success".to_string()
			    }else{
				s.body
			    },
			    Err(err) => format!("{err} Failed to get contents"),
			};
		    }else{
			response_text = "Enter a file ID".to_string();
		    }
		}
		"fd" => {
		    // Delete a file
                    let file_id: String = meta.collect::<Vec<&str>>().join(" ");
		    response_text = match api_interface.files_delete(file_id) {
			Ok(_) => "Deleted".to_string(),
			Err(err) => format!("{err} Failed to delete"),
		    };
		}
                "p" => {
                    response_text = format!(
                        "OpenAI Interface: {api_interface}\n\
			 Record File:{}\n\
			 Model: {}\n\
			 Model Mode: {}\n\
			 Image: {:#?}\n\
			 mask: {:#?}\n\
			 audio file:{:#?}\n\
			 Completions{}",
                        // Display the parameters
                        self.record_file,
			self.model,
			self.model_mode,
			self.image,
			self.mask,
			self.audio_file,
			self.local_data.keys().fold("".to_string(), |a, b| format!("{a}\n\t{b}")),
                    );
                }
		"mm" => {
		    // Display all the info we can get about models
		    response_text = match api_interface.model_list() {
			Ok(s) => s.body,
			Err(err) => format!("{err}: Failed to get model list"),
		    }
		}
                "md" => {
                    // Display known models
                    let mut model_list: Vec<&str> = self.model_mode.models_available();
                    model_list.sort();
                    response_text = format!(
                        "Models for mode: {}: {}",
                        self.model_mode,
                        model_list
                            .iter()
                            .fold(String::new(), |a, b| format!("{a}\n\t{b}"))
                    );
                }
                "ms" => {
                    // Set a model
                    if let Some(model_name) = meta.next() {
                        response_text = format!("New model: {model_name}");
                        self.model = model_name.to_string();
                    } else {
                        response_text = "No model".to_string();
                    }
                }
                "ml" => {
                    response_text = "Modes\n\t\
				     completions\n\t\
				     chat\n\t\
				     image\n\t\
				     image_edit\n\t\
				     audio_transcription\n\t\
				     "
			.to_string()
                }
                "m" => {
                    // Set the mode (effectively the API endpoint at OpenAI
                    match meta.next() {
                        // "! m" on its own to get a list of models
                        // "! m <model name>" to change it
                        Some(mode) => match mode {
                            "completions" => {
                                response_text = "Model mode => Completions\n".to_string();
                                self.model_mode = ModelMode::Completions;
                            }
                            "chat" => {
                                // A conversation with the LLM. `system_prompt` sets
                                // the tone of the conversation.  It can be over
                                // ridden here, and there must be some prompt
                                let purpose = meta.collect::<Vec<&str>>().join(" ");
                                if purpose.is_empty()
                                    && api_interface.context.purpose.is_empty()
                                {
                                    response_text =
                                        "Provide a system prompt for the chat".to_string();
                                } else {
                                    self.model_mode = ModelMode::Chat;
                                    response_text = "Model mode => Chat\n".to_string();
                                    if !purpose.is_empty() {
                                        api_interface.context.purpose = purpose;
                                    }
                                }
                            }
                            "image" => {
                                // Create images from prompts.  If a file is passed in
                                // it is an image to edit, so the mode is set to
                                // `ImageEdit`
                                let file_name: String = meta.collect::<Vec<&str>>().join(" ");
                                if file_name.is_empty() {
                                    // User is going to get AI to generate the image
                                    self.model_mode = ModelMode::Image;
                                    response_text = "Model mode => Image\n".to_string();
                                } else {
                                    // User is supplying an image
                                    if PathBuf::from(file_name.as_str()).exists() {
                                        self.image = Some(PathBuf::from(file_name));
                                        self.model_mode = ModelMode::ImageEdit;
                                        response_text = "Model mode => ImageEdit\n".to_string();
                                    } else {
                                        self.model_mode = ModelMode::Image;
                                        response_text =
                                            "File: {file_name} does not exist.  Model mode => Image\n"
                                            .to_string();
                                    }
                                }
                            }
                            "image_edit" => {
                                // Edit an image.
                                match self.model_mode {
                                    ModelMode::Image => {
                                        if self.image.is_none() && self.focus_image_url.is_none() {
                                            response_text = format!(
                                                "Cannot switch to ImageEdit mode \
						 from {} mode until you have created \
						 an image.  Enter a prompt to create an image",
                                                self.model_mode
                                            );
                                        } else if self.mask.is_none() {
                                            response_text = format!(
                                                "Cannot switch to ImageEdit mode \
						 from {} mode until you have created \
						 a mask.",
                                                self.model_mode
                                            );
                                        } else {
                                            response_text = "Edit image".to_string();
                                            self.model_mode = ModelMode::ImageEdit;
                                        }
                                    }
                                    _ => {
                                        response_text = format!("Cannot switch to ImageEdit mode from {} mode.  Must be in Image mode", self.model_mode);
                                    }
                                };
                            }
                            "audio_transcription" => {
                                if self.audio_file.is_none() {
                                    response_text = "Add an audio file before switching to audio_transcription mode".to_string();
                                } else {
                                    self.model_mode = ModelMode::AudioTranscription;
                                    response_text = "Audio Transcription mode".to_string();
                                }
                            }
                            _ => response_text = format!("{mode} not a Model Mode\n"),
                        },
                        None => {
                            response_text = "Model modes\n\
					     completions\n\
					     chat\n\
					     image\n\
					     image_edit\n\
					     audio_transcription\n"
                                .to_string()
                        }
                    }
                }
                "dx" => {
                    response_text = api_interface.context.as_string();
                }
                "cx" => {
                    response_text = "Clear context".to_string();
                    api_interface.clear_context();
                }
		"ppx" => {
		    // Print out the conversation to the passed path
		    // in a human readable form
			    
		    let file_path: String = meta.collect::<Vec<&str>>().join(" ");
		    response_text = match File::create(file_path.clone()) {
			Ok(mut f) => {
			    // Created file
			    // Save the context into the specified file
			    let context: Context = api_interface.get_context()?;
			    // `context` has query/response pairs.  So has an even length
			    assert!(context.len() % 2 == 0);
			    let context = CliInterface::pretty_print_conversation(context)?;
			    f.write_all(context.as_bytes())?;
			    format!("Wrote context to {file_path}")
			}
		    
			Err(err) => {
			    // Failed to create file
			    format!("{err}: Failed to open file at: {file_path}")
			}

		    };
		}
		
                "v" => {
                    // set verbosity
                    if let Some(v) = meta.next() {
                        response_text = match v.parse::<usize>() {
                            Ok(v) => {
                                self.verbose = v;
                                format!("Verbosity set to {v}\n")
                            }
                            Err(err) => format!("Cannot make a usize from {v} because: {err}\n"),
                        }
                    } else {
                        response_text = "No verbosity level passed".to_string();
                    }
                }
                "k" => {
                    // set tokens
                    if let Some(t) = meta.next() {
                        response_text = match t.parse::<u32>() {
                            Ok(t) => {
                                api_interface.tokens = t;
                                format!("New tokens: {t}\n")
                            }
                            Err(err) => format!("Cannot make a float from {t} because: {err}\n"),
                        };
                    } else {
                        response_text = "No tokens".to_string();
                    }
                }
                "t" => {
                    // set temperature
                    if let Some(t) = meta.next() {
                        response_text = match t.parse::<f32>() {
                            Ok(t) => {
                                if (0.0_..=2.0).contains(&t) {
                                    api_interface.temperature = t;
                                    format!("New temperature: {t}\n")
                                } else {
                                    "A float between 0 and 2\n".to_string()
                                }
                            }
                            Err(err) => format!("Cannot make a float from {t} because: {err}\n"),
                        }
                    } else {
                        response_text = "No temperature".to_string();
                    }
                }
                "sp" => {
                    if self.model_mode != ModelMode::Chat {
                        response_text = "This only makes sense in Chat mode".to_string();
                    } else {
                        let system_prompt = meta.collect::<Vec<&str>>().join(" ");
                        if system_prompt.is_empty() {
                            if api_interface.context.purpose.is_empty() {
                                response_text = "Provide a system prompt for the chat".to_string();
                            } else {
                                response_text =
                                    format!("System Prompt {}", api_interface.context.purpose);
                            }
                        } else {
                            response_text = format!("System Prompt {system_prompt}");
                            api_interface.context.purpose = system_prompt;
                        }
                    }
                }
                "ci" => {
                    // Clear `api_imterface.image` and api_interface.miage_focus_url`
                    self.image = None;
                    self.focus_image_url = None;

                    // If mode is ImageEdit set it to Image
                    if self.model_mode == ModelMode::ImageEdit {
                        //		self.api
                        response_text = format!("Image cleared. Mode: {}", self.model_mode);
                    } else {
                        response_text = "Image cleared".to_string();
                    }
                }
                "a" => {
                    let file_name: String = meta.collect::<Vec<&str>>().join(" ");
                    if file_name.is_empty() {
                        response_text = format!(
                            "Enter an audio file to transcribe: {}",
                            current_dir()?.display()
                        );
                    } else if PathBuf::from(file_name.as_str()).exists() {
                        self.model_mode = ModelMode::AudioTranscription;
                        self.audio_file = Some(file_name.clone());
                        let _path = Path::new(file_name.as_str());
                        response_text = format!(
                            "Audio Transcription mode.  \
			     File: {file_name}"
                        );
                    } else {
                        response_text = format!(
                            "{file_name} dose not exist.  Paths relative to {}",
                            current_dir()?.display()
                        );
                    }
                }
                "mask" => {
                    // Set a mask
                    let file_name: String = meta.collect::<Vec<&str>>().join(" ");
                    if file_name.is_empty() {
                        response_text = format!(
                            "Enter the mask file path relative to: {}",
                            current_dir()?.display()
                        );
                    } else if PathBuf::from(file_name.as_str()).exists() {
                        self.mask = Some(PathBuf::from(file_name));
                        response_text = format!("Mask set to: {:?}", self.mask.clone().unwrap());
                    } else {
                        response_text = format!(
                            "{file_name} dose not exist.  Paths relative to {}",
                            current_dir()?.display()
                        );
                    }
                }
		"ftr" => {
		    // Retrieve tune info
                    let id: String = meta.collect::<Vec<&str>>().join(" ");
		    response_text = match api_interface.fine_tune_retrieve(id.as_str()) {
			Ok(res) => res.body,
			Err(err) => format!("{err}: Failed ftr => fine_tune_retrieve {id}"),			
		    }
		}
		"ftl" => {
		    // List fine tune models
		    response_text = match api_interface.fine_tune_list() {
			Ok(vs) => vs.body,
			Err(err) => format!("{err}: Failed ftl => fine_tune_list"),
		    };
		}
		"ft" => {
		    // Start fine tuning a model
                    match meta.next() {
                        Some(name) => {
			    response_text = match api_interface.fine_tune_create(name.to_string()) {
				Ok(result) => result.body,
				Err(err) => format!("{err}: Failed fine_tune_create{name}"),
			    };
			},
                        None => {
                            response_text = "Cannot get name".to_string()
                        }
                    };
		}
			
			    
		    
                "fl" => {
                    // Load a file's contents into a buffer to use as
                    // part of a prompt
                    match meta.next() {
                        Some(name) => {
			    let file_name: String = meta.collect::<Vec<&str>>().join(" ");
			    if file_name.is_empty() {
				response_text = format!(
				    "! fl <name> <file path>: The contents of a file\
				     is bound to the name for use in prompts: {{name}}\
				     expands to file content.  The path is relative to:\
				     {}",
				    current_dir()?.display()
				);
			    } else if !PathBuf::from(file_name.as_str()).exists() {
				response_text = format!(
				    "{file_name} does not exist.  Paths relative to {}",
				    current_dir()?.display()
				);
			    } else {
				// Got the name and the file

				// Get file contents
				let contents = fs::read_to_string(Path::new(&file_name))?;
				// Associate the name and the contents
				_ = self.local_data.insert(name.to_string(), contents);
				response_text = format!("Loaded {name}");

			    }
			},
                        None => {
                            response_text = "Cannot get name".to_string()
                        }
                    };

                }
		"sx" => {
		    let file_path: String = meta.collect::<Vec<&str>>().join(" ");
		    // Save the context into the specified file
		    let context = api_interface.get_context()?;
		    let serialized_context = serde_json::to_string(&context)?;
		    response_text = format!("Saved context to {}", file_path);
		    let mut file = File::create(file_path)?;
		    file.write_all(serialized_context.as_bytes())?;

		}
		"rx" => {
		    // Read the context from a file.
		    let file_path: String = meta.collect::<Vec<&str>>().join(" ");
		    if file_path.is_empty() {
			response_text = format!(
			    "Enter the path of the file containing the context: {}",
			    current_dir()?.display()
			);
		    } else if PathBuf::from(file_path.as_str()).exists() {
			// Read the contents of the file.
			let file_contents = fs::read_to_string(Path::new(&file_path))?;
			// Deserialize the Vec<String> from the file contents.
			let context: Context = serde_json::from_str(&file_contents)?;

			// Set the context in the API interface.
			api_interface.set_context(context);

			response_text = "Context loaded from file.".to_string();
		    } else {
			response_text = format!(
			    "{file_path} does not exist. Paths relative to {}",
			    current_dir()?.display()
			);
		    }
		}
                "?" => {
                    response_text = "\
		    p  Display settings\n\
		    mm Display all available information about all models\n\
		    md Display all available models for the current mode\n\
		    ms <model> Change the current model\n\
		    ml List modes\
		    m  <mode> Change mode (API endpoint\n\
		    dx Display context (for chat)\n\
		    cx Clear context\n\
		    ppx <path> Pretty print conversation to path\n\
		    v  Set verbosity\n\
		    k  Set max tokens for completions\n\
		    t  Set temperature for completions\n\
		    sp Set system prompt (after `! cc`\n\
		    ci Clear image\
		    mask <path> Set the mask to use in image edit mode.  A 1024x1024 PNG with transparent mask\n\
		    a <path> Audio file for transcription\n\
		    ci Clear the image stored for editing\n\
		    f List the files stored on the server\n\
		    fu <path> Upload a file of fine tuning data\n\
		    fd <file id> Delete a file\n\
		    fi <file id> Get information about file\n\
		    fc <file id> [destination_file] Get contents of file\n\
		    ft <file ID> Start a fine tune model using a JASONL training file \n\
		    ftl List fine tunes\n\
		    ftr <id> Retrieve the info for a fine tune\n\
		    fl <name> <path>  Associate the contents of the `path` with `name` for use in prompts like: {{name}}\n\
		    sx <path>  Save the context to a file at the specified path\n\
		    rx <path>  Restore the context from a file at the specified path\n\
 		    ?  This text\n"
                        .to_string()
                }
                _ => response_text = format!("Unknown command: {cmd}\n"),
            };
        } else {
            response_text = "Enter a meta command".to_string();
        }
        Ok(response_text)
    }

    /// Data about the request before it goes out.  Cach headers, only
    /// output changes
    pub fn after_request(
        &mut self,
        response_headers: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = "".to_string();
        if self.verbose > 0 {
            for k in response_headers.keys() {
                if let Some(v) = self.header_cache.get(k) {
                    if v == response_headers.get(k).unwrap() {
                        continue;
                    }
                }
                self.header_cache
                    .insert(k.clone(), response_headers.get(k).unwrap().clone());
                result += &format!("{k}: {}\n", response_headers[k]);
            }
        } else {
        }
        Ok(result)
    }

    pub fn pretty_print_conversation(context: Context) -> Result<String, Box<dyn Error>> {
        let saved_context = context.as_string();
        Ok(saved_context)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Get the command line options
    let cmd_line_opts = Arguments::parse();

    // API key.  Stored in openai_interface
    let _key_binding: String;
    let api_key = match cmd_line_opts.api_key.as_deref() {
        Some(key) => key,
        None => {
            _key_binding = env::var("OPENAI_API_KEY").unwrap();
            _key_binding.as_str()
        }
    };

    // The model.  Stored in openai_interface
    let model = cmd_line_opts.model.as_str();

    // Maximum tokens.  Stored in openai_interface.  TODO: Store in CliInterface?
    let tokens: u32 = cmd_line_opts.max_tokens;

    // .  Stored in openai_interface
    let temperature: f32 = cmd_line_opts.temperature;

    // The mode.  Stored in openai_interface.  TODO Should be stored
    // in CliInterface
    let mode: ModelMode = match ModelMode::from_str(cmd_line_opts.mode.as_str()) {
        Ok(m) => m,
        Err(_) => panic!("{} is an invalid mode", cmd_line_opts.mode.as_str()),
    };

    let mut cli_interface = CliInterface {
        record_file: DEFAULT_RECORD_FILE.to_string(),
        history_file: DEFAULT_HISTORY_FILE.to_string(),
        verbose: 0,
        audio_file: None,
        model: model.to_string(),
        model_mode: mode.clone(),
        focus_image_url: None,
        mask: None,
        image: None,
        header_cache: HashMap::new(),
        cost: 0.0,
        local_data: HashMap::new(),
    };
    // The file name of the conversation record
    cli_interface.record_file = cmd_line_opts.record_file;
    // Keep  record of the conversations
    let mut options = OpenOptions::new();
    let mut conversation_record_file: File = options
        .write(true)
        .append(true)
        .create(true)
        .open(cli_interface.record_file.as_str())
        .unwrap();
    let mut read_line: Editor<MyHelper, FileHistory> = cli_interface.set_up_read_line()?;
    let mut prompt: String;
    let mut api_interface = ApiInterface::new(api_key.to_string(), tokens, temperature);
    if let Some(sp) = cmd_line_opts.system_prompt {
        api_interface.context.purpose = sp;
    }
    loop {
        // Read the input text
        let p = format!("{}/{mode}:> ", cli_interface.model);
        read_line.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
        let readline = read_line.readline(&p);
        let input = match readline {
            Ok(line) => line,
            Err(err) => {
                // Exit from loop
                eprintln!("Readline {err}");
                break;
            }
        };
        read_line.add_history_entry(input.as_str())?;

        // Expand and varoables i the prompt

        prompt = cli_interface.expand_variables(input.clone())?;
        _ = conversation_record_file
            .write(
                format!(
                    "Q: {}\n{}\n",
                    Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    prompt
                )
                .as_bytes(),
            )
            .unwrap();

        // The response that will be displayed to the user.
        // It can be from more than one source:
        // * It can be a response from the LLM
        // * It can be information about the state of this programme
        // * It can be the completion message (error or success) for
        //   some change to the state of this programme
        let response_text: String;
        let prompt = prompt.as_str().trim();
        if prompt.is_empty() {
            response_text = "No prompt\n".to_string();
        } else if prompt.starts_with('!') {
            let cprompt = prompt;
            response_text = cli_interface.process_meta(cprompt, &mut api_interface)?;
        } else {
            // Send the prompt to the LLM
            let start_time = Local::now();
            let response = match cli_interface.model_mode {
                ModelMode::AudioTranscription => {
                    let prompt_param: Option<&str> = if prompt.is_empty() {
                        None
                    } else {
                        Some(prompt)
                    };
                    match api_interface.audio_transcription(
                        Path::new(cli_interface.audio_file.as_ref().unwrap().as_str()),
                        prompt_param,
                    ) {
                        Ok(r) => {
                            format!("{}\n{}", cli_interface.after_request(r.headers)?, r.body,)
                        }
                        Err(err) => format!("{err}"),
                    }
                }
                ModelMode::Chat => match api_interface.chat(prompt, cli_interface.model.as_str()) {
                    Ok(apt_result) => {
                        // Get ready
                        cli_interface.cost = apt_result
                            .headers
                            .get("Cost")
                            .unwrap()
                            .parse::<f64>()
                            .unwrap();
                        fn update_spent(cost: f64) -> impl FnMut(SharedState) -> SharedState {
                            move |mut ss| {
                                ss.spent += cost;
                                ss
                            }
                        }

                        // Call read_write_atomic with the closure
                        let ss: SharedState = match SharedState::read_write_atomic(update_spent(
                            cli_interface.cost,
                        )) {
                            Ok(ss) => ss,
                            Err(err) => panic!("{err}: Failed to update costs"),
                        };

                        let this_cost = cli_interface.cost;
                        let total_cost = ss.spent;
                        let conversation_cost = api_interface.context.cost;
                        format!(
                            "{:.2}/{:.2}/{:.2}:{}{}\n{}",
                            this_cost,
                            conversation_cost,
                            total_cost,
                            api_interface.context.len(),
                            cli_interface.after_request(apt_result.headers)?,
                            apt_result.body,
                        )
                    }
                    Err(err) => format!("{err}"),
                },

                ModelMode::Completions => {
                    match api_interface.completion(prompt, cli_interface.model.as_str()) {
                        Ok(r) => {
                            format!("{}\n{}", cli_interface.after_request(r.headers)?, r.body,)
                        }
                        Err(err) => format!("{err}"),
                    }
                }
                ModelMode::Image => match api_interface.image(prompt) {
                    Ok(r) => {
                        // Returned a url
                        // Store the link to the image for refinement
                        cli_interface.focus_image_url = Some(r.body);
                        // Open image
                        let url: String = cli_interface.focus_image_url.as_ref().unwrap().clone();
                        match cli_interface.process_image_url(&url) {
                            Ok(_) => format!("Opened: {url}"),
                            Err(err) => format!("{err}: Failed to open: {url}"),
                        }
                    }
                    Err(err) => format!("{err}"),
                },
                ModelMode::ImageEdit => {
                    match api_interface.image_edit(
                        prompt,
                        cli_interface.image.clone().unwrap().as_path(),
                        cli_interface.mask.clone().unwrap().as_path(),
                    ) {
                        Ok(r) => {
                            // Open image
                            match cli_interface.process_image_url(r.body.as_str()) {
                                Ok(_) => format!("Opened: {}", r.body),
                                Err(err) => format!("{err}: Failed to open: {}", r.body),
                            }
                        }
                        Err(err) => format!("{err}"),
                    }
                }
            };
            let end_time = Local::now();
            let duration = end_time.signed_duration_since(start_time);
            response_text = format!("{} seconds\n{response}", duration.num_seconds());
        }

        // Put state dependant logic here to display useful information
        if cli_interface.verbose > 0 {
            eprintln!(
                "Conversation: {} turns and {} bytes",
                api_interface.context.len(),
                api_interface.context.sz()
            );
        }

        _ = conversation_record_file
            .write(
                format!(
                    "A: {}\n{response_text}\n",
                    Local::now().format("%Y-%m-%dT%H:%M:%S"),
                )
                .as_bytes(),
            )
            .unwrap();
        println! {"{response_text}"};
    }

    read_line
        .append_history(cli_interface.history_file.as_str())
        .unwrap();
    read_line.clear_history().unwrap();
    Ok(())
}
