use dirs::home_dir;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{create_dir, remove_file, DirEntry, File};
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

fn get_log_dir() -> PathBuf {
    [home_dir().unwrap(), ".local/share/budgr/logs/".into()]
        .iter()
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PurchaseType {
    Groceries,
    Leisure,
    Bill,
    School,
}

impl fmt::Display for PurchaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurchaseType::Groceries => write!(f, "Groceries"),
            PurchaseType::Leisure => write!(f, "Leisure"),
            PurchaseType::Bill => write!(f, "Bill"),
            PurchaseType::School => write!(f, "School"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Purchase {
    pub name: String,
    pub tag: PurchaseType,
    pub cost: i64,
}

// A log is a list of purchases
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Log {
    pub purchases: Vec<Purchase>,
    pub name: String,
}

impl Default for Log {
    fn default() -> Log {
        Log {
            purchases: Vec::new(),
            name: "new-log".to_string(),
        }
    }
}

struct SerializeLog {
    name: String,
    contents: String,
}

pub struct Budgr {
    pub logs: Vec<Log>,
}

// everything needed to interact with the data
impl Budgr {
    // change this to deserialize, not just make new variables
    pub fn new() -> Self {
        let logs: Vec<Log> = Vec::new(); // temporary (before frontend)
        Self { logs }
    }

    pub fn serialize(&self) -> Result<(), std::io::Error> {
        // TODO: make directory if it does not exist
        // serialize the logs into SerializeLog type to be put away into files
        let mut serialize_logs: Vec<SerializeLog> = Vec::new();
        for log in self.logs.iter() {
            let contents = serde_json::to_string(&log);
            // skip errors
            if let Ok(contents) = contents {
                serialize_logs.push(SerializeLog {
                    name: log.name.clone() + ".json",
                    contents,
                });
            }
        }

        // put into files (might want to do this more efficiently in the future)
        for log in serialize_logs.iter() {
            let mut path: PathBuf = get_log_dir();
            path.push(log.name.clone());
            let mut file = match File::create(path) {
                Ok(file) => file,
                Err(err) => panic!("create file failed: {}", err),
            };
            match file.write_all(log.contents.as_bytes()) {
                Ok(_) => println!("successful write"),
                Err(err) => panic!("write all failed: {}", err),
            }
        }
        Ok(())
    }

    pub fn new_log(&mut self, name: String) -> Result<(), String> {
        // check if log exists already
        for log in self.logs.iter() {
            if log.name == name {
                return Err("File alrealdy exists".to_string());
            }
        }

        // check if there are spaces
        for c in name.chars() {
            if c == ' ' {
                return Err("Spaces not allowed in log names".to_string());
            }
        }

        self.logs.push(Log {
            name,
            ..Default::default()
        });
        Ok(())
    }

    pub fn delete_log(&mut self, log_index: usize) -> Result<(), String> {
        if log_index > self.logs.len() || self.logs.len() == 0 {
            return Err("ERROR: log index out of range".to_string());
        }

        let path = get_path_to_log(self.logs[log_index].name.as_str());
        self.logs.remove(log_index);
        println!("PATH TO REMOVE: {}", path.display());
        remove_file(path).unwrap();

        Ok(())
    }

    pub fn add_purchase(
        &mut self,
        log_index: usize,
        name: String,
        tag: PurchaseType,
        cost: i64,
    ) -> Result<(), String> {
        let valid_index = match log_index {
            _ if self.logs.len() < log_index => return Err("ERROR: Index too large".to_string()),
            _ => log_index,
        };

        self.logs[valid_index].add_purchase(name, tag, cost);

        Ok(())
    }

    pub fn get_total(&self, log_index: usize) -> Result<i64, String> {
        let valid_index = match log_index {
            _ if self.logs.len() < log_index => return Err("ERROR: Index too large".to_string()),
            _ => log_index,
        };

        Ok(self.logs[valid_index]
            .purchases
            .iter()
            .map(|purchase| purchase.cost)
            .sum())
    }

    pub fn print_logs(&self) {
        (0..self.logs.len()).for_each(|i| self.print_log(i));
    }

    fn print_purchase(purchase: &Purchase) {
        println!(
            "purchase: name: {}, tag: {}, purchase: {}",
            purchase.name, purchase.tag, purchase.cost
        );
    }

    pub fn print_log(&self, log_index: usize) {
        println!(
            "Log Print: {}, log index: {}",
            self.logs[log_index].name, log_index
        );
        for purchase in self.logs[log_index].purchases.iter() {
            Budgr::print_purchase(&purchase);
        }
        println!("\n");
    }

    pub fn get_expenses(&self, log_index: usize) -> Result<i64, String> {
        let valid_index = match log_index {
            _ if self.logs.len() < log_index || self.logs.len() == 0 => {
                return Err("ERROR: Index incorrect".to_string())
            }
            _ => log_index,
        };

        let total = self.logs[valid_index]
            .purchases
            .iter()
            .map(|p| p.cost)
            .sum();

        Ok(total)
    }
}

pub fn read_budgr_from_directory() -> Result<Budgr, std::io::Error> {
    let mut budgr: Budgr = Budgr::new();

    // there should be a path in .local for this program in the future, make a const for this?
    let mut glob_str = get_log_dir();
    glob_str.push("*.json");
    // TODO: figure out a way to handle glob_str that isn't completely stupid
    for path in glob(glob_str.into_os_string().into_string().unwrap().as_str()).unwrap() {
        if let Ok(path) = path {
            println!("LOADED FILE: {}", path.display());
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            budgr.logs.push(serde_json::from_reader(reader)?);
        }
    }

    Ok(budgr)
}

// check if a given index is valid
fn check_index<T>(vec: Vec<T>, index: usize) -> Result<usize, String> {
    match index {
        _ if index > vec.len() => return Err("index too large".to_string()),
        _ => return Ok(index),
    }
}

// return the absolute path to a log json file
fn get_path_to_log(log_name: &str) -> PathBuf {
    let mut path = get_log_dir();
    path.push(format!("{}{}", log_name, ".json"));

    path
}

impl Log {
    pub fn get_total(&self) -> i64 {
        self.purchases.iter().map(|purchase| purchase.cost).sum()
    }
    pub fn add_purchase(&mut self, name: String, tag: PurchaseType, cost: i64) {
        self.purchases.push(Purchase { name, tag, cost });
    }
}
