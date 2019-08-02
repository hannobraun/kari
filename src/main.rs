mod interpreter;
mod tokenizer;


use tokenizer::Tokenizer;


fn main() {
    let program = include_str!("../examples/hello_world.kr");

    let tokenizer   = Tokenizer::new(program.chars());
    let interpreter = interpreter::Interpreter::new(tokenizer);

    interpreter.run();
}
