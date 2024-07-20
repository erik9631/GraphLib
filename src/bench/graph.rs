use std::time::Instant;
use crate::graph;
use crate::graph::MSize;
use crate::graph::TraverseResult::Continue;

#[test]
pub fn graph_disconnect_bench(){
    // prepare data
    let data_size = 20000;
    let mut graph = graph::Graph::with_reserve(data_size);
    let root = graph.create_leaf(0);
    let mut handles = Vec::with_capacity(data_size);
    for i in 0..data_size {
        handles.push(graph.create_and_connect_leaf(root.edge_handle, i+1));
    }

    let start = Instant::now();
    while handles.len() > 0 {
        let handle = handles.pop().unwrap();
        graph.edges.disconnect(root.edge_handle, handle.edge_handle);
    }
    assert_eq!(graph.edges.len(root.edge_handle), 0);
    println!("Time taken: {:?}", start.elapsed());
}

#[test]
pub fn bfs_bench(){
    // prepare data
    let data_size = 1020;
    let mut graph = graph::Graph::with_reserve(data_size);
    let root = graph.create_leaf(0);
    let mut number_of_nodes = 1;
    for i in 0..data_size {
        let child = graph.create_and_connect_leaf(root.edge_handle, i+1);
        number_of_nodes += 1;
        for j in 0..data_size {
            graph.create_and_connect_leaf(child.edge_handle, (j*data_size));
            number_of_nodes += 1;
        }
    }

    let start = Instant::now();
    graph.bfs(root.edge_handle, |graph, handle|{
        graph.vertices[handle.vertex_handle] = 0;
        return Continue;
    });
    println!("Time taken: {:?}", start.elapsed());
}