#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::process;
use std::slice;

struct Tape {
    head: usize,
    cells: Vec<u8>,
}

impl Tape {
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(1024);
        cells.push(0);
        Tape {
            head: 0,
            cells,
        }
    }

    pub fn right(&mut self) {
        if self.head == self.cells.len() - 1 {
            self.cells.push(0);
        }
        self.head += 1;
    }

    pub fn left(&mut self) {
        if self.head == 0 {
            panic!("Tried to go to a negative tape index!");
        } else {
            self.head -= 1;
        }
    }

    pub fn increment(&mut self) {
        self.cells[self.head] = self.cells[self.head].wrapping_add(1);
    }

    pub fn decrement(&mut self) {
        self.cells[self.head] = self.cells[self.head].wrapping_sub(1);
    }

    pub fn output(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(slice::from_ref(&self.cells[self.head]))
    }

    pub fn input(&mut self, reader: &mut impl Read) -> io::Result<()> {
        if let Err(e) = reader.read_exact(slice::from_mut(&mut self.cells[self.head])) {
            match e.kind() {
                io::ErrorKind::UnexpectedEof => {
                    self.cells[self.head] = 0;
                    Ok(())
                }
                _ => Err(e),
            }
        } else {
            Ok(())
        }
    }

    pub fn is_zero(&self) -> bool {
        self.cells[self.head] == 0
    }
}

struct Interpreter<I, O> {
    input: I,
    output: O,
    tape: Tape,
}

impl<I: Read, O: Write> Interpreter<I, O> {
    pub fn new(input: I, output: O) -> Self {
        Self {
            input,
            output,
            tape: Tape::new(),
        }
    }

    pub fn run(&mut self, code: &[u8]) -> io::Result<()> {
        let mut i: usize = 0;
        loop {
            if i == code.len() {
                break;
            }

            match code[i] as char {
                '>' => self.tape.right(),
                '<' => self.tape.left(),
                '+' => self.tape.increment(),
                '-' => self.tape.decrement(),
                '.' => self.tape.output(&mut self.output)?,
                ',' => self.tape.input(&mut self.input)?,
                '[' => {
                    if self.tape.is_zero() {
                        i = self.matching_for_left_paren(i, code)?;
                    }
                }
                ']' => {
                    if !self.tape.is_zero() {
                        i = self.matching_for_right_paren(i, code)?;
                    }
                }
                _ => {}
            }
            i += 1;
        }

        Ok(())
    }

    pub fn output(&self) -> &O {
        &self.output
    }

    pub fn into_output(self) -> O {
        self.output
    }

    pub fn matching_for_left_paren(&self, current_index: usize, code: &[u8]) -> io::Result<usize> {
        let mut encountered: usize = 0;

        for (i, &character) in code[(current_index + 1)..].iter().enumerate() {
            match character as char {
                '[' => {
                    encountered += 1;
                }
                ']' => {
                    if encountered == 0 {
                        return Ok(i);
                    }
                    encountered -= 1
                }
                _ => {}
            }
        }

        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid code"))
    }

    pub fn matching_for_right_paren(&self, current_index: usize, code: &[u8]) -> io::Result<usize> {
        let mut encountered: usize = 0;

        for (i, &character) in code[..current_index].iter().enumerate().rev() {
            match character as char {
                ']' => {
                    encountered += 1;
                }
                '[' => {
                    if encountered == 0 {
                        return Ok(i);
                    }
                    encountered -= 1
                }
                _ => {}
            }
        }

        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid code"))
    }
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 2 || args[1] == "-h" || args[1] == "--help" {
        eprintln!("Usage: brainruck SOURCE_FILE");
        process::exit(1);
    }

    let mut code = String::new();
    BufReader::new(File::open(&args[1])?).read_to_string(&mut code)?;

    let input = BufReader::new(io::stdin());
    let output = BufWriter::new(io::stdout());

    let mut interpreter = Interpreter::new(input, output);
    interpreter.run(code.as_bytes())?;

    Ok(())
}
