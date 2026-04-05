use std::{
    collections::{HashMap},
    env::args,
    fs::File,
    io::{Read, Write, stdin, stdout},
};
#[derive(PartialEq, Eq)]
enum Block {
    /* Jump for data pointer */
    Jump(isize),
    /* Increments cell under data pointer to a value specified by u8 */
    Modify(u8),
    /* Read from stdin */
    Read,
    /* Write to stdout */
    Write,
    /* Loop block for whiles `[]` and the skelet of the program */
    Loop(Vec<Block>),
    /* None value. Here because i am too lazy to do all properly */
    None
}
fn main() {
    let path = args().skip(1).next().unwrap();
    let mut file = File::open(path).unwrap();
    let mut code = Vec::new();
    file.read_to_end(&mut code).unwrap();
    let tree = start_parse(code.as_slice());
    run(tree);
}
/* State of BF program */
struct State {
    /* Data pointer position */
    datap: isize,
    /* memory */
    heap: HashMap<isize, u8>,
}
/* Entrance to recursive running of BF program */
fn run(b: Block) {
    let mut state = State {
        datap: 0,
        heap: HashMap::new(),
    };
    match b {
        Block::Loop(blocks) => {
            for b in blocks {
                execute(&b, &mut state);
            }
        }
        _ => unreachable!(),
    }
}
fn execute(b: &Block, state: &mut State) {
    match b {
        Block::Jump(amount) => state.datap += amount,
        Block::Modify(amount) => {
            state.heap.insert(
                state.datap,
                state
                    .heap
                    .get(&state.datap)
                    .unwrap_or(&0)
                    .wrapping_add(*amount),
            );
        }
        Block::Read => {
            let mut buf = [0u8];
            stdin().read(&mut buf).unwrap();
            state.heap.insert(state.datap, buf[0]);
        }
        Block::Write => {
            stdout().write_all(&[*state.heap.get(&state.datap).unwrap_or(&0)]).unwrap();
        }
        Block::Loop(blocks) => loop {
            let cond = *state.heap.get(&state.datap).unwrap_or(&0) != 0;
            if cond {
                for b in blocks {
                    execute(b, state);
                }
            } else {
                break;
            }
        },
        Block::None => {}
    }
}
fn start_parse(code: &[u8]) -> Block {
    let mut jump = 0;
    while !b",.<>+-[]".contains(&code[jump]) { /*                  */
        jump += 1;                             /* skiping comments */
    }                                          /*                  */
    let mut l = Vec::new();
    while jump < code.len() {
        let (b, j) = parse(&code[jump..]);
        l.push(b);
        jump += j;
    }
    return Block::Loop(l);
}
fn parse(code: &[u8]) -> (Block, usize) {
    enum ManyMode {
        Jump,
        Modify,
    }
    let mut jump = 0;
    while jump < code.len() && !(b",.<>+-[]".contains(&code[jump])) { /*                  */
        jump += 1;                                                    /* skiping comments */
    }                                                                 /*                  */
    if jump >= code.len(){
        return (Block::None, jump);
    }
    if code[jump] == b'[' {
        let mut l = Vec::new();
        jump += 1;
        while jump < code.len() && code[jump] != b']' {
            let (b, j) = parse(&code[jump..]);
            jump += j;
            while jump < code.len() && !(b",.<>+-[]".contains(&code[jump])) { /*                  */
                jump += 1;                                                    /* skiping comments */  
            }                                                                 /*       yes, again */  
            
            if b != Block::None { /*                                                             */
                l.push(b);        /* skiping None blocks because they are part of my imagination */
            }                     /*                                                             */  
        }
        jump += 1;
        return (Block::Loop(l), jump);
    } else if code[jump] == b'.' {
        return (Block::Write, jump + 1);
    } else if code[jump] == b',' {
        return (Block::Read, jump + 1);
    } else { //here is the optimization
        let mode = match code[jump] { //deciding which type of serial operator is that block
            b'>' | b'<' => ManyMode::Jump,
            b'+' | b'-' => ManyMode::Modify,
            v => unreachable!("got this char '{}'", v as char),
        };
        let mut v = 0;
        loop {
            if jump >= code.len() {
                break;
            };
            match mode {
                ManyMode::Jump => match code[jump] {
                    b'>' => v += 1,
                    b'<' => v -= 1,
                    _ => break,
                },
                ManyMode::Modify => match code[jump] {
                    b'+' => v += 1,
                    b'-' => v -= 1,
                    _ => break,
                },
            }
            jump += 1;
        }
        match mode {
            ManyMode::Jump => {
                return (Block::Jump(v), jump);
            }
            ManyMode::Modify => {
                return (Block::Modify((v % 256) as u8), jump);
            }
        }
    }
}
