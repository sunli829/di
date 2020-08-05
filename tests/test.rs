use di::*;
use std::collections::HashMap;
use std::sync::Mutex;

trait Log {
    fn log(&self, msg: &str);
}

#[derive(Component)]
#[di(interface = "Log")]
struct ConsoleLog {
    #[value(default = "default_prefix")]
    prefix: String,
}

fn default_prefix() -> String {
    "#".to_string()
}

impl Log for ConsoleLog {
    fn log(&self, msg: &str) {
        println!("{}/ {}", self.prefix, msg);
    }
}

trait Storage {
    fn set(&self, name: &str, value: i32);

    fn get(&self, name: &str) -> Option<i32>;
}

#[derive(Component)]
#[di(interface = "Storage")]
struct MemoryStorage {
    #[inject]
    log: Injected<dyn Log>,

    values: Mutex<HashMap<String, i32>>,
}

impl Storage for MemoryStorage {
    fn set(&self, name: &str, value: i32) {
        let mut values = self.values.lock().unwrap();
        self.log.log(&format!("set {}={}", name, value));
        values.insert(name.to_string(), value);
    }

    fn get(&self, name: &str) -> Option<i32> {
        let values = self.values.lock().unwrap();
        self.log.log(&format!("get {}", name));
        values.get(name).cloned()
    }
}

#[test]
fn test_component() {
    SystemBuilder::new()
        .config_file("tests/config.json")
        .register::<ConsoleLog>()
        .register::<MemoryStorage>()
        .run(|| {
            let storage = create_context().get::<dyn Storage>("storage").unwrap();
            storage.set("a", 10);
            storage.set("b", 20);
            storage.get("a");
        });
}
