use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
struct EducationData {
    country_or_area: String,
    year: u32,
    indicator: String,
    series: String,
    value: Option<f64>,
}

struct Graph {
    nodes: Vec<String>,
    adjacency_matrix: Vec<Vec<f64>>,
}

fn main() {
    let csv_file_path = "/Users/franklinwibisono/Downloads/finalcopy/SYB66_309_202310_Education.csv";

    // Load and preprocess data
    match load_and_preprocess_data(csv_file_path) {
        Ok(data) => {
            // Construct a graph from the data
            let graph = construct_graph(&data);
            
            // Perform clustering and other operations
            let clusters = cluster_graph(&graph);
            
            // Print the clusters and the adjacency matrix
            print_clusters(&clusters, &graph);
        },
        Err(e) => {
            eprintln!("Failed to load data: {:?}", e);
        },
    }
}

fn load_and_preprocess_data(csv_file_path: &str) -> io::Result<Vec<EducationData>> {
    // Open the CSV file
    let file = File::open(csv_file_path)?;
    let reader = BufReader::new(file);

    let mut data = Vec::new();

    // Read each line of the CSV file
    for (line_index, line) in reader.lines().enumerate() {
        let line = line?;
        
        // Skip the header line
        if line_index == 0 {
            continue;
        }
        
        // Split the line into fields
        let fields: Vec<&str> = line.split(',').collect();
        
        // Check for correct number of fields
        if fields.len() < 5 {
            continue;
        }

        // Extract data fields
        let country_or_area = fields[0].to_string();
        let year: u32 = fields[1].parse().unwrap_or(0);
        let indicator = fields[2].to_string();
        let series = fields[3].to_string();
        let value: Option<f64> = fields[4].parse().ok();

        // Push the EducationData object to data vector
        data.push(EducationData {
            country_or_area,
            year,
            indicator,
            series,
            value,
        });
    }

    Ok(data)
}

fn construct_graph(data: &[EducationData]) -> Graph {
    let mut nodes = Vec::new();
    let mut adjacency_matrix = Vec::new();
    let mut node_indices = HashMap::new();

    // Initialize nodes and adjacency matrix
    for record in data {
        let country_or_area = &record.country_or_area;

        // If the country is not yet in the graph, add it
        let node_index = *node_indices
            .entry(country_or_area.clone())
            .or_insert_with(|| {
                nodes.push(country_or_area.clone());
                adjacency_matrix.push(vec![0.0; nodes.len()]);
                nodes.len() - 1
            });

        // Update the adjacency matrix based on the value and the year of the record
        for target_index in 0..adjacency_matrix.len() {
            let adjustment_factor = record.year as f64 * 0.01; // Example usage of year
            let value_to_add = record.value.unwrap_or(0.0) * adjustment_factor;
            adjacency_matrix[node_index][target_index] += value_to_add;
        }
    }

    Graph {
        nodes,
        adjacency_matrix,
    }
}

fn cluster_graph(_graph: &Graph) -> Vec<Vec<usize>> {
    // Placeholder clustering algorithm. You can replace this with a real implementation.
    Vec::new()
}

fn print_clusters(clusters: &Vec<Vec<usize>>, graph: &Graph) {
    // Print the clusters
    for (cluster_index, cluster) in clusters.iter().enumerate() {
        println!("Cluster {}:", cluster_index);
        for &node_index in cluster {
            println!("  - {}", graph.nodes[node_index]);
        }
    }

    // Print the adjacency matrix for debugging and visualization
    println!("\nAdjacency Matrix:");
    for row in &graph.adjacency_matrix {
        println!("{:?}", row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    // A helper function to capture the standard output of a function
    fn capture_output<F>(func: F) -> String
    where
        F: FnOnce(&mut dyn Write),
    {
        let mut buffer = Cursor::new(Vec::new());
        func(&mut buffer);
        String::from_utf8(buffer.into_inner()).unwrap()
    }

    #[test]
    fn test_print_clusters() {
        // Define nodes
        let nodes = vec!["USA".to_string(), "Canada".to_string()];

        // Define clusters
        let clusters = vec![
            vec![0], // Cluster containing USA
            vec![1], // Cluster containing Canada
        ];

        // Define an adjacency matrix for the graph
        let adjacency_matrix = vec![
            vec![1.0, 0.5], // USA to USA and USA to Canada
            vec![0.5, 2.0], // Canada to USA and Canada to Canada
        ];

        // Create a graph struct with nodes and adjacency matrix
        let graph = Graph {
            nodes: nodes.clone(),
            adjacency_matrix: adjacency_matrix.clone(),
        };

        // Capture the output of the print_clusters function
        let output = capture_output(|writer| print_clusters(&clusters, &graph));

        // Clean up the captured output to remove extra newlines
        let cleaned_output = output.trim_end().to_string();

        // Assert expected output
        let expected_output = "Cluster 0:\n  - USA\nCluster 1:\n  - Canada\n\nAdjacency Matrix:\n[1.0, 0.5]\n[0.5, 2.0]";
        assert_eq!(cleaned_output, expected_output);
    }

    // Additional tests for other functionality...
}
