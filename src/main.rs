
mod examples;
use examples::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with the number of the tutorial, e.g. `1_1_2`");
        std::process::exit(1);
    }
    let tutorial_id = &args[1];

    match tutorial_id.as_str() {
        "1_1" => main_1_1(),
        "1_2" => main_1_2(),
        "1_2_1" => main_1_2_1(),
        "1_3" => main_1_3(),
        "1_3_1" => main_1_3_1(),
        "1_4" => main_1_4(),
        "1_4_1" => main_1_4_1(),
        "1_5" => main_1_5(),
        "1_5_1" => main_1_5_1(),
        _     => println!("Unknown tutorial id")
    }
}
