use std::io;

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

fn main() {
    println!("Please enter the temperature value: \n");

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let temp_value: f64 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("\nPlease enter a valid number for temperature value.\n");
                continue;
            }
        };

        if temp_value < -273.15 {
            println!("\nTemperature below absolute zero is not possible. Please enter a valid temperature.\n");
            continue;
        }

        if temp_value.is_infinite() || temp_value.is_nan() {
            println!("\nPlease enter a finite number for temperature value.\n");
            continue;
        }

        println!("\nSelect the unit to convert from (C/F/K): \n");
        let mut from_unit = String::new();
        io::stdin()
            .read_line(&mut from_unit)
            .expect("Failed to read line");
        let from_unit = match from_unit.trim().to_uppercase().as_str() {
            "C" => TemperatureUnit::Celsius,
            "F" => TemperatureUnit::Fahrenheit,
            "K" => TemperatureUnit::Kelvin,
            _ => {
                println!("\nInvalid unit. Please enter C, F, or K.\n");
                continue;
            }
        };

        println!("\nSelect the unit to convert to (C/F/K): \n");
        let mut to_unit = String::new();
        io::stdin()
            .read_line(&mut to_unit)
            .expect("Failed to read line");
        let to_unit = match to_unit.trim().to_uppercase().as_str() {
            "C" => TemperatureUnit::Celsius,
            "F" => TemperatureUnit::Fahrenheit,
            "K" => TemperatureUnit::Kelvin,
            _ => {
                println!("\nInvalid unit. Please enter C, F, or K.\n");
                continue;
            }
        };

        let converted_value = convert_temp(temp_value, from_unit, to_unit);
        println!("\nConverted temperature: {:.2}", converted_value);
    }
}
