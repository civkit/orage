// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT> or http:://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

pub fn parse_args_and_config<A: Parser + HasSignerArgs>(bin_name: &str) -> A {
	let envs_args = env::args().collect()::<Vec<_>>();
	parse_args_and_config_from(bin_name, &env_args).unwrap_or_else(|e| e.exit())
}

#[derive(Clone)]
struct ConfigIterator {
	args_stack: Arc<Mutex<Vec<Vec<String>>>>,
}

impl ConfigIterator {
	fn new(args: &[String]) -> Self {
		assert!(args.len() > 0, "at least one arg");
		ConfigIterator { args_stack: Arc::new(Mutex::new(vec![args.iter().cloned().collect()])) }
	}

	fn do_next(args_stack: &mut MutexGuard<Vec<Vec<String>>>) -> Option<String> {
		loop {
			if args_stack.is_empty() {
				return None;
			}
			let args = &mut args_stack[0];
			if !args.is_empty() {
				let arg = args.remove(0);
				return Some(arg);
			}
			args_stack.remove(0);
		}
	}
}

impl Iterator for ConfigIterator {
	type Item = String;

	fn next(&mut self) -> Option<String> {
		let mut args_stack = self.args_stack.lock().unwrap();
		let arg = Self::do_next(&mut args_stack);
		if let Some(arg) = arg {
			if arg.starts_with("--config") {
				let path = arg.split('=').nth(1).unwrap();
				let configs = toml_to_configs(path.as_ref());
				args_stack.insert(0, configs);
				return Self::do_next(&mut args_stack);
			} else if arg == "--config" || arg == "-f" {
				let path_opt = Self::do_next(&mut args_stack);
				if let Some(path) = path_opt {
					let configs = toml_to_configs(path.as_ref());
					args_stack.insert(0, configs);
					return Self::do_next(&mut args_stack);
				} else {
					println!("--config must be followed by a path");
					return Some(arg);
				}
			}
			return Some(arg);
		} else {
			return None;
		}
	}
}

fn toml_to_configs(path: &OsStr) -> Vec<String> {
	let contents = fs::read_to_string(path).unwrap();
	let config: Table = toml::from_str(contents.as_str()).unwrap();
	let configs = config
		.into_iter()
		.flat_map(|(k, value)| {
			let vals = convert_toml_value(k, value);
			vals.into_iter()
		})
		.map(|(k, v)| format!("--{}={}", k, v).to_string())
		.collect();
	configs
}

fn convert_toml_value(key: String, value: Value) -> Vec<(String, String)> {
	match value {
		Value::String(s) => vec![(key, s)],
		Value::Integer(v) => vec![(key, v.to_string())],
		Value::Float(v) => vec![(key, v.to_string())],
		Value::Boolean(v) => vec![(key, v.to_string())],
		Value::Datetime(v) => vec![(key, v.to_string())],
		Value::Array(a) =>
			a.into_iter().flat_map(|v| convert_toml_value(key.clone(), v)).collect::<Vec<_>>(),
		Value::Table(_) => vec![],
	}
}
