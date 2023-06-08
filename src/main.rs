use dbus::MessageType;
use dbus::arg::messageitem::MessageItem;
use dbus::{blocking::Connection, message::MatchRule, Message, strings::Member};
use dbus::channel::MatchingReceiver;
use std::time::Duration;
 
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;
use std::env;

use std::time::{SystemTime, UNIX_EPOCH};


fn main() {

    let args: Vec<String> = env::args().collect();
 
    // Connect to the session bus
    let connection = Connection::new_session().unwrap();
    let proxy = connection.with_proxy("org.freedesktop.DBus", "/org/freedesktop/DBus", Duration::from_millis(5000));
 
    let rule = MatchRule::new();
    let result: Result<(), dbus::Error> =
        proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (vec![rule.match_str()], 0u32));

    
    let mut file = File::create("logs.txt");

    match result {
        Err(e) => {
            eprintln!("Err {:?}", e);
        }
 
        Ok(_) => {
            connection.start_receive(
                rule, 
                Box::new(|msg, _| {
                    handle_message(&msg);
                    true
                }),
            );
        }
    }
 
 
    loop {
        connection.process(Duration::from_millis(1000)).unwrap();
    }
 
}



fn handle_message(msg: &Message) {
    if let Some(message) = is_notif(msg) {
        let items = message.get_items();
        let (program, name, summary) = (items.get(0), items.get(3), items.get(4));
        let notification = new_notif(program, name, summary);
        let json = format_to_json(notification);
        write_to_file(&json);
    }
}

fn is_notif(msg: &Message) -> Option<&Message> {
    if msg.msg_type() != MessageType::MethodCall { return None };
    if &*msg.interface().unwrap() != "org.freedesktop.Notifications" {return None};
    let member_res = Member::new("Notify");
    match member_res {
        Err(e) => eprint!("{}", e),
        Ok(member) => if msg.member() != Some(member) {return None;}

    }
    return Some(msg);
}

fn format_to_json(n: Notif) -> String {
    return format!("{{ \"program\": {}, \"name\": {}, \"summary\": {}, \"time\": {} }} \n",
     &n.program[4..n.program.len()-1], &n.name[4..n.name.len()-1], &n.body[4..n.body.len()-1], n.time);
}

fn write_to_file(text: &str) {
    // Open a file with append option
    let mut data_file = OpenOptions::new()
        .append(true)
        .open("logs.txt")
        .expect("cannot open file");

    // Write to a file
    data_file
        .write(text.as_bytes())
        .expect("write failed");
}

struct Notif {
    program: String,
    name: String,
    body: String,
    time: u64
}

fn new_notif(p: Option<&MessageItem>, n: Option<&MessageItem>, b: Option<&MessageItem>) -> Notif {
    Notif { program: format!("{:?}", p.unwrap()),
        name: format!("{:?}", n.unwrap()),
        body: format!("{:?}", b.unwrap()),
        time:  SystemTime::now().duration_since(UNIX_EPOCH).expect("Cant get time?").as_secs()
        }
}