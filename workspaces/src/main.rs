use std::env;

mod sales;

fn main() {
    // Collect the arguments into a vector
    let args: Vec<String> = env::args().collect();

    // Check if we received at least one argument (excluding the program name)
    if args.len() > 1 {
        let argument = &args[1]; // This is your argument

        // Example if-else using the argument
        if argument == "sales" {
            let _ = sales::main();
        } else if argument == "stake" {
            let _ = sales::main();
        } else {

        }
    } else {
        println!("No argument was provided");
    }
}
