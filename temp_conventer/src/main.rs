use std::io;
use clap::Parser;

#[derive(Parser)]
#[command(name = "temp_converter")]
#[command(about = "A simple temperature converter CLI")]
struct Args {
    /// The temperature value to convert
    #[arg(long)]
    value: Option<f64>,

    /// The unit of the input temperature (C, F, K)
    #[arg(long)]
    from: Option<String>,

    /// The unit to convert to (C, F, K)
    #[arg(long)]
    to: Option<String>,
}

enum TemperatureUnit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

fn convert_temp(value: f64, from: TemperatureUnit, to: TemperatureUnit) -> f64 {
    match (from, to) {
        (TemperatureUnit::Celsius, TemperatureUnit::Fahrenheit) => value * 9.0 / 5.0 + 32.0,
        (TemperatureUnit::Celsius, TemperatureUnit::Kelvin) => value + 273.15,
        (TemperatureUnit::Kelvin, TemperatureUnit::Fahrenheit) => {
            (value - 273.15) * 9.0 / 5.0 + 32.0
        }
        (TemperatureUnit::Kelvin, TemperatureUnit::Celsius) => value - 273.15,
        (TemperatureUnit::Fahrenheit, TemperatureUnit::Celsius) => (value - 32.0) * 5.0 / 9.0,
        (TemperatureUnit::Fahrenheit, TemperatureUnit::Kelvin) => {
            (value - 32.0) * 5.0 / 9.0 + 273.15
        }
        _ => value,
    }
}

impl TemperatureUnit {
    fn from_str(input: &str) -> Option<TemperatureUnit> {
        match input.to_uppercase().as_str() {
            "C" => Some(TemperatureUnit::Celsius),
            "F" => Some(TemperatureUnit::Fahrenheit),
            "K" => Some(TemperatureUnit::Kelvin),
            _ => None,
        }
    }
}

fn main() {
    let args = Args::parse();

    if let (Some(value), Some(from_str), Some(to_str)) = (args.value, args.from, args.to) {
        let from_unit = match TemperatureUnit::from_str(&from_str) {
            Some(unit) => unit,
            None => {
                eprintln!("Error: Invalid 'from' unit '{}'. Use C, F, or K.", from_str);
                std::process::exit(1);
            }
        };

        let to_unit = match TemperatureUnit::from_str(&to_str) {
            Some(unit) => unit,
            None => {
                eprintln!("Error: Invalid 'to' unit '{}'. Use C, F, or K.", to_str);
                std::process::exit(1);
            }
        };

        if value < -273.15 {
            eprintln!("Error: Temperature below absolute zero is not possible.");
            std::process::exit(1);
        }

        if value.is_infinite() || value.is_nan() {
            eprintln!("Error: Please enter a finite number for temperature value.");
            std::process::exit(1);
        }

        let converted_value = convert_temp(value, from_unit, to_unit);
        println!("Converted temperature: {:.2}", converted_value);
        return;
    }

    // Interactive mode
    println!("Please enter the temperature value (or type 'exit' to quit): \n");

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed_input = input.trim();
        
        if trimmed_input.to_lowercase() == "exit" {
            println!("\nExiting the temperature converter. Goodbye!");
            break;
        }
        
        let temp_value: f64 = match trimmed_input.parse() {
            Ok(num) => num,
            Err(_) => {
                println!("\nPlease enter a valid number for temperature value or 'exit' to quit.\n");
                continue;
            }
        };

        if temp_value < -273.15 {
            println!(
                "\nTemperature below absolute zero is not possible. Please enter a valid temperature.\n"
            );
            continue;
        }

        if temp_value.is_infinite() || temp_value.is_nan() {
            println!("\nPlease enter a finite number for temperature value.\n");
            continue;
        }

        println!("\nPlease enter the unit of the temperature (C, F, K): \n");
        let mut unit_input = String::new();
        io::stdin()
            .read_line(&mut unit_input)
            .expect("Failed to read line");
        let from_unit = match TemperatureUnit::from_str(unit_input.trim()) {
            Some(unit) => unit,
            None => {
                println!("\nInvalid unit. Please enter C, F, or K.\n");
                continue;
            }
        };

        println!("\nPlease enter the unit to convert to (C, F, K): \n");
        let mut to_unit_input = String::new();
        io::stdin()
            .read_line(&mut to_unit_input)
            .expect("Failed to read line");
        let to_unit = match TemperatureUnit::from_str(to_unit_input.trim()) {
            Some(unit) => unit,
            None => {
                println!("\nInvalid unit. Please enter C, F, or K.\n");
                continue;
            }
        };

        let converted_value = convert_temp(temp_value, from_unit, to_unit);
        println!("\nConverted temperature: {:.2}\n", converted_value);
    }
}
