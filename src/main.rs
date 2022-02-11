#![feature(once_cell)]
#![feature(fn_traits)]
#![feature(panic_info_message)]

pub mod parser;
pub mod util;
pub mod event;
pub mod vm;

use std::fs::File;
use std::io::{BufRead, BufReader};
use backtrace::Backtrace;
use rustop::opts;
use crate::event::event_pipeline::EventPipeline;
use crate::vm::VM;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("----- Error -----");
        if cfg!(debug_assertions) {
            let current_backtrace = Backtrace::new();
            eprintln!("Backtrace");
            eprintln!("{:?}", current_backtrace);
        } else {
            eprintln!("Backtrace removed. Run in debug mode to show")
        }
        eprintln!("{:?}", panic_info.message().ok_or_else(|| {}).map_err(|_| "No message provided").unwrap());
        eprintln!("----- Error -----");
    }));

    let (args,_) = opts! {
        param action:String, name:"action", desc:"Action to execute. Actions: simulate, compile";
        param file:String, name:"file", desc:"File to perform the action on";
    }.parse_or_exit();

    match args.action.as_str() {
        "simulate" => {
            let lines = read(args.file);
            let split = parser::pre_parse(lines);
            let tokens = parser::tokenize(split);
            let listeners = parser::split(tokens);

            let mut vm = VM {
                pipeline: EventPipeline {
                    listeners
                }
            };
            vm.start();
        }
        "compile" => {
        }
        _ => {
            eprintln!("Invalid action! Try --help for help");
        }
    }
}

fn read(path: String) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file!");
    BufReader::new(file).lines().map(|line| line.expect("Failed to read line for some reason!")).collect()
}