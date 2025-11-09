use chrono::Local;
use rusqlite::Connection;
use strsim::levenshtein;

#[derive(Debug)]
struct DBEntry {
    name: String,
    url: String,
}
trait Commands {
    fn getname(&self) -> String;
    fn exec(&mut self, args: &[&str]);
}

struct Terminal {
    commands: Vec<Box<dyn Commands>>,
}
struct PingCommand {}
struct CountCommand {}
struct TimesCommand {
    count: u32,
}
struct EchoCommand {}
struct DateCommand {}
struct BkCommand {
    conn: Connection,
}

impl Commands for PingCommand {
    fn getname(&self) -> String {
        "ping".to_string()
    }
    fn exec(&mut self, _args: &[&str]) {

        println!("pong!");
    }
}
impl Commands for CountCommand {
    fn getname(&self) -> String {
        "count".to_string()
    }
    fn exec(&mut self, args: &[&str]) {
        if !args.is_empty() {
            //println!("{args}");
            println!("counted {} args",args.len());
        } else {
            println!("Eroare pentru functia count, parametri insuficienti");
        }
    }
}
impl Commands for TimesCommand {
    fn getname(&self) -> String {
        "times".to_string()
    }
    fn exec(&mut self, _args: &[&str]) {

        self.count += 1;
        println!("{}", self.count);
    }
}
impl Commands for EchoCommand {
    fn getname(&self) -> String {
        "echo".to_string()
    }
    fn exec(&mut self, args: &[&str]) {
        println!("{}",args.join(" "))
    }
}
impl Commands for DateCommand {
    fn getname(&self) -> String {
        "date".to_string()
    }

    fn exec(&mut self, _args: &[&str]) {

        let timp = Local::now();
        println!("{}", timp.format("%Y-%m-%d %H:%M:%S"));
    }
}
impl Commands for BkCommand {
    fn getname(&self) -> String {
        "bk".to_string()
    }
    fn exec(&mut self, args: &[&str]) {
        let create = r"
create table if not exists links (
    name text    not null,
    url  text    not null
);
";
        match self.conn.execute(create, ()) {
            Ok(_) => match args[0] {
                "add" => {
                    if args.len() >= 3 {
                        let name = args[1];
                        let url = args[2];
                        match self.conn.execute(
                            "insert into links (name, url) values (?1, ?2);",
                            (name, url),
                        ) {
                            Ok(_) => {
                                println!("Adaugat cu success perechea ({name},{url}) in db")
                            }
                            Err(e) => {
                                println!("Eroare la inserare in db: {e}")
                            }
                        }
                    } else {
                        println!("Add nu are suficiente argumente");
                    }
                }
                "search" => {
                    if args.len() >= 2 {
                        let name = args[1];
                        match self.conn.prepare("select * from links") {
                            Ok(mut stmt) => {
                                match stmt.query_map([], |row| {
                                    Ok(DBEntry {
                                        name: row.get("name")?,
                                        url: row.get("url")?,
                                    })
                                }) {
                                    Ok(links_iter) => {
                                        let mut cnt: u32 = 0;
                                        for i in links_iter {
                                            match i {
                                                Ok(obj) => {
                                                    if obj.name.contains(name) {
                                                        println!("{} -> {}", obj.name, obj.url);
                                                        cnt += 1;
                                                    }
                                                }
                                                Err(e) => {
                                                    println!(
                                                        "Eroare la obtinerea unui obiect DBEntry din querry: {e}"
                                                    );
                                                }
                                            }
                                        }
                                        if cnt != 0 {
                                            println!(
                                                "S-au gasit {cnt} nume in db care contin substringul {name}"
                                            )
                                        } else {
                                            println!(
                                                "Nu s-au gasit nume in db care sa contina {name}"
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        println!("Eroare la popularea structurilor DBEntry: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Nu s-a executat calumea select * from links: {e}")
                            }
                        };
                    }
                }
                &_ => {
                    println!("Eroare, bk are optiunile: add <name> <url>, search <name>")
                }
            },
            Err(e) => {
                println!("Nu s-au inserat datele in db: {e}")
            }
        }
    }
}
impl Terminal {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
    fn register(&mut self, value: Box<dyn Commands>) {
        self.commands.push(value);
    }
    fn run(&mut self) {
        match std::fs::read_to_string("comenzi.txt") {
            Ok(comenzi) => {
                's: for linii in comenzi.lines() {
                    if linii.is_empty() {
                        continue;
                    }
                    let mut exec: Vec<&str> = linii.split_whitespace().collect();
                    let command_name = exec[0];
                    exec.remove(0);
                    let exec = exec;
                    let mut found = false;
                    for comanda in &mut self.commands {
                        if command_name == "stop" {
                            break 's;
                        }
                        if comanda.getname() == command_name {
                            comanda.exec(&exec);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        let sugestie = self
                            .commands
                            .iter()
                            .map(|cmd| {
                                (
                                    cmd.getname(),
                                    levenshtein(
                                        &cmd.getname(),
                                        command_name.to_lowercase().as_str(),
                                    ),
                                )
                            })
                            .min_by_key(|(_, dist)| *dist);

                        if let Some((nume, dist)) = sugestie {
                            if dist <= 2 {
                                println!("Ai vrut sa scrii '{nume}' in loc de {command_name}?");
                            } else {
                                println!("Comandă necunoscută: '{command_name}'");
                            }
                        } else {
                            println!("Comandă necunoscută: '{command_name}'");
                        }
                    }
                }
            }
            Err(e) => {
                println!("Eroare la citire: {e}")
            }
        }
    }
}

fn main() -> Result<(), rusqlite::Error> {
    let mut terminal = Terminal::new();
    terminal.register(Box::new(PingCommand {}));
    terminal.register(Box::new(CountCommand {}));
    terminal.register(Box::new(TimesCommand { count: 0 }));
    terminal.register(Box::new(EchoCommand {}));
    terminal.register(Box::new(DateCommand {}));
    terminal.register(Box::new(BkCommand {
        conn: Connection::open_in_memory()?,
    }));
    terminal.run();
    Ok(())
}