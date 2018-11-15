use std::error::Error;
use std::fmt;

pub struct Command<E, U> {
    pub exec: Box<Fn() -> Result<E, Box<Error>>>,
    pub unexec: Box<Fn() -> Result<U, Box<Error>>>,
}

impl<E, U> fmt::Debug for Command<E, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Command {{ exec: Box(callback), unexec: Box(callback) }}"
        )
    }
}

#[derive(Debug)]
pub struct Invoker<E, U> {
    commands: Vec<Command<E, U>>,
    undo_commands: Vec<Command<E, U>>,
}

impl<E, U> Invoker<E, U> {
    pub fn new() -> Self {
        Invoker {
            commands: Vec::new(),
            undo_commands: Vec::new(),
        }
    }

    pub fn exec(&mut self, command: Command<E, U>) -> Result<E, Box<Error>> {
        let exec = &command.exec;
        self.undo_commands.clear();
        let result = exec();
        self.commands.push(command);
        result
    }

    pub fn exec_or_undo(&mut self, command: Command<E, U>) -> Result<E, Box<Error>> {
        let result = self.exec(command);

        match result {
            err @ Err(_) => {
                self.undo();
                err
            }
            ok @ Ok(_) => ok,
        }
    }

    pub fn exec_or_undo_all(&mut self, command: Command<E, U>) -> Result<E, Box<Error>> {
        let result = self.exec(command);

        match result {
            err @ Err(_) => {
                self.undo_all();
                err
            }
            ok @ Ok(_) => ok,
        }
    }

    pub fn undo_all(&mut self) {
        for _ in 0..self.commands.len() {
            self.undo();
        }
    }

    pub fn undo(&mut self) -> Result<U, Box<Error>> {
        let command = self.commands.pop().unwrap();
        let unexec = &command.unexec;
        let result = unexec();
        self.undo_commands.push(command);
        result
    }

    pub fn redo(&mut self) -> Result<E, Box<Error>> {
        let command = self.undo_commands.pop().unwrap();
        let exec = &command.exec;
        let result = exec();
        self.commands.push(command);
        result
    }
}

#[macro_export]
macro_rules! exec_by_name {
    (
        $name:ident,
        $invoker:ident,
        {
            exec: $($move_exec:ident)* $exec_body:block,
            unexec: $($move_unexec:ident)* $unexec_body:block
        }
    ) => {
        $invoker.$name(Command {
            exec: Box::new($($move_exec)* || $exec_body),
            unexec: Box::new($($move_unexec)* || $unexec_body),
        })
    };
    (
        $name:ident,
        $invoker:ident,
        $($move_exec:ident)* $exec_body:block
    ) => {
        $invoker.$name(Command {
            exec: Box::new($($move_unexec)* || $exec_body),
            unexec: Box::new(|| Ok(())),
        })
    };
}

#[macro_export]
macro_rules! exec {
    ($($args:tt)+) => { exec_by_name!(exec, $($args)+) }
}

#[macro_export]
macro_rules! exec_or_undo_all {
    ($($args:tt)+) => { exec_by_name!(exec_or_undo_all, $($args)+) }
}

#[macro_export]
macro_rules! exec_or_undo {
    ($($args:tt)+) => { exec_by_name!(exec_or_undo, $($args)+) }
}
