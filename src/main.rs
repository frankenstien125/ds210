use csv::ReaderBuilder;
use serde_derive::Deserialize;
use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::algo::Louvain::louvain;
use petgraph::visit::EdgeRef;
use ndarray::Array2;
use smartcore::cluster::kmeans::KMeans;
use smartcore::dataset::*;
use smartcore::linalg::basic::matrix::DenseMatrix;

#[derive(Debug, Deserialize)]
struct EducationData {
    country_or_area: String,
    year: u32,
    indicator: String,
    series: String,
    value: Option<f64>,
    // Add more fields as necessary
}

fn main() -> Result<(), Box<dyn Error>> {
    // Define the path to the CSV file
    let csv_file_path = "/Users/franklinwibisono/Downloads/final/SYB66_309_202310_Education.csv";

    // Load the CSV data
    let data = load_and_preprocess_data(csv_file_path)?;

    // Construct a graph from the preprocessed data
    let (graph, node_labels) = construct_graph(&data)?;

    // Apply graph clustering
    let clusters = cluster_graph(&graph);

    // Print the clusters for verification
    print_clusters(&clusters, &node_labels);

    // Compute k-means clustering
    let k_means_clusters = compute_k_means(&graph, &data)?;

    // Count nodes and vertices
    count_nodes_and_edges(&graph);

    Ok(())
}

fn load_and_preprocess_data(csv_file_path: &str) -> Result<Vec<EducationData>, Box<dyn Error>> {
    // Open the CSV file
    let file = File::open(csv_file_path)?;

    // Create a CSV reader with headers
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    // Prepare to store data for further processing
    let mut data: Vec<EducationData> = Vec::new();

    // Iterate through CSV records and deserialize each one into an `EducationData` struct
    for result in reader.deserialize() {
        // Deserialize each record
        let record: EducationData = result?;
        // Store the record for later processing
        data.push(record);
    }

    Ok(data)
}

fn construct_graph(data: &[EducationData]) -> Result<(Graph<String, f64>, Vec<String>), Box<dyn Error>> {
    // Create a new graph
    let mut graph = Graph::new_undirected();

    // Create a map to store the index of each country node
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();
    // Store the node labels
    let mut node_labels: Vec<String> = Vec::new();

    // Add nodes and edges based on data
    for record in data {
        // Extract data from the record
        let country = &record.country_or_area;
        let value = record.value.unwrap_or(0.0);

        // Add the country as a node if not already present
        let node_index = *node_indices
            .entry(country.to_string())
            .or_insert_with(|| {
                node_labels.push(country.to_string());
                graph.add_node(country.to_string())
            });

        // Example: Create edges with weights based on value (similarity metric)
        // In a real-world scenario, you should compute similarity metrics

        // For now, just create an edge with the given value
        graph.update_edge(node_index, node_index, value);
    }

    Ok((graph, node_labels))
}

fn cluster_graph(graph: &Graph<String, f64>) -> Vec<Vec<NodeIndex>> {
    // Apply the Louvain method for community detection
    let (clusters, _) = louvain(graph, None);
    
    // Convert the cluster results to a list of node indices
    let mut cluster_nodes = vec![];
    
    for (node, cluster_id) in clusters {
        // Ensure the vector is large enough to accommodate all clusters
        if cluster_id >= cluster_nodes.len() {
            cluster_nodes.resize_with(cluster_id + 1, Vec::new);
        }
        
        // Add the node index to the respective cluster
        cluster_nodes[cluster_id].push(node);
    }
    
    cluster_nodes
}

fn print_clusters(clusters: &Vec<Vec<NodeIndex>>, node_labels: &Vec<String>) {
    // Print the clusters with node labels for verification
    for (index, cluster) in clusters.iter().enumerate() {
        println!("Cluster {}: {:?}", index, cluster.iter().map(|node| &node_labels[node.index()]).collect::<Vec<_>>());
    }
}

