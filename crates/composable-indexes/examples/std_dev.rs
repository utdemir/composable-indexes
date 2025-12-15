use composable_indexes::{aggregation::std_dev, core::Collection};

fn main() {
    // Create a collection with a standard deviation index
    let mut temperatures = Collection::new(std_dev::<f64>());

    println!("Temperature readings standard deviation tracker");
    println!("==============================================\n");

    // Add some temperature readings
    println!("Adding temperature readings...");
    let readings = vec![20.5, 22.3, 19.8, 21.0, 23.5, 20.0, 22.8];
    let mut keys = Vec::new();

    for (i, &temp) in readings.iter().enumerate() {
        let key = temperatures.insert(temp);
        keys.push(key);
        let std_dev = temperatures.query(|ix| ix.get());
        println!(
            "  Reading {}: {:.1}°C - Current std dev: {:.4}°C",
            i + 1,
            temp,
            std_dev
        );
    }

    println!(
        "\n✓ Final standard deviation: {:.4}°C",
        temperatures.query(|ix| ix.get())
    );

    // Remove some readings
    println!("\nRemoving outlier readings...");
    for (i, key) in keys.iter().take(2).enumerate() {
        temperatures.delete_by_key(*key);
        let std_dev = temperatures.query(|ix| ix.get());
        println!(
            "  After removing reading {}: std dev = {:.4}°C",
            i + 1,
            std_dev
        );
    }

    println!(
        "\n✓ Final standard deviation after cleanup: {:.4}°C",
        temperatures.query(|ix| ix.get())
    );
}
