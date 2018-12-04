use std::error::Error;
use std::fmt;

pub trait CommandTrait<R,E,U>: fmt::Debug {
    fn get_exec(&self) -> &Box<Fn() -> Result<R, E> + Send + Sync>;
    fn get_unexec(&self) -> &Box<Fn() -> Result<(), U> + Send + Sync>;
}

pub struct Command<R,E=Box<Error>,U=E> {
    pub exec: Box<Fn() -> Result<R, E> + Send + Sync>,
    pub unexec: Box<Fn() -> Result<(), U> + Send + Sync>,
}

impl<R,E,U> CommandTrait<R,E,U> for Command<R,E,U> {
    fn get_exec(&self) -> &Box<Fn() -> Result<R, E> + Send + Sync> {
        &self.exec
    }
    fn get_unexec(&self) -> &Box<Fn() -> Result<(), U> + Send + Sync> {
        &self.unexec
    }
}

impl<R,E,U> fmt::Debug for Command<R,E,U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command {{ exec: Box(callback), unexec: Box(callback) }}")
    }
}

#[derive(Debug)]
pub struct Invoker<R,E=Box<Error>,U=E> {
    commands: Vec<Box<dyn CommandTrait<R,E,U> + Send + Sync>>,
    undo_commands: Vec<Box<dyn CommandTrait<R,E,U> + Send + Sync>>,
}

impl<R,E,U> Invoker<R,E,U> {
    pub fn new() -> Self {
        Invoker {
            commands: Vec::new(),
            undo_commands: Vec::new(),
        }
    }

    pub fn exec(&mut self, command: impl CommandTrait<R,E,U> + Send + Sync + 'static) -> Result<R, E> {
        let exec = command.get_exec();
        self.undo_commands.clear();
        let result = exec();
        self.commands.push(Box::new(command));
        result
    }

    pub fn exec_or_undo(&mut self, command: impl CommandTrait<R,E,U> + Send + Sync + 'static) -> Result<R, E> {
        let result = self.exec(command);

        match result {
            err @ Err(_) => {
                self.undo();
                err
            }
            ok @ Ok(_) => ok,
        }
    }

    pub fn exec_or_undo_all(&mut self, command: impl CommandTrait<R,E,U> + Send + Sync + 'static) -> Result<R, E> {
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

    pub fn undo(&mut self) -> Option<Result<(), U>> {

        if let Some(command) = self.commands.pop() {

            let unexec = command.get_unexec();
            let result = unexec();
            self.undo_commands.push(command);
            Some(result)

        } else { None }

    }

    pub fn redo(&mut self) -> Option<Result<R, E>> {

        if let Some(command) = self.undo_commands.pop() {

            let exec = command.get_exec();
            let result = exec();
            self.commands.push(command);
            Some(result)

        } else { None }

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
            exec: Box::new($($move_exec)* || $exec_body),
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