fn compute_k_means(graph: &Graph<String, f64>, data: &[EducationData]) -> Result<Vec<usize>, Box<dyn Error>> {
    // Convert graph adjacency matrix to ndarray::Array2
    let num_nodes = graph.node_count();
    let mut adjacency_matrix = Array2::zeros((num_nodes, num_nodes));

    for edge in graph.edge_references() {
        let (source, target) = (edge.source(), edge.target());
        let weight = edge.weight();

        adjacency_matrix[[source.index(), target.index()]] = *weight;
        adjacency_matrix[[target.index(), source.index()]] = *weight;
    }

    // Use DenseMatrix from smartcore
    let matrix = DenseMatrix::from_array(num_nodes, num_nodes, &adjacency_matrix.into_raw_vec());

    // Define k-means clustering
    let k = 3; // Choose the number of clusters
    let kmeans = KMeans::new(k);

    // Fit k-means to the matrix
    let results = kmeans.fit_predict(&matrix)?;

    // Print k-means results
    println!("k-Means clustering results: {:?}", results);

    Ok(results)
}

fn count_nodes_and_edges(graph: &Graph<String, f64>) {
    // Count the number of nodes and edges in the graph
    let num_nodes = graph.node_count();
    let num_edges = graph.edge_count();

    println!("Number of nodes in the graph: {}", num_nodes);
    println!("Number of edges in the graph: {}", num_edges);
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::UnGraph;

    #[test]
    fn test_load_and_preprocess_data() {
        let csv_file_path = "test_data.csv"; // Provide a path to a test CSV file
        let result = load_and_preprocess_data(csv_file_path);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.len() > 0, "Data should not be empty");
    }

    #[test]
    fn test_construct_graph() {
        let test_data = vec![
            EducationData {
                country_or_area: "Country1".to_string(),
                year: 2020,
                indicator: "Indicator1".to_string(),
                series: "Series1".to_string(),
                value: Some(10.0),
            },
            EducationData {
                country_or_area: "Country2".to_string(),
                year: 2020,
                indicator: "Indicator2".to_string(),
                series: "Series2".to_string(),
                value: Some(20.0),
            },
            // Add more test data as necessary
        ];

        let result = construct_graph(&test_data);
        assert!(result.is_ok());

        let (graph, node_labels) = result.unwrap();
        assert_eq!(graph.node_count(), test_data.len());
        assert_eq!(node_labels.len(), test_data.len());
    }

    #[test]
    fn test_cluster_graph() {
        let test_data = vec![
            EducationData {
                country_or_area: "Country1".to_string(),
                year: 2020,
                indicator: "Indicator1".to_string(),
                series: "Series1".to_string(),
                value: Some(10.0),
            },
            EducationData {
                country_or_area: "Country2".to_string(),
                year: 2020,
                indicator: "Indicator2".to_string(),
                series: "Series2".to_string(),
                value: Some(20.0),
            },
            // Add more test data as necessary
        ];

        let (graph, _) = construct_graph(&test_data).unwrap();
        let clusters = cluster_graph(&graph);

        // Verify that clustering produces some clusters
        assert!(clusters.len() > 0);
    }

    #[test]
    fn test_compute_k_means() {
        let test_data = vec![
            EducationData {
                country_or_area: "Country1".to_string(),
                year: 2020,
                indicator: "Indicator1".to_string(),
                series: "Series1".to_string(),
                value: Some(10.0),
            },
            EducationData {
                country_or_area: "Country2".to_string(),
                year: 2020,
                indicator: "Indicator2".to_string(),
                series: "Series2".to_string(),
                value: Some(20.0),
            },
            // Add more test data as necessary
        ];

        let (graph, _) = construct_graph(&test_data).unwrap();
        let k_means_result = compute_k_means(&graph, &test_data);
        assert!(k_means_result.is_ok());

        // Verify k-means clustering produces valid results
        let k_means_clusters = k_means_result.unwrap();
        assert!(k_means_clusters.len() > 0);
    }

    #[test]
    fn test_count_nodes_and_edges() {
        let test_data = vec![
            EducationData {
                country_or_area: "Country1".to_string(),
                year: 2020,
                indicator: "Indicator1".to_string(),
                series: "Series1".to_string(),
                value: Some(10.0),
            },
            EducationData {
                country_or_area: "Country2".to_string(),
                year: 2020,
                indicator: "Indicator2".to_string(),
                series: "Series2".to_string(),
                value: Some(20.0),
            },
            // Add more test data as necessary
        ];

        let (graph, _) = construct_graph(&test_data).unwrap();

        // Call the function to count nodes and edges
        count_nodes_and_edges(&graph);
        // In the current code, the function only prints output,
        // so you can verify the output manually or use a different
        // approach to assert the expected number of nodes and edges.
    }
}
